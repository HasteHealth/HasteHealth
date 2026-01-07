use crate::{
    auth_n::email::send_password_reset_email,
    services::AppState,
    tenants::{SubscriptionTier, create_tenant},
    ui::{
        components::{banner, page_html},
        pages::message::message_html,
    },
};
use axum::{Form, response::IntoResponse};
use axum::{extract::State, response::Response};
use axum_extra::routing::TypedPath;
use haste_fhir_model::r4::generated::{
    terminology,
    types::{FHIRString, HumanName},
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_jwt::ProjectId;
use haste_repository::{
    Repository,
    admin::SystemAdmin,
    types::user::{User, UserRole, UserSearchClauses},
};
use maud::html;
use std::sync::Arc;

#[derive(serde::Deserialize, TypedPath)]
#[typed_path("/signup")]
pub struct GlobalSignupGet {}

pub async fn global_signup_get<
    Repo: Repository + Send + Sync,
    Search: SearchEngine + Send + Sync,
    Terminology: FHIRTerminology + Send + Sync,
>(
    _: GlobalSignupGet,
    State(_app_state): State<Arc<AppState<Repo, Search, Terminology>>>,
) -> Result<Response, OperationOutcomeError> {
    Ok(page_html(html! {
        (banner("Sign Up", None))
        div class="w-full bg-white rounded-lg shadow  md:mt-0  xl:p-0  sm:max-w-md" {
            form class="space-y-4 md:space-y-6" action=("/auth/signup") method="POST" {
                div class="grid grid-cols-4 gap-1 space-y-4 p-6 sm:p-8" {                
                    div class="col-span-4" {
                        label for="email" class="block text-sm font-medium text-slate-600 dark:text-white" {
                            "Email address"
                        }
                        input type="email" id="email" class="bg-gray-50 border border-gray-300 text-slate-900 sm:text-sm rounded-lg focus:ring-blue-600 focus:border-blue-600 block w-full p-2.5" placeholder="name@company.com" required="" name="email" {}
                    }

                    div class="col-span-2" {
                        label for="first-name" class="block text-sm font-medium text-slate-600" { "First name" }
                        input id="first-name" class="bg-gray-50 border border-gray-300 text-slate-900 sm:text-sm rounded-lg focus:ring-orange-600 focus:border-orange-600 block w-full p-2.5 " required name="first-name" value="" {}
                    }

                    div class="col-span-2" {
                        label for="last-name" class="block text-sm font-medium text-slate-600" { "Last name" }
                        input id="last-name" class="bg-gray-50 border border-gray-300 text-slate-900 sm:text-sm rounded-lg focus:ring-orange-600 focus:border-orange-600 block w-full p-2.5 " required name="last-name" {}
                    }

                    div class="col-span-4" {
                        button type="submit" class="w-full text-white bg-orange-500 hover:bg-orange-500 focus:ring-4 focus:outline-none focus:ring-orange-300 font-medium rounded-lg text-sm px-5 py-2.5 text-center" {
                            "Continue"
                        }
                    }
                }
            }
        }
    }).into_response())
}

#[derive(serde::Deserialize)]
pub struct GlobalSignupForm {
    pub email: String,
    #[serde(rename = "first-name")]
    pub first_name: String,
    #[serde(rename = "last-name")]
    pub last_name: String,
}

async fn create_or_retrieve_user_tenant<
    Repo: Repository + Send + Sync,
    Search: SearchEngine + Send + Sync,
    Terminology: FHIRTerminology + Send + Sync,
>(
    app_state: &AppState<Repo, Search, Terminology>,
    signup_form: &GlobalSignupForm,
) -> Result<User, OperationOutcomeError> {
    let mut result = SystemAdmin::search(
        app_state.repo.as_ref(),
        &UserSearchClauses {
            email: Some(signup_form.email.to_string()),
            role: Some(UserRole::Owner),
            method: None,
        },
    )
    .await?;

    if let Some(user) = result.pop() {
        return Ok(user);
    } else {
        let result = create_tenant(
            app_state,
            None,
            "default",
            &SubscriptionTier::Free,
            haste_fhir_model::r4::generated::resources::User {
                role: Box::new(terminology::UserRole::Owner(None)),
                email: Some(Box::new(FHIRString {
                    value: Some(signup_form.email.to_string()),
                    ..Default::default()
                })),
                name: Some(Box::new(HumanName {
                    given: Some(vec![Box::new(FHIRString {
                        value: Some(signup_form.first_name.to_string()),
                        ..Default::default()
                    })]),
                    family: Some(Box::new(FHIRString {
                        value: Some(signup_form.last_name.to_string()),
                        ..Default::default()
                    })),
                    ..Default::default()
                })),
                ..Default::default()
            },
            None,
        )
        .await?;

        Ok(result.owner)
    }
}

#[derive(serde::Deserialize, axum_extra::routing::TypedPath)]
#[typed_path("/signup")]
pub struct GlobalSignupPost {}

pub async fn global_signup_post<
    Repo: Repository + Send + Sync,
    Search: SearchEngine + Send + Sync,
    Terminology: FHIRTerminology + Send + Sync,
>(
    _: GlobalSignupPost,
    State(app_state): State<Arc<AppState<Repo, Search, Terminology>>>,
    Form(form): Form<GlobalSignupForm>,
) -> Result<Response, OperationOutcomeError> {
    let user = create_or_retrieve_user_tenant(app_state.as_ref(), &form).await?;

    send_password_reset_email(
        app_state.as_ref(),
        &user.tenant,
        &ProjectId::System,
        &user,
        Some(html! {
            div {
                span {
                    "To set your password and complete your signup, please click the button below. If you did not request this email, please ignore it."
                }
            }
        }),
    )
    .await?;

    Ok(message_html(
None,
        None,
        html! {
            div {
                span {
                    "Welcome to Haste Health"
                }
            }
            div {
                span {
                    r#"An email has been sent to your email address "# 
                    span class="underline text-orange-600" { (user.email.unwrap_or("unknown".to_string())) } 
                    r#" to reset your password"#
                }
            }
        }
    ).into_response())
}
