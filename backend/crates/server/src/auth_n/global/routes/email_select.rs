use crate::{extract::path_tenant::TenantIdentifier, services::AppState};
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
use maud::Markup;
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
    todo!();
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
