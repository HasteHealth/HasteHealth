use axum::extract::{OriginalUri, State};
use oxidized_fhir_search::SearchEngine;
use oxidized_fhir_terminology::FHIRTerminology;
use oxidized_repository::{Repository, types::tenant::Tenant};

use crate::{auth_n::oidc::extract::client_app::OIDCClientApplication, services::AppState};

#[derive(TypedPath)]
#[typed_path("/logout")]
pub struct ScopePost;

fn scope_post<
    Repo: Repository + Send + Sync,
    Search: SearchEngine + Send + Sync,
    Terminology: FHIRTerminology + Send + Sync,
>(
    _: ScopePost,
    uri: OriginalUri,
    State(state): State<Arc<AppState<Repo, Search, Terminology>>>,
    current_session: Session,
    OIDCClientApplication(_client_app): OIDCClientApplication,
    Tenant { tenant }: Tenant,
) -> Result<Response, OperationOutcomeError> {
}
