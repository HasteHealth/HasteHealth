use crate::services::AppState;
use axum::extract::State;
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_repository::Repository;
use std::sync::Arc;

#[derive(serde::Deserialize, axum_extra::routing::TypedPath)]
#[typed_path("/login")]
pub struct LoginPath {}

#[allow(dead_code)]
pub async fn login_global_get<
    Repo: Repository + Send + Sync,
    Search: SearchEngine + Send + Sync,
    Terminology: FHIRTerminology + Send + Sync,
>(
    _: LoginPath,
    State(_app_state): State<Arc<AppState<Repo, Search, Terminology>>>,
) -> Result<(), OperationOutcomeError> {
    // let admin_app_redirect_url = auth_n::oidc::hardcoded_clients::admin_app::redirect_url(
    //     app_state.config.as_ref(),
    //     tenant_id,
    // );

    Ok(())
}
