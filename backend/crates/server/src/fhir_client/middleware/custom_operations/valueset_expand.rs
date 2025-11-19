use crate::fhir_client::middleware::ServerMiddlewareState;
use haste_fhir_generated_ops::generated::ValueSetExpand;
use haste_fhir_ops::OperationExecutor;
use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_jwt::{ProjectId, TenantId};
use haste_repository::Repository;

pub fn valueset_expand<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
>() -> OperationExecutor<
    ServerMiddlewareState<Repo, Search, Terminology>,
    ValueSetExpand::Input,
    ValueSetExpand::Output,
> {
    OperationExecutor::new(
        ValueSetExpand::CODE.to_string(),
        Box::new(
            |ctx: ServerMiddlewareState<Repo, Search, Terminology>,
             _tenant: TenantId,
             _project: ProjectId,
             input: ValueSetExpand::Input| {
                Box::pin(async move {
                    let output = ctx.terminology.expand(input).await?;
                    Ok(output)
                })
            },
        ),
    )
}
