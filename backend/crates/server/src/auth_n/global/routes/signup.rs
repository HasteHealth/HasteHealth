use crate::services::AppState;
use axum::extract::State;
use axum_extra::routing::TypedPath;
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_repository::Repository;
use std::sync::Arc;

#[derive(serde::Deserialize, TypedPath)]
#[typed_path("/signup")]
pub struct GlobalSignup {}

#[allow(dead_code)]
pub async fn signup_initial<
    Repo: Repository + Send + Sync,
    Search: SearchEngine + Send + Sync,
    Terminology: FHIRTerminology + Send + Sync,
>(
    _: GlobalSignup,
    State(_app_state): State<Arc<AppState<Repo, Search, Terminology>>>,
) -> Result<(), OperationOutcomeError> {
    Ok(())
}
