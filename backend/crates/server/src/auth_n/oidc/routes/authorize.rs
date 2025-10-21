use crate::{
    auth_n::{
        oidc::{
            extract::{client_app::OIDCClientApplication, scopes::Scopes},
            middleware::OIDCParameters,
            routes::{route_string::oidc_route_string, scope::ScopeForm},
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
    response::{IntoResponse, Redirect, Response},
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
        ProjectId, TenantId,
        authorization_code::{
            AuthorizationCodeKind, CreateAuthorizationCode, PKCECodeChallengeMethod,
        },
        membership::MembershipSearchClaims,
        scope::{ClientId, CreateScope, ScopeKey, UserId},
        user::UserRole,
    },
};
use std::{borrow::Cow, sync::Arc, time::Duration};
use tower_sessions::Session;

fn exclamation_point() -> Markup {
    html! {
        svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor" aria-hidden="true" data-slot="icon" class="w-6 h-6 text-gray-300" {
            path fill-rule="evenodd" d="M18 10a8 8 0 1 1-16 0 8 8 0 0 1 16 0Zm-8-5a.75.75 0 0 1 .75.75v4.5a.75.75 0 0 1-1.5 0v-4.5A.75.75 0 0 1 10 5Zm0 10a1 1 0 1 0 0-2 1 1 0 0 0 0 2Z" clip-rule="evenodd" {}
        }
    }
}

#[allow(unused)]
fn scopes_html_form(
    tenant: &TenantId,
    project: &ProjectId,
    client_application: &ClientApplication,
    authorization_info: &ScopeForm,
) -> Markup {
    let client_name = client_application
        .name
        .value
        .as_ref()
        .map(|s| Cow::Borrowed(s))
        .unwrap_or(Cow::Owned("Unnamed Client".to_string()));

    let scope_route = oidc_route_string(tenant, project, "auth/scope");
    let scope_route_str = scope_route.to_str().expect("Could not create scope route");

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
                section class="bg-gray-50  h-screen" {
                    div class="flex flex-col items-center justify-center px-6 py-8 mx-auto md:h-screen lg:py-0" {
                        a href="#" class="flex items-center mb-6 text-2xl font-semibold text-gray-900 " {
                            img class="w-8 h-8 mr-2" src="/img/logo.svg" alt="logo" {
                                "Oxidized Health"
                            }
                        }
                        div class="w-full bg-white rounded-lg shadow  md:mt-0  xl:p-0  sm:max-w-md" {
                            div class="p-6 space-y-4 md:space-y-6 sm:p-8" {
                                div class="flex flex-col justify-center items-center text-2xl font-semibold text-gray-900  space-y-2" {
                                    div {
                                        div class="flex  justify-center items-center w-12 h-12 rounded-full bg-green-100 text-green-800 " {
                                            div {(client_name.chars().next().unwrap_or('U').to_uppercase())}
                                        }
                                    }
                                    div {(client_name)}
                                }

                                div {
                                    span class="text-sm text-gray-500" {
                                        "The above app is requesting the following permissions. Please review and either consent or deny access for the app."
                                    }
                                }
                                div class="max-h-72 overflow-auto" {
                                    table class="border-collapse  list-inside list-disc w-full" {
                                        tbody {
                                            @for s in authorization_info.scope.0.iter() {
                                                tr class="border border-gray-200"{
                                                    td class="p-4" {
                                                        (String::from(s.clone()))
                                                    }
                                                    td {
                                                        div class="items-center justify-center flex" {
                                                            (exclamation_point())
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                div class="justify-center items-center flex space-x-4" {
                                    form action=(scope_route_str) method="POST" {
                                        input readonly="" class="hidden" type="text" name="client_id" value=(authorization_info.client_id) {}
                                        input readonly="" class="hidden" type="text" name="response_type" value=(authorization_info.response_type) {}
                                        input readonly="" class="hidden" type="text" name="state" value=(authorization_info.state)  {}
                                        input readonly="" class="hidden" type="text" name="code_challenge" value=(authorization_info.code_challenge)  {}
                                        input readonly="" class="hidden" type="text" name="code_challenge_method" value=(authorization_info.code_challenge_method) {}
                                        input readonly="" class="hidden" type="text" name="scope" value=(String::from(authorization_info.scope.clone())) {}
                                        input readonly="" class="hidden" type="text" name="redirect_uri" value=(authorization_info.redirect_uri) {}
                                        input readonly="" class="hidden" type="checkbox" name="accept" checked {}
                                        button type="submit" class="cursor-pointer w-full text-white bg-teal-600 hover:bg-teal-700 focus:ring-4 focus:outline-none focus:ring-teal-300 font-medium rounded-lg text-sm px-5 py-2.5 text-center dark:bg-teal-600 dark:hover:bg-teal-700 dark:focus:ring-teal-800" {
                                            "Allow"
                                        }
                                    }

                                    form action=(scope_route_str) method="POST" {
                                        input readonly="" class="hidden" type="text" name="client_id" value=(authorization_info.client_id) {}
                                        input readonly="" class="hidden" type="text" name="response_type" value=(authorization_info.response_type) {}
                                        input readonly="" class="hidden" type="text" name="state" value=(authorization_info.state)  {}
                                        input readonly="" class="hidden" type="text" name="code_challenge" value=(authorization_info.code_challenge)  {}
                                        input readonly="" class="hidden" type="text" name="code_challenge_method" value=(authorization_info.code_challenge_method) {}
                                        input readonly="" class="hidden" type="text" name="scope" value=(String::from(authorization_info.scope.clone())) {}
                                        input readonly="" class="hidden" type="text" name="redirect_uri" value=(authorization_info.redirect_uri) {}
                                        input readonly="" class="hidden" type="checkbox" name="accept" {}
                                        button type="submit" class="cursor-pointer w-full text-gray-900 bg-gray-100 hover:bg-gray-200 focus:ring-4 focus:outline-none  font-medium rounded-lg text-sm px-5 py-2.5 text-center dark:text-white dark:bg-gray-600 dark:hover:bg-gray-700 dark:focus:ring-gray-800" {
                                            "Deny"
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
    scopes: Scopes,
    Tenant { tenant }: Tenant,
    Project { project }: Project,
    State(app_state): State<Arc<AppState<Repo, Search, Terminology>>>,
    OIDCClientApplication(client_app): OIDCClientApplication,
    Extension(oidc_params): Extension<OIDCParameters>,
    current_session: Session,
) -> Result<Response, OperationOutcomeError> {
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

    let response_type = oidc_params.parameters.get("response_type").ok_or_else(|| {
        OperationOutcomeError::error(
            IssueType::Invalid(None),
            "response_type parameter is required.".to_string(),
        )
    })?;

    let redirect_uri = oidc_params.parameters.get("redirect_uri").ok_or_else(|| {
        OperationOutcomeError::error(
            IssueType::Invalid(None),
            "redirect_uri parameter is required.".to_string(),
        )
    })?;

    if !is_valid_redirect_url(&redirect_uri, &client_app) {
        return Err(OperationOutcomeError::error(
            IssueType::Invalid(None),
            "Invalid redirect URI.".to_string(),
        ));
    }

    let uri = Uri::try_from(redirect_uri).map_err(|_| {
        OperationOutcomeError::error(IssueType::Invalid(None), "Invalid redirect uri".to_string())
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

    let client_id = client_app.id.clone().ok_or_else(|| {
        OperationOutcomeError::error(
            IssueType::Invalid(None),
            "Client ID is required.".to_string(),
        )
    })?;

    let existing_scopes = ProjectAuthAdmin::<CreateScope, _, _, _, _>::read(
        &*app_state.repo,
        &tenant,
        &project,
        &ScopeKey::new(
            ClientId::new(client_id.to_string()),
            UserId::new(user.id.clone()),
        ),
    )
    .await?;

    let existing_scope_str = existing_scopes
        .as_ref()
        .map(|s| Cow::Borrowed(&s.scope))
        .unwrap_or_else(|| Cow::Owned("".to_string()));

    let existing_scopes = Scopes::try_from(existing_scope_str.as_str())?;

    if existing_scopes != scopes {
        return Ok(scopes_html_form(
            &tenant,
            &project,
            &client_app,
            &ScopeForm {
                client_id: client_app
                    .id
                    .as_ref()
                    .map(|s| s.to_string())
                    .unwrap_or("".to_string()),
                scope: scopes,
                response_type: response_type.clone(),
                state: state.clone(),
                code_challenge: code_challenge.to_string(),
                code_challenge_method: String::from(code_challenge_method),
                redirect_uri: redirect_uri.to_string(),
                accept: None,
            },
        )
        .into_response());
    }

    let authorization_code = ProjectAuthAdmin::create(
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

    let redirection = Uri::builder()
        .scheme(uri.scheme().cloned().unwrap_or(Scheme::HTTPS))
        .authority(uri.authority().unwrap().clone())
        .path_and_query(
            uri.path().to_string() + "?code=" + &authorization_code.code + "&state=" + state,
        )
        .build()
        .unwrap();

    let redirect = Redirect::to(&redirection.to_string());
    let response = redirect.into_response();
    Ok(response)
}
