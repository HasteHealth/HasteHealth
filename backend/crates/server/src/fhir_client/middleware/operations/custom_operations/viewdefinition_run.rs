use crate::fhir_client::{
    ServerCTX,
    middleware::{ServerMiddlewareState, operations::ServerOperationContext},
};
use haste_fhir_client::{FHIRClient, request::InvocationRequest};
use haste_fhir_generated_ops::generated::ViewDefinitionRun;
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_ops::OperationExecutor;
use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_jwt::{ProjectId, TenantId};
use haste_repository::Repository;
use std::sync::Arc;

pub fn view_definition_run<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
    Client: FHIRClient<Arc<ServerCTX<Client>>, OperationOutcomeError> + 'static,
>() -> OperationExecutor<
    ServerOperationContext<ServerMiddlewareState<Repo, Search, Terminology>, Client>,
    ViewDefinitionRun::Input,
    ViewDefinitionRun::Output,
> {
    OperationExecutor::new(
        ViewDefinitionRun::CODE.to_string(),
        Box::new(
            |context: ServerOperationContext<
                ServerMiddlewareState<Repo, Search, Terminology>,
                Client,
            >,
             _tenant: TenantId,
             _project: ProjectId,
             _request: &InvocationRequest,
             input: ViewDefinitionRun::Input| {
                Box::pin(async move {
                    let output = haste_sql_on_fhir::view_definition_run(
                        context.ctx.clone(),
                        context.ctx.client.as_ref(),
                        &input,
                    )
                    .await?;

                    Ok(output)
                })
            },
        ),
    )
}
