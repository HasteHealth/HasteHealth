use crate::{
    auth_n::{
        oidc::{extract::client_app::OIDCClientApplication, ui::login::login_form_html},
        session,
    },
    extract::path_tenant::{Project, TenantIdentifier},
    services::AppState,
};
use axum::{
    Form,
    extract::{OriginalUri, State},
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::routing::TypedPath;
use maud::Markup;
use oxidized_fhir_operation_error::OperationOutcomeError;
use oxidized_fhir_search::SearchEngine;
use oxidized_fhir_terminology::FHIRTerminology;
use oxidized_repository::{
    Repository,
    types::user::{LoginMethod, LoginResult},
};
use serde::Deserialize;
use std::sync::Arc;
use tower_sessions::Session;

#[derive(TypedPath)]
#[typed_path("/login")]
pub struct Login;

pub async fn login_get(
    _: Login,
    TenantIdentifier { tenant }: TenantIdentifier,
    Project(project): Project,
    OIDCClientApplication(client_app): OIDCClientApplication,
    uri: OriginalUri,
) -> Result<Markup, OperationOutcomeError> {
    let response = login_form_html(&tenant, &project, &client_app, &uri.to_string());

    Ok(response)
}

#[derive(Deserialize)]
pub struct LoginForm {
    pub email: String,
    pub password: String,
}

pub async fn login_post<
    Repo: Repository + Send + Sync,
    Search: SearchEngine + Send + Sync,
    Terminology: FHIRTerminology + Send + Sync,
>(
    _: Login,
    TenantIdentifier { tenant }: TenantIdentifier,
    Project(project): Project,
    uri: OriginalUri,
    State(state): State<Arc<AppState<Repo, Search, Terminology>>>,
    current_session: Session,
    OIDCClientApplication(client_app): OIDCClientApplication,
    Form(login_data): Form<LoginForm>,
) -> Result<Response, OperationOutcomeError> {
    let login_result = state
        .repo
        .login(
            &tenant,
            &LoginMethod::EmailPassword {
                email: login_data.email,
                password: login_data.password,
            },
        )
        .await?;

    match login_result {
        LoginResult::Success { user } => {
            session::user::set_user(&current_session, &tenant, &user).await?;
            let authorization_redirect = Redirect::to(
                &(uri
                    .path()
                    .to_string()
                    .replace("/interactions/login", "/auth/authorize")
                    + "?"
                    + uri.query().unwrap_or("")),
            );

            Ok(authorization_redirect.into_response())
        }
        LoginResult::Failure => {
            Ok(login_form_html(&tenant, &project, &client_app, &uri.to_string()).into_response())
        }
    }
}
