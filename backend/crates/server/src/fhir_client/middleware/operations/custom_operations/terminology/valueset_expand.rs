use crate::fhir_client::{
    middleware::{Backend, operations::ServerOperationContext},
    resolver::ServerCTXResolver,
};
use haste_fhir_client::request::InvocationRequest;
use haste_fhir_generated_ops::generated::ValueSetExpand;
use haste_fhir_ops::OperationExecutor;
use haste_fhir_terminology::FHIRTerminology;
use haste_jwt::{ProjectId, TenantId};

pub fn valueset_expand_op<B: Backend>()
-> OperationExecutor<ServerOperationContext<B>, ValueSetExpand::Input, ValueSetExpand::Output> {
    OperationExecutor::new(
        &ValueSetExpand::OPERATION_DEFINITION,
        Box::new(
            |context: ServerOperationContext<B>,
             _tenant: TenantId,
             _project: ProjectId,
             _request: &InvocationRequest,
             input: ValueSetExpand::Input| {
                Box::pin(async move {
                    let output = context
                        .state
                        .terminology
                        .expand(ServerCTXResolver::new(context.ctx), input)
                        .await?;
                    Ok(output)
                })
            },
        ),
    )
}
