use crate::fhir_client::{
    ServerCTX,
    middleware::{
        ServerMiddlewareState,
        operations::{ServerOperationContext, custom_operations::TerminologyResolver},
    },
};
use haste_fhir_client::{FHIRClient, request::InvocationRequest};
use haste_fhir_generated_ops::generated::ValueSetValidateCode;
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_ops::OperationExecutor;
use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_jwt::{ProjectId, TenantId};
use haste_repository::Repository;
use std::sync::Arc;

pub fn valueset_validate_code_op<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
    Client: FHIRClient<Arc<ServerCTX<Client>>, OperationOutcomeError> + 'static,
>() -> OperationExecutor<
    ServerOperationContext<ServerMiddlewareState<Repo, Search, Terminology>, Client>,
    ValueSetValidateCode::Input,
    ValueSetValidateCode::Output,
> {
    OperationExecutor::new(
        ValueSetValidateCode::CODE.to_string(),
        Box::new(
            |context: ServerOperationContext<
                ServerMiddlewareState<Repo, Search, Terminology>,
                Client,
            >,
             _tenant: TenantId,
             _project: ProjectId,
             _request: &InvocationRequest,
             input: ValueSetValidateCode::Input| {
                Box::pin(async move {
                    let resolver = TerminologyResolver::new(context.ctx);
                    let output = context.state.terminology.validate(resolver, input).await?;
                    Ok(output)
                })
            },
        ),
    )
}
