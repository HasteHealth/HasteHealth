use crate::auth_n::oidc::hardcoded_clients::admin_app;
use crate::services::AppState;
use crate::ui::components::{banner, page_html};
use axum::response::{IntoResponse, Redirect, Response};
use axum::{Form, extract::State};
use haste_fhir_model::r4::generated::resources::Membership;
use haste_fhir_model::r4::generated::terminology::IssueType;
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_repository::Repository;
use haste_repository::admin::SystemAdmin;
use haste_repository::types::membership::SystemMemberSearchClauses;
use haste_repository::types::scope::UserId;
use haste_repository::types::user::UserSearchClauses;
use std::sync::Arc;

#[derive(serde::Deserialize, axum_extra::routing::TypedPath)]
#[typed_path("/login")]
pub struct LoginGet {}

pub async fn login_get<
    Repo: Repository + Send + Sync,
    Search: SearchEngine + Send + Sync,
    Terminology: FHIRTerminology + Send + Sync,
>(
    _: LoginGet,
    State(_app_state): State<Arc<AppState<Repo, Search, Terminology>>>,
) -> Result<Response, OperationOutcomeError> {
    todo!();
}

pub struct LoginForm {
    email: String,
}

pub async fn login_post<
    Repo: Repository + Send + Sync,
    Search: SearchEngine + Send + Sync,
    Terminology: FHIRTerminology + Send + Sync,
>(
    Form(form): Form<LoginForm>,
    State(app_state): State<Arc<AppState<Repo, Search, Terminology>>>,
) -> Result<Response, OperationOutcomeError> {
    let users_with_email = SystemAdmin::search(
        app_state.repo.as_ref(),
        &UserSearchClauses {
            email: Some(form.email.to_string()),
            method: None,
            role: None,
        },
    )
    .await?;

    if users_with_email.len() > 10 {
        return Err(OperationOutcomeError::error(
            IssueType::TooCostly(None),
            "Too many users with the same email. Go to your users project login or contact your administrator.".to_string(),
        ));
    }

    for user in users_with_email.iter() {
        let memberships = SystemAdmin::<Membership, SystemMemberSearchClauses>::search(
            app_state.repo.as_ref(),
            &SystemMemberSearchClauses {
                tenant: None,
                user_id: Some(UserId::new(user.id.clone())),
            },
        )
        .await?;
    }

    todo!();
}
