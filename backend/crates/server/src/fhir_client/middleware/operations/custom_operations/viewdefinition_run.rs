use crate::fhir_client::middleware::{Backend, operations::ServerOperationContext};
use haste_fhir_client::request::InvocationRequest;
use haste_fhir_generated_ops::generated::ViewDefinitionRun;
use haste_fhir_model::r4::generated::{
    resources::ResourceType,
    terminology::IssueType,
    types::{FHIRString, Reference},
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_ops::OperationExecutor;
use haste_jwt::{ProjectId, TenantId};

/// At instance level the view is the ViewDefinition in the URL
/// (`ViewDefinition/[id]/$viewdefinition-run`), so `viewReference` is set from
/// it and neither view parameter may be sent.
fn with_instance_view(
    input: ViewDefinitionRun::Input,
    instance_id: Option<String>,
) -> Result<ViewDefinitionRun::Input, OperationOutcomeError> {
    let Some(id) = instance_id else {
        return Ok(input);
    };

    if input.viewReference.is_some() || input.viewResource.is_some() {
        return Err(OperationOutcomeError::error(
            IssueType::invalid(),
            "viewReference and viewResource are not allowed at instance level: the view is the \
             ViewDefinition in the URL."
                .to_string(),
        ));
    }

    Ok(ViewDefinitionRun::Input {
        viewReference: Some(Reference {
            reference: Some(Box::new(FHIRString {
                value: Some(format!("{}/{id}", ResourceType::ViewDefinition.as_ref())),
                ..Default::default()
            })),
            ..Default::default()
        }),
        ..input
    })
}

pub fn view_definition_run<B: Backend>()
-> OperationExecutor<ServerOperationContext<B>, ViewDefinitionRun::Input, ViewDefinitionRun::Output>
{
    OperationExecutor::new(
        &ViewDefinitionRun::OPERATION_DEFINITION,
        Box::new(
            |context: ServerOperationContext<B>,
             _tenant: TenantId,
             _project: ProjectId,
             request: &InvocationRequest,
             input: ViewDefinitionRun::Input| {
                let instance_id = match request {
                    InvocationRequest::Instance(instance) => Some(instance.id.clone()),
                    InvocationRequest::Type(_) | InvocationRequest::System(_) => None,
                };

                Box::pin(async move {
                    let input = with_instance_view(input, instance_id)?;

                    let output = haste_sql_on_fhir::view_definition_run(
                        context.ctx.clone(),
                        context.ctx.client.clone(),
                        &input,
                        context.state.config.sql_on_fhir.run_limits(),
                    )
                    .await?;

                    Ok(output)
                })
            },
        ),
    )
}
