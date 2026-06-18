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
        terminology::{IssueType, OutputFormatCodes},
    },
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_ops::OperationExecutor;
use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_fhirpath::{Config, FPEngine};
use haste_jwt::{ProjectId, ResourceId, TenantId};
use haste_reflect::MetaValue;
use haste_repository::Repository;
use std::borrow::Cow;
use std::{collections::HashMap, sync::Arc};

async fn resolve_view_definition<
    'a,
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
    Client: FHIRClient<Arc<ServerCTX<Client>>, OperationOutcomeError> + 'static,
>(
    context: &ServerOperationContext<ServerMiddlewareState<Repo, Search, Terminology>, Client>,
    input: &'a ViewDefinitionRun::Input,
) -> Result<Cow<'a, ViewDefinition>, OperationOutcomeError> {
    if let Some(view_definition) = &input.viewResource {
        Ok(Cow::Borrowed(view_definition))
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

        Ok(Cow::Owned(view_definition))
    } else {
        Err(OperationOutcomeError::error(
            IssueType::Invalid(None),
            "Either viewResource or viewReference must be provided".to_string(),
        ))
    }
}

fn get_output_format(
    input: &ViewDefinitionRun::Input,
) -> Result<OutputFormatCodes, OperationOutcomeError> {
    let output_format = input
        ._format
        .as_ref()
        .and_then(|output_format| output_format.value.clone())
        .and_then(|format| {
            Some(OutputFormatCodes::try_from(format).map_err(|e| {
                OperationOutcomeError::error(
                    IssueType::Invalid(None),
                    format!("Invalid output format: {}", e),
                )
            }))
        })
        .unwrap_or(Ok(OutputFormatCodes::Ndjson(None)))?;

    Ok(output_format)
}

async fn get_resources_to_process<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
    Client: FHIRClient<Arc<ServerCTX<Client>>, OperationOutcomeError> + 'static,
>(
    _context: &ServerOperationContext<ServerMiddlewareState<Repo, Search, Terminology>, Client>,
    input: &ViewDefinitionRun::Input,
) -> Result<Vec<Resource>, OperationOutcomeError> {
    if let Some(input_resource) = input.resource.clone() {
        Ok(input_resource)
    } else {
        Ok(vec![])
    }
}

fn build_hashmap_fp_variables<'a>(
    viewdefinition: &'a ViewDefinition,
) -> HashMap<String, &'a dyn MetaValue> {
    let mut hashmap = HashMap::new();

    if let Some(constants) = &viewdefinition.constant {
        for constant in constants {
            if let Some(name) = &constant.name.value.as_ref() {
                hashmap.insert((*name).clone(), &constant.value as &dyn MetaValue);
            }
        }
    }

    hashmap
}

async fn process_resource<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
    Client: FHIRClient<Arc<ServerCTX<Client>>, OperationOutcomeError> + 'static,
>(
    _context: &ServerOperationContext<ServerMiddlewareState<Repo, Search, Terminology>, Client>,
    view_definition: &ViewDefinition,
    input: &Resource,
) -> Result<(), OperationOutcomeError> {
    let fp_engine = FPEngine::new();
    let variables = Arc::new(build_hashmap_fp_variables(view_definition));
    if let Some(_where_conditionals) = &view_definition.where_ {}

    for _select_statement in view_definition.select.iter() {
        let _context = fp_engine.evaluate_with_config(
            "$this",
            vec![input],
            Arc::new(Config {
                variable_resolver: Some(haste_fhirpath::ExternalConstantResolver::Variable(
                    variables.clone(),
                )),
            }),
        );
    }

    todo!("Not implemented");
}

async fn process_view_definition<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
    Client: FHIRClient<Arc<ServerCTX<Client>>, OperationOutcomeError> + 'static,
>(
    context: &ServerOperationContext<ServerMiddlewareState<Repo, Search, Terminology>, Client>,
    view_definition: &ViewDefinition,
    input: &ViewDefinitionRun::Input,
) -> Result<Binary, OperationOutcomeError> {
    let _output_format = get_output_format(input)?;
    let _limit = input
        ._limit
        .as_ref()
        .and_then(|limit| limit.value.clone())
        .unwrap_or(100);

    let _since = input
        ._since
        .as_ref()
        .and_then(|since| since.value.clone())
        .unwrap_or(r4::datetime::Instant::Iso8601(Utc::now()));

    let input_ = get_resources_to_process(context, input).await?;

    let _result = input_
        .iter()
        .map(|resource| process_resource(context, view_definition, resource));

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
                    let view_definition = resolve_view_definition(&context, &input).await?;

                    let output =
                        process_view_definition(&context, &view_definition, &input).await?;

                    Ok(ViewDefinitionRun::Output { return_: output })
                })
            },
        ),
    )
}
