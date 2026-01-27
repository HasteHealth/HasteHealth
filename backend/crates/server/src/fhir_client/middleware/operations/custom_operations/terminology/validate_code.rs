use crate::fhir_client::middleware::operations::ServerOperationContext;
use haste_fhir_client::request::InvocationRequest;
use haste_fhir_generated_ops::generated::ValueSetValidateCode;
use haste_fhir_ops::OperationExecutor;
use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_jwt::{ProjectId, TenantId};
use haste_repository::Repository;

pub fn valueset_validate_code_op<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
>() -> OperationExecutor<
    ServerOperationContext<Repo, Search, Terminology>,
    ValueSetValidateCode::Input,
    ValueSetValidateCode::Output,
> {
    OperationExecutor::new(
        ValueSetValidateCode::CODE.to_string(),
        Box::new(
            |context: ServerOperationContext<Repo, Search, Terminology>,
             _tenant: TenantId,
             _project: ProjectId,
             _request: &InvocationRequest,
             input: ValueSetValidateCode::Input| {
                Box::pin(async move {
                    let output = context.state.terminology.validate(input).await?;
                    Ok(output)
                })
            },
        ),
    )
}
