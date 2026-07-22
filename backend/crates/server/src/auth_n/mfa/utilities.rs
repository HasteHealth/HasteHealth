use haste_encryption::{
    Encryptor as _, SecretsProvider, encryption::aes::AesGcmEncryptor, traits::EncryptionResult,
};
use haste_fhir_model::r4::generated::terminology::IssueType;
use haste_fhir_operation_error::OperationOutcomeError;
use haste_repository::types::{mfa::UserMFACredential, user::User};

use crate::config::ServerConfig;

fn parse_totp_algorithm(algorithm: &str) -> Result<totp_rs::Algorithm, OperationOutcomeError> {
    match algorithm {
        "SHA1" => Ok(totp_rs::Algorithm::SHA1),
        "SHA256" => Ok(totp_rs::Algorithm::SHA256),
        "SHA512" => Ok(totp_rs::Algorithm::SHA512),
        other => {
            tracing::error!(
                algorithm = other,
                "Unrecognized TOTP algorithm stored for MFA credential"
            );

            Err(OperationOutcomeError::error(
                IssueType::exception(),
                "Could not generate TOTP for MFA".to_string(),
            ))
        }
    }
}

pub fn get_aes_key(config: &ServerConfig) -> Result<&str, OperationOutcomeError> {
    let Some(aes_key_id) = config.security.aes_key.as_ref() else {
        return Err(OperationOutcomeError::error(
            IssueType::exception(),
            "AES key is not configured for MFA encryption".to_string(),
        ));
    };

    Ok(aes_key_id)
}

pub async fn get_aes_encryptor(
    secret_provider: &dyn SecretsProvider,
    config: &ServerConfig,
) -> Result<AesGcmEncryptor, OperationOutcomeError> {
    let aes_key_id = get_aes_key(config)?;
    let aes_secret = secret_provider.get_secret(aes_key_id).await?;
    let aes_encryptor = AesGcmEncryptor::new(aes_secret.expose_bytes())?;
    Ok(aes_encryptor)
}

async fn get_totp_secret(
    secret_provider: &dyn SecretsProvider,
    config: &ServerConfig,
    user_mfa_credential: &UserMFACredential,
) -> Result<Vec<u8>, OperationOutcomeError> {
    let aes_encryptor = get_aes_encryptor(secret_provider, config).await?;

    let decrypted_secret = aes_encryptor.decrypt(&EncryptionResult {
        nonce: user_mfa_credential.secret_nonce.clone(),
        ciphertext: user_mfa_credential.secret_ciphertext.clone(),
    })?;

    Ok(decrypted_secret)
}

pub async fn user_mfa_to_totp(
    secret_provider: &dyn SecretsProvider,
    config: &ServerConfig,
    user: &User,
    user_mfa_credential: UserMFACredential,
) -> Result<totp_rs::TOTP, OperationOutcomeError> {
    let totp = totp_rs::TOTP::new(
        parse_totp_algorithm(&user_mfa_credential.totp_algorithm)?,
        user_mfa_credential.totp_digits as usize,
        user_mfa_credential.totp_skew as u8,
        user_mfa_credential.totp_period as u64,
        get_totp_secret(secret_provider, config, &user_mfa_credential).await?,
        Some("Haste Health".to_string()),
        user.email.clone().unwrap_or(user.id.clone()),
    )
    .map_err(|e| {
        tracing::error!(error = ?e);

        OperationOutcomeError::error(
            IssueType::exception(),
            "Could not generate TOTP for MFA".to_string(),
        )
    })?;

    Ok(totp)
}
