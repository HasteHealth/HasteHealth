use axum::{
    extract::{OriginalUri, State},
    response::{IntoResponse as _, Redirect, Response},
};
use axum_extra::{extract::Cached, routing::TypedPath};
use haste_encryption::{Encryptor as _, encryption::aes::AesGcmEncryptor};
use haste_fhir_model::r4::generated::terminology::IssueType;
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_repository::{
    Repository,
    admin::TenantModelAdmin,
    types::{
        mfa::{UserMFACredential, UserMFACredentialCreate, UserMFASearchClaims},
        scope::UserId,
    },
};
use serde::Deserialize;
use std::sync::Arc;
use tower_sessions::Session;

use crate::{auth_n::session, extract::path_tenant::TenantIdentifier, services::ServerState};

pub fn redirect_to_mfa_activation(
    uri: &OriginalUri,
    user_mfa_credential: &UserMFACredential,
    replace_path: &str,
) -> String {
    uri.path().to_string().replace(
        replace_path,
        &format!("/register/{}", user_mfa_credential.id),
    )
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/create")]
pub struct MFACreatePOST;

// Creates User MFA credentials if user is allowed further MFA credentials.
// If user has reached the maximum allowed MFA credentials, an error is returned.
pub async fn create_post<
    Repo: Repository + Send + Sync,
    Search: SearchEngine + Send + Sync,
    Terminology: FHIRTerminology + Send + Sync,
>(
    _: MFACreatePOST,
    uri: OriginalUri,
    Cached(TenantIdentifier { tenant }): Cached<TenantIdentifier>,
    State(state): State<Arc<ServerState<Repo, Search, Terminology>>>,
    Cached(current_session): Cached<Session>,
) -> Result<Response, OperationOutcomeError> {
    let completed_auth_state = session::user::get_completed_authorization_state(&current_session)
        .await
        .map_err(|_e| {
            OperationOutcomeError::error(
                IssueType::Security(None),
                "User is not logged in.".to_string(),
            )
        })?;

    let existing_mfa_credentials = TenantModelAdmin::<UserMFACredentialCreate, _, _, _, _>::search(
        state.repo.as_ref(),
        &tenant,
        &UserMFASearchClaims {
            tenant: tenant.clone(),
            user_id: UserId::new(completed_auth_state.user.id.clone()),
            is_active: None,
        },
    )
    .await?;

    // Todo make this amount configurable
    if existing_mfa_credentials.len() > state.config.security.mfa.max_credentials_per_user {
        return Err(OperationOutcomeError::error(
            IssueType::Security(None),
            format!(
                "User has reached the maximum of {} MFA credentials allowed.",
                state.config.security.mfa.max_credentials_per_user
            ),
        ));
    }

    let Some(aes_key_id) = state.config.security.aes_key.as_ref() else {
        return Err(OperationOutcomeError::error(
            IssueType::Exception(None),
            "AES key is not configured for MFA encryption".to_string(),
        ));
    };

    let aes_secret = state.secret_provider.get_secret(aes_key_id).await?;

    let aes_encryptor = AesGcmEncryptor::new(aes_secret.expose_bytes())?;

    let secret = totp_rs::Secret::default().to_bytes().map_err(|e| {
        tracing::error!(error = ?e);

        OperationOutcomeError::error(
            IssueType::Exception(None),
            "Could not generate secret for MFA".to_string(),
        )
    })?;

    let aes_encrypted_secret = aes_encryptor.encrypt(&secret)?;

    let user_mfa_credential = TenantModelAdmin::<UserMFACredentialCreate, _, _, _, _>::create(
        state.repo.as_ref(),
        &tenant,
        UserMFACredentialCreate {
            user_id: UserId::new(completed_auth_state.user.id.clone()),
            credential_type: haste_repository::types::mfa::MFACredentialType::TOTP,
            secret_ciphertext: aes_encrypted_secret.ciphertext,
            secret_nonce: aes_encrypted_secret.nonce,
            key_id: aes_key_id.clone(),
            totp_algorithm: Some(totp_rs::Algorithm::SHA1.to_string()),
            totp_digits: Some(6 as i16),
            totp_period: Some(30 as i16),
            totp_skew: Some(1 as i16),
        },
    )
    .await?;

    let authorization_redirect =
        Redirect::to(&(redirect_to_mfa_activation(&uri, &user_mfa_credential, "/create")));

    Ok(authorization_redirect.into_response())
}
