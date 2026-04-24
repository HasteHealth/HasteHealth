use crate::auth_n::oidc::routes::route_string::tenant_route_string;
use crate::services::AppState;
use crate::ui::components::{banner, page_html};
use axum::response::{IntoResponse, Redirect, Response};
use axum::{Form, extract::State};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_jwt::TenantId;
use haste_repository::Repository;
use maud::html;
use std::sync::Arc;

#[derive(serde::Deserialize, axum_extra::routing::TypedPath)]
#[typed_path("/tenant-select")]
pub struct TenantSelectGet {}

pub async fn tenant_select_get<
    Repo: Repository + Send + Sync,
    Search: SearchEngine + Send + Sync,
    Terminology: FHIRTerminology + Send + Sync,
>(
    _: TenantSelectGet,
    State(_app_state): State<Arc<AppState<Repo, Search, Terminology>>>,
) -> Result<Response, OperationOutcomeError> {
    let signup_url = "/auth/signup";
    let action_url = "/auth/tenant-select";

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

#[derive(serde::Deserialize)]
pub struct TenantSelectForm {
    pub tenant: String,
}

#[derive(serde::Deserialize, axum_extra::routing::TypedPath)]
#[typed_path("/tenant-select")]
pub struct TenantSelectPost {}

pub async fn tenant_select_post(
    _: TenantSelectPost,
    Form(form): Form<TenantSelectForm>,
) -> Result<Response, OperationOutcomeError> {
    let tenant_id = TenantId::new(form.tenant);
    let project_select_route =
        tenant_route_string(&tenant_id).join("./auth/interactions/project-select");

    if let Some(project_select_route) = project_select_route.to_str() {
        Ok(Redirect::to(&project_select_route).into_response())
    } else {
        tracing::error!(
            "Failed to get admin app redirect URL for tenant '{}'",
            tenant_id.as_ref(),
        );
        Err(OperationOutcomeError::error(
            haste_fhir_model::r4::generated::terminology::IssueType::Exception(None),
            "Failed to determine admin app URL for tenant".to_string(),
        ))
    }
}
