use crate::fhir_client::{
    ServerCTX,
    middleware::{
        ServerMiddlewareContext, ServerMiddlewareNext, ServerMiddlewareOutput,
        ServerMiddlewareState,
    },
};
use oxidized_fhir_model::r4::generated::terminology::IssueType;
use oxidized_fhir_operation_error::OperationOutcomeError;
use oxidized_fhir_search::SearchEngine;
use oxidized_fhir_terminology::FHIRTerminology;
use oxidized_repository::{
    Repository,
    types::{Author, ProjectId, TenantId},
};
use std::sync::Arc;

/// Sets tenant to search in system for artifact resources IE SDs etc..
pub fn set_artifact_tenant<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
>(
    state: ServerMiddlewareState<Repo, Search, Terminology>,
    mut context: ServerMiddlewareContext,
    next: Option<Arc<ServerMiddlewareNext<Repo, Search, Terminology>>>,
) -> ServerMiddlewareOutput {
    Box::pin(async move {
        let ctx = Arc::new(ServerCTX {
            tenant: TenantId::System,
            project: ProjectId::System,
            fhir_version: context.ctx.fhir_version.clone(),
            author: Author {
                id: "system".to_string(),
                kind: "admin".to_string(),
            },
        });

        context.ctx = ctx;

        if let Some(next) = next {
            next(state, context).await
        } else {
            Err(OperationOutcomeError::fatal(
                IssueType::Exception(None),
                "No next middleware found".to_string(),
            ))
        }
    })
}
