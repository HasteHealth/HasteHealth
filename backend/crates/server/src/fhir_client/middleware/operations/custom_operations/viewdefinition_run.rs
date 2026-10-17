use crate::fhir_client::middleware::{Backend, operations::ServerOperationContext};
use haste_fhir_client::request::InvocationRequest;
use haste_fhir_generated_ops::generated::ViewDefinitionRun;
use haste_fhir_ops::OperationExecutor;
use haste_jwt::{ProjectId, TenantId};

pub fn view_definition_run<B: Backend>()
-> OperationExecutor<ServerOperationContext<B>, ViewDefinitionRun::Input, ViewDefinitionRun::Output>
{
    OperationExecutor::new(
        &ViewDefinitionRun::OPERATION_DEFINITION,
        Box::new(
            |context: ServerOperationContext<B>,
             _tenant: TenantId,
             _project: ProjectId,
             _request: &InvocationRequest,
             input: ViewDefinitionRun::Input| {
                Box::pin(async move {
                    let output = haste_sql_on_fhir::view_definition_run(
                        context.ctx.clone(),
                        context.ctx.client.clone(),
                        &input,
                    )
                    .await?;

                    Ok(output)
                })
            },
        ),
    )
}
