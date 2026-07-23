use axum::{
    extract::{OriginalUri, State},
    response::{IntoResponse as _, Response},
};
use axum_extra::{extract::Cached, routing::TypedPath};
use haste_encryption::Encryptor as _;
use haste_fhir_model::r4::generated::terminology::IssueType;
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_repository::{
    Repository,
    admin::TenantModelAdmin,
    types::{
        mfa::{UserMFACredentialCreate, UserMFASearchClaims},
        scope::UserId,
    },
};
use serde::Deserialize;
use std::sync::Arc;
use tower_sessions::Session;

use crate::{
    auth_n::{
        mfa::{
            routes::activate::{activate_html, mfa_activation_post_route},
            utilities::{get_aes_encryptor, get_aes_key},
        },
        session,
    },
    extract::{csrf_token::CSRFToken, path_tenant::TenantIdentifier},
    services::ServerState,
};

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
    CSRFToken(csrf_token): CSRFToken,
    Cached(TenantIdentifier { tenant }): Cached<TenantIdentifier>,
    State(state): State<Arc<ServerState<Repo, Search, Terminology>>>,
    Cached(current_session): Cached<Session>,
) -> Result<Response, OperationOutcomeError> {
    let get_auth_state = session::user::get_completed_authorization_state(&current_session)
        .await
        .map_err(|_e| {
            OperationOutcomeError::error(
                IssueType::security(),
                "User is not logged in.".to_string(),
            )
        })?;

    let existing_mfa_credentials = TenantModelAdmin::<UserMFACredentialCreate, _, _, _, _>::search(
        state.repo.as_ref(),
        &tenant,
        &UserMFASearchClaims {
            tenant: tenant.clone(),
            user_id: UserId::new(get_auth_state.user.id.clone()),
            is_active: None,
        },
    )
    .await?;

    // Todo make this amount configurable
    if existing_mfa_credentials.len() >= state.config.security.mfa.max_credentials_per_user {
        return Err(OperationOutcomeError::error(
            IssueType::security(),
            format!(
                "User has reached the maximum of {} MFA credentials allowed.",
                state.config.security.mfa.max_credentials_per_user
            ),
        ));
    }

    let secret = totp_rs::Secret::default().to_bytes().map_err(|e| {
        tracing::error!(error = ?e);

        OperationOutcomeError::error(
            IssueType::exception(),
            "Could not generate secret for MFA".to_string(),
        )
    })?;

    let aes_key_id = get_aes_key(&state.config)?;
    let aes_encryptor = get_aes_encryptor(state.secret_provider.as_ref(), &state.config).await?;
    let aes_encrypted_secret = aes_encryptor.encrypt(&secret)?;

    let user_mfa_credential = TenantModelAdmin::<UserMFACredentialCreate, _, _, _, _>::create(
        state.repo.as_ref(),
        &tenant,
        UserMFACredentialCreate {
            user_id: UserId::new(get_auth_state.user.id.clone()),
            credential_type: haste_repository::types::mfa::MFACredentialType::TOTP,
            secret_ciphertext: aes_encrypted_secret.ciphertext,
            secret_nonce: aes_encrypted_secret.nonce,
            key_id: aes_key_id.to_string(),
            totp_algorithm: Some(totp_rs::Algorithm::SHA1.to_string()),
            totp_digits: Some(6 as i16),
            totp_period: Some(30 as i16),
            totp_skew: Some(1 as i16),
        },
    )
    .await?;

    let mfa_active_html = activate_html(
        state.as_ref(),
        &tenant,
        &get_auth_state.user,
        &user_mfa_credential,
        &csrf_token,
        &mfa_activation_post_route(&uri, &user_mfa_credential, "/create"),
    )
    .await?;

    Ok(mfa_active_html.into_response())
}
