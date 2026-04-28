use crate::{
    ServerEnvironmentVariables,
    auth_n::email::send_email,
    services::AppState,
    ui::{
        components::{banner, page_html},
        pages::message::message_html,
    },
};
use axum::{
    Form,
    extract::State,
    http::Uri,
    response::{IntoResponse as _, Response},
};
use axum_extra::routing::TypedPath;

use email_address::EmailAddress;
use haste_fhir_model::r4::generated::terminology::IssueType;
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_repository::{
    Repository,
    admin::SystemAdmin,
    types::user::{User, UserSearchClauses},
};
use maud::html;
use serde::Deserialize;
use std::sync::Arc;

#[derive(TypedPath)]
#[typed_path("/login")]
pub struct EmailSelect;

pub async fn global_login_get(_: EmailSelect) -> Result<Response, OperationOutcomeError> {
    let global_login_post_uri = "/auth/login";
    let signup_url = "/auth/signup";

    Ok(page_html(html! {
            (banner("Login", None))
            div class="w-full bg-white rounded-lg shadow  md:mt-0  xl:p-0" {
                form class="space-y-4 md:space-y-6" action=(global_login_post_uri) method="POST" {
                    div class="p-6 space-y-2 sm:p-8" {
                        div {
                            label for="email" class="block mb-2 text-sm font-medium text-slate-600 dark:text-white" {
                                "Enter your email"
                            }
                            input type="email" id="email" class="bg-gray-50 border border-gray-300 text-slate-900 sm:text-sm rounded-lg focus:ring-blue-600 focus:border-blue-600 block w-full p-2.5" placeholder="name@company.com" required="" name="email" {}
                        }
                        button type="submit" class="w-full text-white bg-orange-500 hover:bg-orange-500 focus:ring-4 focus:outline-none focus:ring-orange-300 font-medium rounded-lg text-sm px-5 py-2.5 text-center" {
                            "Continue"
                        }
                        div class="mt-2 flex items-center justify-start" {
                            a href=(signup_url) class="text-sm font-medium text-orange-600 hover:underline " { "Sign up" }
                        }
                    }
                }
            }
        }).into_response())
}

#[derive(Deserialize)]
pub struct GlobalLoginForm {
    pub email: EmailAddress,
}

pub async fn global_login_post<
    Repo: Repository + Send + Sync,
    Search: SearchEngine + Send + Sync,
    Terminology: FHIRTerminology + Send + Sync,
>(
    _: EmailSelect,
    State(state): State<Arc<AppState<Repo, Search, Terminology>>>,
    Form(login_data): Form<GlobalLoginForm>,
) -> Result<Response, OperationOutcomeError> {
    let found_users = SystemAdmin::<User, UserSearchClauses>::search(
        state.repo.as_ref(),
        &UserSearchClauses {
            email: Some(login_data.email.to_string()),
            role: None,
            method: None,
        },
    )
    .await?;

    let api_url = state.config.get(ServerEnvironmentVariables::APIURI)?;

    let tenant_select_email = crate::ui::email::base::base(
        &Uri::try_from(api_url.as_str()).map_err(|_| {
            OperationOutcomeError::fatal(
                IssueType::Exception(None),
                "API Url is invalid".to_string(),
            )
        })?,
        html! {
            div style="padding-top: 24px;" {
                "We received a request to log in to your account. If you made this request, click one of the tenants we found associated with your email."
            }

            div style="font-weight: 600; padding-top: 12px;" {
                "If you did not make this request, you can ignore this email. Otherwise, please click one of the tenants below that is associated with your account to log in."
            }

            table style="padding-top: 24px; width:100%" {
                @for user in &found_users {
                    tr style="padding-bottom: 12px;" {
                        td style="padding:12px; border: 1px solid #e5e7eb;" {
                            a href=(format!("{}/w/{}/auth/interactions/project-select", api_url, user.tenant)) {
                                "Tenant " (user.tenant)
                            }
                        }
                    }
                }
            }
        },
    );

    if !found_users.is_empty() {
        send_email(
            state.config.as_ref(),
            &login_data.email,
            "Haste Health Login",
            &tenant_select_email.into_string(),
        )
        .await?;
    }

    Ok(message_html(
        None,
        None,
        html! { div class="space-y-4 text-slate-900" {

                r#"An email has been sent to your email address "#
                span class="underline text-orange-600" { (&login_data.email) }
                r#"."#

                div {
                    span class="font-semibold" {
                        "If you don't receive an email shortly, please check your spam folder."
                    }
                }
            }
        },
    )
    .into_response())
}
