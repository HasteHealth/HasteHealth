use crate::fhir_client::{
    ServerCTX,
    middleware::{ServerMiddlewareState, operations::ServerOperationContext},
};
use chrono::Utc;
use haste_fhir_client::{FHIRClient, request::InvocationRequest};
use haste_fhir_generated_ops::generated::ViewDefinitionRun;
use haste_fhir_model::r4::{
    self,
    generated::{
        resources::{Binary, Resource, ResourceType, ViewDefinition},
        terminology::IssueType,
    },
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_ops::OperationExecutor;
use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_jwt::{ProjectId, ResourceId, TenantId};
use haste_repository::Repository;
use std::sync::Arc;

async fn resolve_view_definition<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
    Client: FHIRClient<Arc<ServerCTX<Client>>, OperationOutcomeError> + 'static,
>(
    context: ServerOperationContext<ServerMiddlewareState<Repo, Search, Terminology>, Client>,
    input: &ViewDefinitionRun::Input,
) -> Result<ViewDefinition, OperationOutcomeError> {
    if let Some(view_definition) = &input.viewResource {
        return Ok(view_definition.clone());
    } else if let Some(view_definition_reference) = input.viewReference.as_ref() {
        let view_definition_reference = view_definition_reference
            .reference
            .as_ref()
            .ok_or_else(|| {
                OperationOutcomeError::error(
                    IssueType::Invalid(None),
                    "viewReference.reference is required".to_string(),
                )
            })?
            .value
            .as_ref()
            .ok_or_else(|| {
                OperationOutcomeError::error(
                    IssueType::Invalid(None),
                    "viewReference.reference.value is required".to_string(),
                )
            })?;

        let reference_pieces = view_definition_reference.split('/').collect::<Vec<_>>();

        let view_definition_id = ResourceId::new(
            reference_pieces
                .last()
                .ok_or_else(|| {
                    OperationOutcomeError::error(
                        IssueType::Invalid(None),
                        "Invalid viewReference.reference format".to_string(),
                    )
                })?
                .to_string(),
        );

        let Some(view_definition) = context
            .state
            .repo
            .read_latest(
                &context.ctx.tenant,
                &context.ctx.project,
                &ResourceType::ViewDefinition,
                &view_definition_id,
            )
            .await?
            .and_then(|v| match v {
                Resource::ViewDefinition(view_definition) => Some(view_definition),
                _ => None,
            })
        else {
            return Err(OperationOutcomeError::error(
                IssueType::NotFound(None),
                format!(
                    "ViewDefinition not found with id '{:?}'",
                    view_definition_id
                ),
            ));
        };

        Ok(view_definition)
    } else {
        Err(OperationOutcomeError::error(
            IssueType::Invalid(None),
            "Either viewResource or viewReference must be provided".to_string(),
        ))
    }
}

async fn get_input<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
    Client: FHIRClient<Arc<ServerCTX<Client>>, OperationOutcomeError> + 'static,
>(
    context: ServerOperationContext<ServerMiddlewareState<Repo, Search, Terminology>, Client>,
    input: &ViewDefinitionRun::Input,
) -> Result<Vec<Resource>, OperationOutcomeError> {
    if let Some(input_resource) = input.resource.clone() {
        Ok(input_resource)
    } else {
        Ok(vec![])
    }
}

async fn process_view_definition<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
    Client: FHIRClient<Arc<ServerCTX<Client>>, OperationOutcomeError> + 'static,
>(
    context: ServerOperationContext<ServerMiddlewareState<Repo, Search, Terminology>, Client>,
    view_definition: &ViewDefinition,
    input: &ViewDefinitionRun::Input,
) -> Result<Binary, OperationOutcomeError> {
    let limit = input
        ._limit
        .as_ref()
        .and_then(|limit| limit.value.clone())
        .unwrap_or(100);

    let since = input
        ._since
        .as_ref()
        .and_then(|since| since.value.clone())
        .unwrap_or(r4::datetime::Instant::Iso8601(Utc::now()));

    let input_ = get_input(context, input).await?;

    // Implement the logic to process the view definition and return the result as Binary
    // For now, we will return an empty Binary as a placeholder
    Ok(Binary::default())
}

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
                    let view_definition = resolve_view_definition(context, &input).await?;

                    Ok(ViewDefinitionRun::Output {
                        return_: Binary::default(),
                    })
                })
            },
        ),
    )
}
