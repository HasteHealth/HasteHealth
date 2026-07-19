use crate::{
    auth_n::session,
    extract::{csrf_token::CSRFToken, path_tenant::TenantIdentifier},
    services::ServerState,
};
use axum::{
    Form,
    extract::{OriginalUri, State},
    response::{IntoResponse, Redirect, Response},
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
        mfa::{MFAKey, UserMFACredentialCreate},
        scope::UserId,
    },
};
use serde::Deserialize;
use std::sync::Arc;
use tower_sessions::Session;

#[derive(Deserialize)]
pub struct MFADeletePOSTBody {
    pub csrf_token: String,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/delete/{id}")]
pub struct MFADeletePOST {
    id: String,
}

pub fn replace_mfa_route(uri: &OriginalUri, user_mfa_id: &str, replace_path: &str) -> String {
    uri.path()
        .to_string()
        .replace(&format!("/delete/{}", user_mfa_id), replace_path)
}

// Keeping as POST so can have an CSRF token for security.
pub async fn mfa_delete_post<
    Repo: Repository + Send + Sync,
    Search: SearchEngine + Send + Sync,
    Terminology: FHIRTerminology + Send + Sync,
>(
    MFADeletePOST { id }: MFADeletePOST,
    uri: OriginalUri,
    CSRFToken(csrf_token): CSRFToken,
    Cached(TenantIdentifier { tenant }): Cached<TenantIdentifier>,
    State(state): State<Arc<ServerState<Repo, Search, Terminology>>>,
    Cached(current_session): Cached<Session>,
    Form(delete_body): Form<MFADeletePOSTBody>,
) -> Result<Response, OperationOutcomeError> {
    if csrf_token != delete_body.csrf_token {
        return Err(OperationOutcomeError::error(
            IssueType::SECURITY,
            "Invalid CSRF token.".to_string(),
        ));
    }

    let get_auth_state = session::user::get_completed_authorization_state(&current_session)
        .await
        .map_err(|_e| {
            OperationOutcomeError::error(IssueType::SECURITY, "User is not logged in.".to_string())
        })?;

    let redirect_to_path = replace_mfa_route(&uri, &id, "/admin");

    TenantModelAdmin::<UserMFACredentialCreate, _, _, _, _>::delete(
        state.repo.as_ref(),
        &tenant,
        &MFAKey::new(UserId::new(get_auth_state.user.id), id.clone()),
    )
    .await?;

    Ok(Redirect::to(&redirect_to_path).into_response())
}
