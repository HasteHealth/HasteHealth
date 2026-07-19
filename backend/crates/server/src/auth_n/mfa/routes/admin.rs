use axum::{
    extract::{OriginalUri, State},
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
    types::{
        mfa::{UserMFACredentialCreate, UserMFASearchClaims},
        scope::UserId,
    },
};
use serde::Deserialize;
use std::sync::Arc;
use tower_sessions::Session;

use crate::{
    auth_n::session,
    extract::{csrf_token::CSRFToken, path_tenant::TenantIdentifier},
    services::ServerState,
    ui::pages::mfa,
};

fn create_mfa_route(uri: &OriginalUri, replace_path: &str) -> String {
    uri.path().to_string().replace(replace_path, "/create")
}

fn delete_mfa_route(uri: &OriginalUri, replace_path: &str) -> String {
    uri.path().to_string().replace(replace_path, "/delete")
}

fn activate_mfa_route(uri: &OriginalUri, replace_path: &str) -> String {
    uri.path().to_string().replace(replace_path, "/activate")
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/admin")]
pub struct MFAAdminGET;

// Creates User MFA credentials if user is allowed further MFA credentials.
// If user has reached the maximum allowed MFA credentials, an error is returned.
pub async fn admin_get<
    Repo: Repository + Send + Sync,
    Search: SearchEngine + Send + Sync,
    Terminology: FHIRTerminology + Send + Sync,
>(
    _: MFAAdminGET,
    uri: OriginalUri,
    CSRFToken(csrf_token): CSRFToken,
    Cached(TenantIdentifier { tenant }): Cached<TenantIdentifier>,
    State(state): State<Arc<ServerState<Repo, Search, Terminology>>>,
    Cached(current_session): Cached<Session>,
) -> Result<Response, OperationOutcomeError> {
    let get_auth_state = session::user::get_completed_authorization_state(&current_session)
        .await
        .map_err(|_e| {
            OperationOutcomeError::error(IssueType::SECURITY, "User is not logged in.".to_string())
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

    let mfa_admin_html = mfa::admin::mfa_admin_html(
        &tenant,
        &csrf_token,
        &existing_mfa_credentials,
        &create_mfa_route(&uri, "/admin"),
        &delete_mfa_route(&uri, "/admin"),
        &activate_mfa_route(&uri, "/admin"),
    );

    Ok(mfa_admin_html.into_response())
}
