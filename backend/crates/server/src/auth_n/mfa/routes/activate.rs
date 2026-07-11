use axum::{
    extract::State,
    response::{IntoResponse as _, Response},
};
use axum_extra::{extract::Cached, routing::TypedPath};
use haste_fhir_model::r4::generated::terminology::IssueType;
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_repository::{
    Repository,
    admin::TenantModelAdmin,
    types::mfa::{UserMFACredential, UserMFACredentialCreate},
};
use maud::html;
use serde::Deserialize;
use std::sync::Arc;
use tower_sessions::Session;

use crate::{auth_n::session, extract::path_tenant::TenantIdentifier, services::ServerState};

#[derive(TypedPath, Deserialize)]
#[typed_path("/activate/{id}")]
pub struct MFAActivateGET {
    pub id: String,
}

// Sends a QR code image for a MFA registration request. The QR code is generated based on the user's email and a secret key,
// which is used for TOTP (Time-based One-Time Password) authentication.
// To activate the MR user most enter code which flips the db table to active.
pub async fn activate_get<
    Repo: Repository + Send + Sync,
    Search: SearchEngine + Send + Sync,
    Terminology: FHIRTerminology + Send + Sync,
>(
    MFAActivateGET { id }: MFAActivateGET,
    Cached(TenantIdentifier { tenant }): Cached<TenantIdentifier>,
    State(state): State<Arc<ServerState<Repo, Search, Terminology>>>,
    Cached(current_session): Cached<Session>,
) -> Result<Response, OperationOutcomeError> {
    let Some(_user) = session::user::get_user(&current_session)
        .await
        .map_err(|_e| {
            OperationOutcomeError::fatal(
                IssueType::Exception(None),
                "Session returned an error when retrieving current user.".to_string(),
            )
        })?
    else {
        return Err(OperationOutcomeError::error(
            IssueType::Security(None),
            "User is not logged in.".to_string(),
        ));
    };

    let Some(_user_mfa_credential) =
        TenantModelAdmin::<UserMFACredentialCreate, UserMFACredential, _, _, _>::read(
            state.repo.as_ref(),
            &tenant,
            &id,
        )
        .await?
    else {
        return Err(OperationOutcomeError::error(
            IssueType::NotFound(None),
            "MFA registration credential not found".to_string(),
        ));
    };

    let secret = totp_rs::Secret::default().to_bytes().map_err(|e| {
        tracing::error!(error = ?e);

        OperationOutcomeError::error(
            IssueType::Exception(None),
            "Could not generate secret for MFA".to_string(),
        )
    })?;

    let totp = totp_rs::TOTP::new(
        totp_rs::Algorithm::SHA1,
        6,
        1,
        30,
        secret,
        Some("Haste Health".to_string()),
        "haste-health@github.com".to_string(),
    )
    .map_err(|e| {
        tracing::error!(error = ?e);

        OperationOutcomeError::error(
            IssueType::Exception(None),
            "Could not generate TOTP for MFA".to_string(),
        )
    })?;

    let qr_code = totp.get_qr_base64().map_err(|e| {
        tracing::error!(error = ?e);

        OperationOutcomeError::error(
            IssueType::Exception(None),
            "Could not generate QR code for MFA".to_string(),
        )
    })?;

    Ok(html! {
        img src=(format!("data:image/png;base64,{}", qr_code)) {}
    }
    .into_response())
}

#[derive(TypedPath)]
#[typed_path("/activate")]
pub struct MFAActivatePOST;

pub async fn activate_post(_: MFAActivatePOST) -> Result<Response, OperationOutcomeError> {
    Ok((axum::http::StatusCode::OK, "MFA activation successful").into_response())
}
