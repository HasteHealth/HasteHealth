use crate::{
    auth_n::{mfa::utilities::user_mfa_to_totp, session},
    extract::{csrf_token::CSRFToken, path_tenant::TenantIdentifier},
    services::ServerState,
    ui::pages::{message::message_html, mfa},
};
use axum::{
    Form,
    extract::{OriginalUri, State},
    response::{IntoResponse, Response},
};
use axum_extra::{extract::Cached, routing::TypedPath};
use haste_fhir_model::r4::generated::terminology::IssueType;
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_jwt::TenantId;
use haste_repository::{
    Repository,
    admin::TenantModelAdmin,
    types::{
        mfa::{MFAKey, UserMFACredential, UserMFACredentialCreate, UserMFACredentialUpdate},
        scope::UserId,
        user::User,
    },
};
use maud::{PreEscaped, html};
use serde::Deserialize;
use sqlx::types::time::OffsetDateTime;
use std::sync::Arc;
use tower_sessions::Session;

pub fn mfa_activation_post_route(
    uri: &OriginalUri,
    user_mfa: &UserMFACredential,
    replace_path: &str,
) -> String {
    uri.path().to_string().replace(
        replace_path,
        &format!("/activate/{}", user_mfa.id.as_deref().unwrap_or("")),
    )
}

#[derive(Deserialize)]
pub struct MFAActivatePOSTBody {
    pub csrf_token: String,
    pub otp_code: String,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/activate/{id}")]
pub struct MFAActivatePOST {
    id: String,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/activate/{id}")]
pub struct MFAActivateGET {
    id: String,
}

pub async fn activate_html<
    Repo: Repository + Send + Sync,
    Search: SearchEngine + Send + Sync,
    Terminology: FHIRTerminology + Send + Sync,
>(
    state: &ServerState<Repo, Search, Terminology>,
    tenant: &TenantId,
    user: &User,
    user_mfa_credential: &UserMFACredential,
    csrf_token: &str,
    mfa_activation_post_route: &str,
) -> Result<PreEscaped<String>, OperationOutcomeError> {
    let totp = user_mfa_to_totp(
        state.secret_provider.as_ref(),
        &state.config,
        user,
        user_mfa_credential.clone(),
    )
    .await?;

    let qr_code = totp.get_qr_base64().map_err(|e| {
        tracing::error!(error = ?e);

        OperationOutcomeError::error(
            IssueType::Exception(None),
            "Could not generate QR code for MFA".to_string(),
        )
    })?;

    let qr_code_image = format!("data:image/png;base64,{}", qr_code);

    let mfa_active_html = mfa::activate::mfa_activate_html(
        &tenant,
        &csrf_token,
        user_mfa_credential
            .id
            .as_ref()
            .ok_or(OperationOutcomeError::error(
                IssueType::Exception(None),
                "User MFA credential ID is missing.".to_string(),
            ))?,
        &qr_code_image,
        user_mfa_credential.totp_digits as usize,
        mfa_activation_post_route,
        None,
    );

    Ok(mfa_active_html)
}

pub async fn activate_get<
    Repo: Repository + Send + Sync,
    Search: SearchEngine + Send + Sync,
    Terminology: FHIRTerminology + Send + Sync,
>(
    MFAActivateGET { id }: MFAActivateGET,
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
                IssueType::Security(None),
                "User is not logged in.".to_string(),
            )
        })?;

    let Some(user_mfa_credential) = TenantModelAdmin::<UserMFACredentialCreate, _, _, _, _>::read(
        state.repo.as_ref(),
        &tenant,
        &MFAKey::new(UserId::new(get_auth_state.user.id.clone()), id),
    )
    .await?
    else {
        return Err(OperationOutcomeError::error(
            IssueType::NotFound(None),
            "User MFA credential not found.".to_string(),
        ));
    };

    if user_mfa_credential.is_active {
        return Err(OperationOutcomeError::error(
            IssueType::Security(None),
            "MFA credential is already active.".to_string(),
        ));
    }

    let mfa_active_html = activate_html(
        state.as_ref(),
        &tenant,
        &get_auth_state.user,
        &user_mfa_credential,
        &csrf_token,
        uri.path(),
    )
    .await?;

    Ok(mfa_active_html.into_response())
}

pub async fn activate_post<
    Repo: Repository + Send + Sync,
    Search: SearchEngine + Send + Sync,
    Terminology: FHIRTerminology + Send + Sync,
>(
    MFAActivatePOST { id }: MFAActivatePOST,
    _uri: OriginalUri,
    CSRFToken(csrf_token): CSRFToken,
    Cached(TenantIdentifier { tenant }): Cached<TenantIdentifier>,
    State(state): State<Arc<ServerState<Repo, Search, Terminology>>>,
    Cached(current_session): Cached<Session>,
    Form(mfa_activate_data): Form<MFAActivatePOSTBody>,
) -> Result<Response, OperationOutcomeError> {
    if mfa_activate_data.csrf_token != csrf_token {
        return Err(OperationOutcomeError::error(
            IssueType::Security(None),
            "Invalid CSRF token.".to_string(),
        ));
    }

    let get_auth_state = session::user::get_completed_authorization_state(&current_session)
        .await
        .map_err(|_e| {
            OperationOutcomeError::error(
                IssueType::Security(None),
                "User is not logged in.".to_string(),
            )
        })?;

    let Some(user_mfa_credential) = TenantModelAdmin::<UserMFACredentialCreate, _, _, _, _>::read(
        state.repo.as_ref(),
        &tenant,
        &MFAKey::new(UserId::new(get_auth_state.user.id.clone()), id),
    )
    .await?
    else {
        return Err(OperationOutcomeError::error(
            IssueType::NotFound(None),
            "User MFA credential not found.".to_string(),
        ));
    };

    let totp = user_mfa_to_totp(
        state.secret_provider.as_ref(),
        &state.config,
        &get_auth_state.user,
        user_mfa_credential.clone(),
    )
    .await?;

    let is_otp_valid = totp
        .check_current(&mfa_activate_data.otp_code)
        .map_err(|_e| {
            OperationOutcomeError::error(IssueType::Security(None), "Invalid OTP code.".to_string())
        })?;

    if !is_otp_valid {
        return Err(OperationOutcomeError::error(
            IssueType::Security(None),
            "Invalid OTP code.".to_string(),
        ));
    }

    TenantModelAdmin::<UserMFACredentialCreate, _, _, _, _>::update(
        state.repo.as_ref(),
        &tenant,
        UserMFACredentialUpdate {
            id: user_mfa_credential.id.ok_or(OperationOutcomeError::error(
                IssueType::Exception(None),
                "User MFA credential ID is missing.".to_string(),
            ))?,
            user_id: UserId::new(get_auth_state.user.id),
            activated_at: Some(OffsetDateTime::now_utc()),
            is_active: true,
        },
    )
    .await?;

    Ok(message_html(
        Some(&tenant),
        None,
        html! { span {
                "MFA has been activated successfully. You can close this window."
            }
        },
    )
    .into_response())
}
