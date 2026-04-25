use crate::{
    extract::path_tenant::TenantIdentifier, services::AppState, ui::components::page_html,
};
use axum::{
    Form,
    extract::{OriginalUri, State},
    response::Response,
};
use axum_extra::{extract::Cached, routing::TypedPath};

use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_repository::Repository;
use maud::{Markup, html};
use serde::Deserialize;
use std::sync::Arc;
use tower_sessions::Session;

#[derive(TypedPath)]
#[typed_path("/login")]
pub struct EmailSelect;

pub async fn email_select_get<
    Repo: Repository + Send + Sync,
    Search: SearchEngine + Send + Sync,
    Terminology: FHIRTerminology + Send + Sync,
>(
    _: EmailSelect,
    State(state): State<Arc<AppState<Repo, Search, Terminology>>>,
    Cached(TenantIdentifier { tenant }): Cached<TenantIdentifier>,
    uri: OriginalUri,
) -> Result<Markup, OperationOutcomeError> {
    Ok(page_html(html! {
        (banner("Enter your tenant identifier", None))
        div class="w-full bg-white rounded-lg shadow md:mt-0 xl:p-0 w-md sm:max-w-md text-slate-700" {
            div class="p-6 space-y-4 md:space-y-6 sm:p-8" {
                form class="space-y-2" action=(action_url) method="POST" {
                    div class="grid grid-cols-4 gap-1" {
                        div class="col-span-4" {
                            label for="tenant" class="block text-sm font-medium text-slate-600" { "Tenant" }
                            input type="tenant" id="tenant" class="bg-gray-50 border border-gray-300 text-slate-900 sm:text-sm rounded-lg focus:ring-orange-600 focus:border-orange-600 block w-full p-2.5 " placeholder="Tenant id" required name="tenant" value="" {}
                        }
                    }

                    div class="space-y-4" {
                        button type="submit" class="w-full text-white bg-orange-500 hover:bg-orange-500 focus:ring-4 focus:outline-none focus:ring-orange-300 font-medium rounded-lg text-sm px-5 py-2.5 text-center " { "Continue" }
                        div class="flex items-center justify-start" {
                            a href=(signup_url) class="text-sm font-medium text-orange-600 hover:underline " { "Sign up" }
                        }
                    }
                }
            }
        }
    }).into_response())
}

#[derive(Deserialize)]
pub struct EmailSelectForm {
    pub email: String,
}

pub async fn email_select_post<
    Repo: Repository + Send + Sync,
    Search: SearchEngine + Send + Sync,
    Terminology: FHIRTerminology + Send + Sync,
>(
    _: EmailSelect,
    Cached(TenantIdentifier { tenant }): Cached<TenantIdentifier>,
    uri: OriginalUri,
    State(state): State<Arc<AppState<Repo, Search, Terminology>>>,
    Cached(current_session): Cached<Session>,
    Form(login_data): Form<EmailSelectForm>,
) -> Result<Response, OperationOutcomeError> {
    todo!();
}
