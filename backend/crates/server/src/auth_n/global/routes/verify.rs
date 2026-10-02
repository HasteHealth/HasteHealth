//! `/auth/verify`: code entry and the steps after it. See [`super::flow`].

use crate::{
    auth_n::global::{
        email_code::{self, CheckOutcome},
        routes::flow::{self, check_csrf, client_ip},
    },
    extract::csrf_token::CSRFToken,
    services::ServerState,
    ui::pages::global_auth::{LOGIN_ROUTE, code_entry_html},
};
use axum::{
    Form,
    extract::{Query, State},
    response::{IntoResponse, Redirect, Response},
};
use axum_client_ip::ClientIp;
use axum_extra::{extract::Cached, routing::TypedPath};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_jwt::TenantId;
use haste_repository::Repository;
use serde::Deserialize;
use std::sync::Arc;
use tower_sessions::Session;

#[derive(TypedPath, Deserialize)]
#[typed_path("/verify")]
pub struct Verify;

/// Reached by reloading the code page.
pub async fn verify_get(
    _: Verify,
    CSRFToken(csrf_token): CSRFToken,
    Cached(session): Cached<Session>,
) -> Result<Response, OperationOutcomeError> {
    let Some(pending) = email_code::pending(&session).await? else {
        return Ok(Redirect::to(LOGIN_ROUTE).into_response());
    };

    Ok(code_entry_html(
        &csrf_token,
        &pending.email,
        flow::start_over_route(pending.purpose),
        None,
        None,
    )
    .into_response())
}

#[derive(Deserialize)]
pub struct VerifyForm {
    pub csrf_token: String,
    pub otp_code: String,
}

pub async fn verify_post<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
>(
    _: Verify,
    CSRFToken(csrf_token): CSRFToken,
    State(state): State<Arc<ServerState<Repo, Search, Terminology>>>,
    Cached(session): Cached<Session>,
    Form(form): Form<VerifyForm>,
) -> Result<Response, OperationOutcomeError> {
    check_csrf(&csrf_token, &form.csrf_token)?;

    let error = match email_code::check(&session, &form.otp_code).await? {
        CheckOutcome::Verified(verified) => {
            return flow::finish(state.as_ref(), &session, &csrf_token, verified).await;
        }
        CheckOutcome::NotPending => return Ok(Redirect::to(LOGIN_ROUTE).into_response()),
        CheckOutcome::Incorrect { attempts_left: 1 } => {
            "That code is not right. One more try before you need a new code.".to_string()
        }
        CheckOutcome::Incorrect { attempts_left } => {
            format!("That code is not right. {attempts_left} tries left.")
        }
        CheckOutcome::Expired => "That code has expired. Send a new one.".to_string(),
        CheckOutcome::TooManyAttempts => {
            "Too many wrong codes. Send a new one to try again.".to_string()
        }
    };

    // Every outcome that reaches here kept the pending entry.
    let Some(pending) = email_code::pending(&session).await? else {
        return Ok(Redirect::to(LOGIN_ROUTE).into_response());
    };

    Ok(code_entry_html(
        &csrf_token,
        &pending.email,
        flow::start_over_route(pending.purpose),
        None,
        Some(&error),
    )
    .into_response())
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/verify/resend")]
pub struct Resend;

#[derive(Deserialize)]
pub struct ResendForm {
    pub csrf_token: String,
}

pub async fn resend_post<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
>(
    _: Resend,
    CSRFToken(csrf_token): CSRFToken,
    State(state): State<Arc<ServerState<Repo, Search, Terminology>>>,
    Cached(session): Cached<Session>,
    ip: Result<ClientIp, axum_client_ip::Rejection>,
    Form(form): Form<ResendForm>,
) -> Result<Response, OperationOutcomeError> {
    check_csrf(&csrf_token, &form.csrf_token)?;

    flow::resend(state.as_ref(), &session, &csrf_token, client_ip(ip)).await
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/verify/tenant")]
pub struct VerifyTenant;

#[derive(Deserialize)]
pub struct TenantQuery {
    pub tenant: String,
}

/// A link on the tenant chooser.
pub async fn tenant_get<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
>(
    _: VerifyTenant,
    Query(query): Query<TenantQuery>,
    State(state): State<Arc<ServerState<Repo, Search, Terminology>>>,
    Cached(session): Cached<Session>,
) -> Result<Response, OperationOutcomeError> {
    let Some(verified) = email_code::verified(&session).await? else {
        return Ok(Redirect::to(LOGIN_ROUTE).into_response());
    };

    flow::choose_tenant(
        state.as_ref(),
        &session,
        verified,
        &TenantId::new(query.tenant),
    )
    .await
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/verify/create")]
pub struct VerifyCreate;

#[derive(Deserialize)]
pub struct CreateForm {
    pub csrf_token: String,
    pub tenant: String,
    #[serde(default)]
    pub password: String,
}

/// The "name your workspace" form.
pub async fn create_post<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
>(
    _: VerifyCreate,
    CSRFToken(csrf_token): CSRFToken,
    State(state): State<Arc<ServerState<Repo, Search, Terminology>>>,
    Cached(session): Cached<Session>,
    Form(form): Form<CreateForm>,
) -> Result<Response, OperationOutcomeError> {
    check_csrf(&csrf_token, &form.csrf_token)?;

    let Some(verified) = email_code::verified(&session).await? else {
        return Ok(Redirect::to(LOGIN_ROUTE).into_response());
    };

    flow::create_workspace(
        state.as_ref(),
        &session,
        &csrf_token,
        verified,
        &form.tenant,
        &form.password,
    )
    .await
}
