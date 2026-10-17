use crate::fhir_client::{
    middleware::{Backend, operations::ServerOperationContext},
    resolver::ServerCTXResolver,
};
use haste_fhir_client::request::InvocationRequest;
use haste_fhir_generated_ops::generated::ValueSetValidateCode;
use haste_fhir_ops::OperationExecutor;
use haste_fhir_terminology::FHIRTerminology;
use haste_jwt::{ProjectId, TenantId};

pub fn valueset_validate_code_op<B: Backend>() -> OperationExecutor<
    ServerOperationContext<B>,
    ValueSetValidateCode::Input,
    ValueSetValidateCode::Output,
> {
    OperationExecutor::new(
        &ValueSetValidateCode::OPERATION_DEFINITION,
        Box::new(
            |context: ServerOperationContext<B>,
             _tenant: TenantId,
             _project: ProjectId,
             _request: &InvocationRequest,
             input: ValueSetValidateCode::Input| {
                Box::pin(async move {
                    let resolver = ServerCTXResolver::new(context.ctx);
                    let output = context.state.terminology.validate(resolver, input).await?;
                    Ok(output)
                })
            },
        ),
    )
}
