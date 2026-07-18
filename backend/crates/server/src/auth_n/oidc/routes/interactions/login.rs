use crate::{
    auth_n::{
        mfa::routes::totp_verification::totp_verification_route,
        oidc::{
            extract::client_app::OIDCClientApplication, routes::authorize::redirect_authorize_uri,
        },
        session::{self, user::SessionAuthorizationState},
    },
    extract::{
        csrf_token::CSRFToken,
        path_tenant::{Project, TenantIdentifier},
    },
    fhir_client::ServerCTX,
    services::ServerState,
    ui::pages,
};
use axum::{
    Form,
    extract::{OriginalUri, State},
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::{extract::Cached, routing::TypedPath};
use haste_fhir_client::FHIRClient;
use haste_fhir_model::r4::generated::{
    resources::{Bundle, BundleEntry, BundleEntryRequest},
    terminology::{HttpVerb, IssueType},
    types::FHIRUri,
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_jwt::ProjectId;
use haste_repository::{
    Repository,
    types::user::{LoginMethod, LoginResult},
};
use maud::Markup;
use serde::Deserialize;
use std::sync::Arc;
use tower_sessions::Session;

#[derive(TypedPath)]
#[typed_path("/login")]
pub struct Login;

pub async fn login_get<
    Repo: Repository + Send + Sync,
    Search: SearchEngine + Send + Sync,
    Terminology: FHIRTerminology + Send + Sync,
>(
    _: Login,
    State(state): State<Arc<ServerState<Repo, Search, Terminology>>>,
    Cached(TenantIdentifier { tenant }): Cached<TenantIdentifier>,
    Cached(Project(project_resource)): Cached<Project>,
    OIDCClientApplication(client_app): OIDCClientApplication,
    CSRFToken(csrf_token): CSRFToken,
    uri: OriginalUri,
) -> Result<Markup, OperationOutcomeError> {
    let idps = resolve_identity_providers(&state, tenant.clone(), &project_resource).await?;
    let response = pages::login::login_form_html(
        &tenant,
        &project_resource,
        &csrf_token,
        idps.as_ref(),
        &client_app,
        &uri.to_string(),
        None,
    );

    Ok(response)
}

#[derive(Deserialize)]
pub struct LoginForm {
    pub csrf_token: String,
    pub email: String,
    pub password: String,
}

/// Resolves the IdentityProviders configured for the given Project resource.
/// Uses a batch request to fetch all IdentityProvider resources referenced by the Project.
async fn resolve_identity_providers<
    Repo: Repository + Send + Sync,
    Search: SearchEngine + Send + Sync,
    Terminology: FHIRTerminology + Send + Sync,
>(
    state: &Arc<ServerState<Repo, Search, Terminology>>,
    tenant: haste_jwt::TenantId,
    project_resource: &haste_fhir_model::r4::generated::resources::Project,
) -> Result<
    Option<Vec<haste_fhir_model::r4::generated::resources::IdentityProvider>>,
    OperationOutcomeError,
> {
    let identity_providers = if let Some(idps) = project_resource.identityProvider.as_ref() {
        let res = state
            .fhir_client
            .batch(
                Arc::new(ServerCTX::system(
                    tenant,
                    ProjectId::System,
                    state.fhir_client.clone(),
                    state.rate_limit.clone(),
                )),
                Bundle {
                    entry: Some(
                        idps.iter()
                            .filter_map(|idp| idp.reference.as_ref())
                            .filter_map(|idp_ref| idp_ref.value.as_ref())
                            .map(|idp_ref| BundleEntry {
                                request: Some(BundleEntryRequest {
                                    method: HttpVerb::GET,
                                    url: Box::new(FHIRUri {
                                        value: Some(idp_ref.to_string()),
                                        ..Default::default()
                                    }),
                                    ..Default::default()
                                }),
                                ..Default::default()
                            })
                            .collect::<Vec<_>>(),
                    ),
                    ..Default::default()
                },
            )
            .await?;

        Some(
            res.entry
                .unwrap_or_default()
                .into_iter()
                .filter_map(|entry| entry.resource)
                .filter_map(|res| match *res {
                    haste_fhir_model::r4::generated::resources::Resource::IdentityProvider(idp) => {
                        Some(idp)
                    }
                    _ => None,
                })
                .collect::<Vec<_>>(),
        )
    } else {
        None
    };

    Ok(identity_providers)
}

pub async fn login_post<
    Repo: Repository + Send + Sync,
    Search: SearchEngine + Send + Sync,
    Terminology: FHIRTerminology + Send + Sync,
>(
    _: Login,
    Cached(TenantIdentifier { tenant }): Cached<TenantIdentifier>,
    Cached(Project(project_resource)): Cached<Project>,
    uri: OriginalUri,
    State(state): State<Arc<ServerState<Repo, Search, Terminology>>>,
    Cached(current_session): Cached<Session>,
    OIDCClientApplication(client_app): OIDCClientApplication,
    CSRFToken(csrf_token): CSRFToken,
    Form(login_data): Form<LoginForm>,
) -> Result<Response, OperationOutcomeError> {
    if login_data.csrf_token != csrf_token {
        return Err(OperationOutcomeError::error(
            IssueType::INVALID,
            "Invalid CSRF Token".to_string(),
        ));
    }

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
            let authorization_redirect = redirect_authorize_uri(&uri, "/interactions/login");
            session::user::set_initial_authorization_state(
                state.repo.as_ref(),
                &current_session,
                user,
            )
            .await?;

            let redirect_target =
                match session::user::get_authorization_state(&current_session).await? {
                    Some(SessionAuthorizationState::MFARequired { .. }) => {
                        totp_verification_route(&tenant, &authorization_redirect)
                    }
                    _ => authorization_redirect,
                };
            let authorization_redirect = Redirect::to(&redirect_target);

            Ok(authorization_redirect.into_response())
        }
        LoginResult::Failure => {
            let idps =
                resolve_identity_providers(&state, tenant.clone(), &project_resource).await?;
            Ok(pages::login::login_form_html(
                &tenant,
                &project_resource,
                &csrf_token,
                idps.as_ref(),
                &client_app,
                &uri.to_string(),
                Some(vec!["Invalid credentials. Please try again.".to_string()]),
            )
            .into_response())
        }
    }
}
