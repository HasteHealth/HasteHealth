use crate::{
    auth_n::session,
    extract::path_tenant::TenantIdentifier,
    services::AppState,
    ui::components::{banner, page_html},
};
use axum::{
    Form,
    extract::{OriginalUri, State},
    response::{IntoResponse as _, Redirect, Response},
};
use axum_extra::{extract::Cached, routing::TypedPath};

use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_repository::Repository;
use maud::html;
use serde::Deserialize;
use std::sync::Arc;
use tower_sessions::Session;

#[derive(TypedPath)]
#[typed_path("/login")]
pub struct EmailSelect;

pub async fn global_login_get<
    Repo: Repository + Send + Sync,
    Search: SearchEngine + Send + Sync,
    Terminology: FHIRTerminology + Send + Sync,
>(
    _: EmailSelect,

    Cached(current_session): Cached<Session>,
) -> Result<Response, OperationOutcomeError> {
    let global_login_post_uri = "/auth/login";
    if let Ok(Some(user)) = session::user::get_user(&current_session).await {
        // User is already authenticated, redirect to project select.
        let redirect_uri = format!("/w/{}/auth/project-select", user.tenant.as_ref());

        Ok(Redirect::to(&redirect_uri).into_response())
    } else {
        Ok(page_html(html! {
            (banner("Global Login", None))
            div class="w-full bg-white rounded-lg shadow  md:mt-0  xl:p-0  sm:max-w-md" {
                form class="space-y-4 md:space-y-6" action=(global_login_post_uri) method="POST" {
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
}

#[derive(Deserialize)]
pub struct GlobalLoginForm {
    pub email: String,
}

pub async fn global_login_post<
    Repo: Repository + Send + Sync,
    Search: SearchEngine + Send + Sync,
    Terminology: FHIRTerminology + Send + Sync,
>(
    _: EmailSelect,
    uri: OriginalUri,
    State(state): State<Arc<AppState<Repo, Search, Terminology>>>,
    Form(login_data): Form<GlobalLoginForm>,
) -> Result<Response, OperationOutcomeError> {
    todo!();
}
