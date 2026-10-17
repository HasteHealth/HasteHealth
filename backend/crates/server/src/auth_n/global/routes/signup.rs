//! `/auth/signup`: one email field. The code it sends is checked at
//! `/auth/verify`. Closed when `signup.enabled` is off.

use crate::{
    auth_n::global::{
        email_code::Purpose,
        routes::flow::{self, EmailForm, client_ip},
    },
    extract::csrf_token::CSRFToken,
    services::ServerState,
    ui::pages::global_auth::signup_closed_html,
};
use axum::{
    Form,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use axum_client_ip::ClientIp;
use axum_extra::{extract::Cached, routing::TypedPath};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_repository::Repository;
use std::sync::Arc;
use tower_sessions::Session;

#[derive(serde::Deserialize, TypedPath)]
#[typed_path("/signup")]
pub struct GlobalSignup;

fn signup_closed() -> Response {
    (StatusCode::FORBIDDEN, signup_closed_html()).into_response()
}

pub async fn global_signup_get<
    Repo: Repository + Send + Sync,
    Search: SearchEngine + Send + Sync,
    Terminology: FHIRTerminology + Send + Sync,
>(
    _: GlobalSignup,
    CSRFToken(csrf_token): CSRFToken,
    State(state): State<Arc<ServerState<Repo, Search, Terminology>>>,
) -> Result<Response, OperationOutcomeError> {
    if !state.config.signup.enabled {
        return Ok(signup_closed());
    }

    Ok(flow::email_form(state.as_ref(), Purpose::Signup, &csrf_token, None).into_response())
}

pub async fn global_signup_post<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
>(
    _: GlobalSignup,
    CSRFToken(csrf_token): CSRFToken,
    State(state): State<Arc<ServerState<Repo, Search, Terminology>>>,
    Cached(session): Cached<Session>,
    ip: Result<ClientIp, axum_client_ip::Rejection>,
    Form(form): Form<EmailForm>,
) -> Result<Response, OperationOutcomeError> {
    if !state.config.signup.enabled {
        return Ok(signup_closed());
    }

    flow::begin(
        state.as_ref(),
        &session,
        &csrf_token,
        client_ip(ip),
        Purpose::Signup,
        form,
    )
    .await
}
