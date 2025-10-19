use crate::{
    auth_n::{
        oidc::{
            extract::{client_app::OIDCClientApplication, scopes::Scopes},
            middleware::OIDCParameters,
            utilities::is_valid_redirect_url,
        },
        session,
    },
    extract::path_tenant::{Project, Tenant},
    services::AppState,
};
use axum::{
    Extension,
    extract::State,
    http::{Uri, uri::Scheme},
    response::Redirect,
};
use axum_extra::routing::TypedPath;
use maud::{Markup, html};
use oxidized_fhir_model::r4::generated::{resources::ClientApplication, terminology::IssueType};
use oxidized_fhir_operation_error::OperationOutcomeError;
use oxidized_fhir_search::SearchEngine;
use oxidized_fhir_terminology::FHIRTerminology;
use oxidized_repository::{
    Repository,
    admin::ProjectAuthAdmin,
    types::{
        authorization_code::{
            AuthorizationCodeKind, CreateAuthorizationCode, PKCECodeChallengeMethod,
        },
        membership::MembershipSearchClaims,
        user::UserRole,
    },
};
use std::{sync::Arc, time::Duration};
use tower_sessions::Session;

#[allow(unused)]
fn scopes_html_form(client_application: ClientApplication, scopes: Scopes) -> Markup {
    html! {
             head {
                meta charset="utf-8" {}
                meta name="viewport" content="width=device-width, initial-scale=1" {}
                link rel="preload" as="image" href="/img/logo.svg" {}
                title { "Oxidized Health" }
                link rel="icon" href="/img/logo.svg" {}
                link rel="stylesheet" href="/css/app.css" {}
            }
            body {
                section class="bg-gray-50 dark:bg-gray-900 h-screen" {
                    div class="flex flex-col items-center justify-center px-6 py-8 mx-auto md:h-screen lg:py-0" {
                        a href="#" class="flex items-center mb-6 text-2xl font-semibold text-gray-900 dark:text-white" {
                            img class="w-8 h-8 mr-2" src="/public/img/logo.svg" alt="logo" {
                                "IGUHealth"
                            }
                        }
                        div class="w-full bg-white rounded-lg shadow dark:border md:mt-0  xl:p-0 dark:bg-gray-800 dark:border-gray-700 sm:max-w-md" {
                            div class="p-6 space-y-4 md:space-y-6 sm:p-8" {
                                div {
                                    div class="flex flex-col justify-center items-center mb-6 text-2xl font-semibold text-gray-900 dark:text-white space-y-2" {
                                        div {
                                            div class="flex  justify-center items-center w-12 h-12 rounded-full bg-green-100 text-green-800 dark:bg-green-800 dark:text-green-100" {
                                                div {"A"}
                                            }
                                        }
                                        div {"Admin Application"}
                                    }
                                }
                                div {
                                    span class="text-sm text-gray-500" {
                                        "The above app is requesting the following permissions. Please review and either consent or deny access for the app."
                                    }
                                div class="max-h-72 overflow-auto" {
                                    table class="border-collapse  list-inside list-disc w-full" {
                                        tbody {
                                            tr class="border"{
                                                td class="p-4" {
                                                    "openid"
                                                }
                                                td {
                                                    div class="items-center justify-center flex"{
                                                        svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor" aria-hidden="true" data-slot="icon" class="w-6 h-6 text-gray-300"{
                                                            path fill-rule="evenodd" d="M18 10a8 8 0 1 1-16 0 8 8 0 0 1 16 0Zm-8-5a.75.75 0 0 1 .75.75v4.5a.75.75 0 0 1-1.5 0v-4.5A.75.75 0 0 1 10 5Zm0 10a1 1 0 1 0 0-2 1 1 0 0 0 0 2Z" clip-rule="evenodd" { }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                        }
                    }
                }
            }
        }
    }
}

#[derive(TypedPath)]
#[typed_path("/authorize")]
pub struct Authorize;

pub async fn authorize<
    Repo: Repository + Send + Sync,
    Search: SearchEngine + Send + Sync,
    Terminology: FHIRTerminology + Send + Sync,
>(
    _: Authorize,
    Scopes(_scopes): Scopes,
    Tenant { tenant }: Tenant,
    Project { project }: Project,
    State(app_state): State<Arc<AppState<Repo, Search, Terminology>>>,
    OIDCClientApplication(client_app): OIDCClientApplication,
    Extension(oidc_params): Extension<OIDCParameters>,
    current_session: Session,
) -> Result<Redirect, OperationOutcomeError> {
    let user = session::user::get_user(&current_session).await?.unwrap();
    // Verify the user has access to the given project.

    match &user.role {
        UserRole::Owner | UserRole::Admin => Ok(()),
        UserRole::Member => {
            // Check that user is a member of the tenant.
            let membership = ProjectAuthAdmin::search(
                &*app_state.repo,
                &tenant,
                &project,
                &MembershipSearchClaims {
                    user_id: Some(user.id.clone()),
                    role: None,
                },
            )
            .await?;

            if membership.is_empty() {
                session::user::clear_user(&current_session).await?;
                Err(OperationOutcomeError::error(
                    IssueType::Forbidden(None),
                    "User is not a member of the project.".to_string(),
                ))
            } else {
                Ok(())
            }
        }
    }?;

    let state = oidc_params.parameters.get("state").ok_or_else(|| {
        OperationOutcomeError::error(
            IssueType::Invalid(None),
            "state parameter is required.".to_string(),
        )
    })?;

    let redirect_uri = oidc_params.parameters.get("redirect_uri").ok_or_else(|| {
        OperationOutcomeError::error(
            IssueType::Invalid(None),
            "redirect_uri parameter is required.".to_string(),
        )
    })?;

    let Some(code_challenge) = oidc_params.parameters.get("code_challenge") else {
        return Err(OperationOutcomeError::error(
            IssueType::Invalid(None),
            "code_challenge parameter is required.".to_string(),
        ));
    };

    let Some(code_challenge_method) = oidc_params
        .parameters
        .get("code_challenge_method")
        .and_then(|code_challenge_method| {
            PKCECodeChallengeMethod::try_from(code_challenge_method.as_str()).ok()
        })
    else {
        return Err(OperationOutcomeError::error(
            IssueType::Invalid(None),
            "code_challenge_method must be a valid PKCE code challenge method.".to_string(),
        ));
    };

    if !is_valid_redirect_url(&redirect_uri, &client_app) {
        return Err(OperationOutcomeError::error(
            IssueType::Invalid(None),
            "Invalid redirect URI.".to_string(),
        ));
    }

    let client_id = client_app.id.ok_or_else(|| {
        OperationOutcomeError::error(
            IssueType::Invalid(None),
            "Client ID is required.".to_string(),
        )
    })?;

    let authorzation_code = ProjectAuthAdmin::create(
        &*app_state.repo,
        &tenant,
        &project,
        CreateAuthorizationCode {
            expires_in: Duration::from_secs(60 * 5),
            kind: AuthorizationCodeKind::OAuth2CodeGrant,
            user_id: user.id,
            client_id: Some(client_id.to_string()),
            pkce_code_challenge: Some(code_challenge.to_string()),
            pkce_code_challenge_method: Some(code_challenge_method),
            redirect_uri: Some(redirect_uri.to_string()),
            meta: None,
        },
    )
    .await?;

    let uri = Uri::try_from(redirect_uri).map_err(|_| {
        OperationOutcomeError::error(IssueType::Invalid(None), "Invalid redirect uri".to_string())
    })?;

    let redirection = Uri::builder()
        .scheme(uri.scheme().cloned().unwrap_or(Scheme::HTTPS))
        .authority(uri.authority().unwrap().clone())
        .path_and_query(
            uri.path().to_string() + "?code=" + &authorzation_code.code + "&state=" + state,
        )
        .build()
        .unwrap();

    Ok(Redirect::to(&redirection.to_string()))
}
