use chrono::Utc;
use haste_fhir_client::FHIRClient;
use haste_fhir_generated_ops::generated::ViewDefinitionRun;
use haste_fhir_model::r4::{
    self,
    generated::{
        resources::{Binary, Resource, ResourceType, ViewDefinition},
        terminology::{IssueType, OutputFormatCodes},
    },
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhirpath::{Config, FPEngine};
use haste_reflect::MetaValue;
use std::borrow::Cow;
use std::{collections::HashMap, sync::Arc};

async fn resolve_view_definition<
    'a,
    CTX: Send + Sync + Clone + 'static,
    Client: FHIRClient<CTX, OperationOutcomeError> + Send + Sync + 'static,
>(
    context: CTX,
    client: &Client,
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

        let view_definition_id = reference_pieces
            .last()
            .ok_or_else(|| {
                OperationOutcomeError::error(
                    IssueType::Invalid(None),
                    "Invalid viewReference.reference format".to_string(),
                )
            })?
            .to_string();

        let result = client
            .read(
                context,
                ResourceType::ViewDefinition,
                view_definition_id.clone(),
            )
            .await?
            .ok_or_else(|| {
                OperationOutcomeError::error(
                    IssueType::NotFound(None),
                    format!(
                        "ViewDefinition not found with id '{:?}'",
                        view_definition_id
                    ),
                )
            })?;

        if let Resource::ViewDefinition(view_definition) = result {
            Ok(Cow::Owned(view_definition))
        } else {
            Err(OperationOutcomeError::error(
                IssueType::Invalid(None),
                "Referenced resource is not a ViewDefinition".to_string(),
            ))
        }
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
    CTX: Send + Sync + Clone + 'static,
    Client: FHIRClient<CTX, OperationOutcomeError> + Send + Sync + 'static,
>(
    _context: CTX,
    _client: &Client,
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
    CTX: Send + Sync + Clone + 'static,
    Client: FHIRClient<CTX, OperationOutcomeError> + Send + Sync + 'static,
>(
    _context: CTX,
    _client: &Client,
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
    CTX: Send + Sync + Clone + 'static,
    Client: FHIRClient<CTX, OperationOutcomeError> + Send + Sync + 'static,
>(
    context: CTX,
    client: &Client,
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

    let input_ = get_resources_to_process(context.clone(), client, input).await?;

    let _result = input_
        .iter()
        .map(|resource| process_resource(context.clone(), client, view_definition, resource));

    // Implement the logic to process the view definition and return the result as Binary
    // For now, we will return an empty Binary as a placeholder
    Ok(Binary::default())
}

pub async fn view_definition_run<
    CTX: Send + Sync + Clone + 'static,
    Client: FHIRClient<CTX, OperationOutcomeError> + Send + Sync + 'static,
>(
    context: CTX,
    client: &Client,
    input: &ViewDefinitionRun::Input,
) -> Result<ViewDefinitionRun::Output, OperationOutcomeError> {
    let view_definition = resolve_view_definition(context.clone(), client, &input).await?;

    let output = process_view_definition(context, client, &view_definition, &input).await?;

    Ok(ViewDefinitionRun::Output { return_: output })
}
