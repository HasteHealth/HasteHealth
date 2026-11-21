use crate::auth_n::error::AuthError;
use crate::auth_n::oidc::routes::route_string::tenant_route_string;
use crate::extract::csrf_token::CSRFToken;
use crate::services::ServerState;
use crate::ui::{
    components::{banner, page_html},
    styles::{BUTTON, CARD, CARD_BODY, INPUT, LABEL, LINK},
};
use axum::response::{IntoResponse, Redirect, Response};
use axum::{Form, extract::State};
//use axum_client_ip::ClientIp;
use crate::auth_n::constant_time;
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
    CSRFToken(csrf_token): CSRFToken,
    //ClientIp(ip_address): ClientIp,
    State(state): State<Arc<ServerState<Repo, Search, Terminology>>>,
) -> Result<Response, OperationOutcomeError> {
    let signup_url = "/auth/signup";
    let action_url = "/auth/tenant-select";

    Ok(page_html(&html! {
        (banner("Enter your tenant identifier", None))
        div class=(CARD) {
            form class=(CARD_BODY) action=(action_url) method="POST" {
                input type="hidden" name="csrf_token" value=(csrf_token) {}
                div {
                    label for="tenant" class=(LABEL) { "Tenant" }
                    input type="text" id="tenant" class=(INPUT) placeholder="Tenant id" required name="tenant" value="" {}
                }
                button type="submit" class=(BUTTON) { "Continue" }
                @if state.config.signup.enabled {
                    div class="text-sm" {
                        a href=(signup_url) class=(LINK) { "Sign up" }
                    }
                }
            }
        }
    }).into_response())
}

#[derive(serde::Deserialize)]
pub struct TenantSelectForm {
    pub csrf_token: String,
    pub tenant: String,
}

#[derive(serde::Deserialize, axum_extra::routing::TypedPath)]
#[typed_path("/tenant-select")]
pub struct TenantSelectPost {}

pub async fn tenant_select_post(
    _: TenantSelectPost,
    CSRFToken(csrf_token): CSRFToken,
    Form(form): Form<TenantSelectForm>,
) -> Result<Response, OperationOutcomeError> {
    if !constant_time::str_eq(&form.csrf_token, &csrf_token) {
        return Err(AuthError::InvalidCsrfToken.into());
    }

    let tenant_id = TenantId::new(form.tenant)?;
    let project_select_route =
        tenant_route_string(&tenant_id).join("./auth/interactions/project-select");

    if let Some(project_select_route) = project_select_route.to_str() {
        Ok(Redirect::to(project_select_route).into_response())
    } else {
        tracing::error!(
            "Failed to get admin app redirect URL for tenant '{}'",
            tenant_id.as_ref(),
        );
        Err(OperationOutcomeError::error(
            haste_fhir_model::r4::generated::terminology::IssueType::exception(),
            "Failed to determine admin app URL for tenant".to_string(),
        ))
    }
}
