use super::{instance_id, read_value_set};
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
             request: &InvocationRequest,
             mut input: ValueSetExpand::Input| {
                // Resolved before the future so it does not borrow `request`.
                let id = instance_id(request);
                Box::pin(async move {
                    if let Some(id) = id {
                        input.valueSet = Some(read_value_set(context.ctx.clone(), id).await?);
                    }
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
