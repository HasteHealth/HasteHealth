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
                div class="p-6 space-y-4 md:space-y-6 sm:p-8" {
                    div {
                        label for="email" class="block mb-2 text-sm font-medium text-slate-600 dark:text-white" {
                            "Enter your email"
                        }
                        input type="email" id="email" class="bg-gray-50 border border-gray-300 text-slate-900 sm:text-sm rounded-lg focus:ring-blue-600 focus:border-blue-600 block w-full p-2.5" placeholder="name@company.com" required="" name="email" {}
                    }
                    button type="submit" class="w-full text-white bg-orange-500 hover:bg-orange-500 focus:ring-4 focus:outline-none focus:ring-orange-300 font-medium rounded-lg text-sm px-5 py-2.5 text-center" {
                        "Continue"
                    }
                }
            }
        }
    }).into_response())
}

#[derive(serde::Deserialize)]
pub struct GlobalSignupForm {
    pub email: String,
}

async fn create_or_retrieve_user_tenant<
    Repo: Repository + Send + Sync,
    Search: SearchEngine + Send + Sync,
    Terminology: FHIRTerminology + Send + Sync,
>(
    app_state: &AppState<Repo, Search, Terminology>,
    email: &str,
) -> Result<User, OperationOutcomeError> {
    let mut result = SystemAdmin::search(
        app_state.repo.as_ref(),
        &UserSearchClauses {
            email: Some(email.to_string()),
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
            email,
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
    let user = create_or_retrieve_user_tenant(app_state.as_ref(), &form.email).await?;

    send_password_reset_email(app_state.as_ref(), &user.tenant, &ProjectId::System, &user).await?;

    Ok(message_html(
        &user.tenant,
        None,
        html! {"Your user has been created. An email has been sent to you with a link to set your password."},
    ).into_response())
}
