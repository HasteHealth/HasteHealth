//! `/auth/login`: the same email code as sign-up, for an address that already
//! has a tenant.

use crate::{
    auth_n::global::{
        email_code::Purpose,
        routes::flow::{self, EmailForm, client_ip},
    },
    extract::csrf_token::CSRFToken,
    services::ServerState,
};
use axum::{
    Form,
    extract::State,
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
#[typed_path("/login")]
pub struct GlobalLogin;

pub async fn global_login_get<
    Repo: Repository + Send + Sync,
    Search: SearchEngine + Send + Sync,
    Terminology: FHIRTerminology + Send + Sync,
>(
    _: GlobalLogin,
    CSRFToken(csrf_token): CSRFToken,
    State(state): State<Arc<ServerState<Repo, Search, Terminology>>>,
) -> Result<Response, OperationOutcomeError> {
    Ok(flow::email_form(state.as_ref(), Purpose::Login, &csrf_token, None).into_response())
}

pub async fn global_login_post<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
>(
    _: GlobalLogin,
    CSRFToken(csrf_token): CSRFToken,
    State(state): State<Arc<ServerState<Repo, Search, Terminology>>>,
    Cached(session): Cached<Session>,
    ip: Result<ClientIp, axum_client_ip::Rejection>,
    Form(form): Form<EmailForm>,
) -> Result<Response, OperationOutcomeError> {
    flow::begin(
        state.as_ref(),
        &session,
        &csrf_token,
        client_ip(ip),
        Purpose::Login,
        form,
    )
    .await
}
