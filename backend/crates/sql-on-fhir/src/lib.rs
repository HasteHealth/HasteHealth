use base64::{Engine as _, engine::general_purpose};
use chrono::Utc;
use futures::{StreamExt as _, stream::FuturesOrdered};
use haste_fhir_client::{
    FHIRClient,
    url::{Parameter, ParsedParameter, ParsedParameters},
};
use haste_fhir_generated_ops::generated::ViewDefinitionRun;
use haste_fhir_model::r4::{
    self,
    generated::{
        resources::{Binary, Resource, ResourceType, ViewDefinition},
        terminology::{IssueType, OutputFormatCodes},
        types::FHIRBase64Binary,
    },
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhirpath::{Config, FPEngine};
use haste_reflect::MetaValue;
use itertools::Itertools as _;
use ordermap::OrderMap;
use std::{borrow::Cow, collections::HashMap, sync::Arc};

use crate::conversions::primitives::PrimitiveValue;

mod conversions;
mod output;

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
    context: CTX,
    client: &Client,
    view_definition: &ViewDefinition,
    input: &ViewDefinitionRun::Input,
) -> Result<Vec<Resource>, OperationOutcomeError> {
    if let Some(input_resources) = input.resource.clone() {
        Ok(input_resources)
    } else {
        let since = input
            ._since
            .as_ref()
            .and_then(|since| since.value.clone())
            .unwrap_or(r4::datetime::Instant::Iso8601(Utc::now()));

        let Some(resource_type): Option<String> = view_definition.resource.as_ref().into() else {
            return Err(OperationOutcomeError::error(
                IssueType::Invalid(None),
                "ViewDefinition.resource is required".to_string(),
            ));
        };

        let resource_type = ResourceType::try_from(resource_type).map_err(|e| {
            OperationOutcomeError::error(
                IssueType::Invalid(None),
                format!("Invalid resource type: {}", e),
            )
        })?;

        let result = client
            .history_type(
                context,
                resource_type,
                ParsedParameters::new(vec![
                    ParsedParameter::Result(Parameter {
                        name: "_since".to_string(),
                        value: vec![since.to_string()],
                        modifier: None,
                        chains: None,
                    }),
                    ParsedParameter::Result(Parameter {
                        name: "_count".to_string(),
                        value: vec!["1000".to_string()],
                        modifier: None,
                        chains: None,
                    }),
                ]),
            )
            .await?;

        Ok(vec![Resource::Bundle(result)])
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

fn cartesian_product(
    select_statement_results: Vec<Vec<OrderMap<String, Vec<Option<PrimitiveValue>>>>>,
) -> Vec<OrderMap<String, Vec<Option<PrimitiveValue>>>> {
    let mut output_results = Vec::new();

    for combination in select_statement_results
        .into_iter()
        .multi_cartesian_product()
    {
        let mut combined_result = OrderMap::new();

        for result in combination {
            for (key, value) in result {
                combined_result
                    .entry(key)
                    .or_insert_with(Vec::new)
                    .extend(value);
            }
        }

        output_results.push(combined_result);
    }

    output_results
}

async fn process_resource<
    CTX: Send + Sync + Clone + 'static,
    Client: FHIRClient<CTX, OperationOutcomeError> + Send + Sync + 'static,
>(
    _context: CTX,
    _client: Arc<Client>,
    view_definition: &ViewDefinition,
    input: Box<Resource>,
) -> Result<Vec<OrderMap<String, Vec<Option<PrimitiveValue>>>>, OperationOutcomeError> {
    let fp_engine = FPEngine::new();
    let variables = Arc::new(build_hashmap_fp_variables(view_definition));
    if let Some(_where_conditionals) = &view_definition.where_ {
        return Err(OperationOutcomeError::error(
            IssueType::NotSupported(None),
            "where conditionals are not yet supported".to_string(),
        ));
    }

    let mut select_statement_results = Vec::with_capacity(view_definition.select.len());

    for select_statement in view_definition.select.iter() {
        let fp_config = Arc::new(Config {
            variable_resolver: Some(haste_fhirpath::ExternalConstantResolver::Variable(
                variables.clone(),
            )),
        });

        let mut iterable_context = None;
        let mut set_null = false;

        if let Some(for_each_fp) = select_statement
            .forEach
            .as_ref()
            .and_then(|f| f.value.as_ref())
        {
            iterable_context = Some(
                fp_engine
                    .evaluate_with_config(for_each_fp, vec![&input], fp_config.clone())
                    .await
                    .map_err(|e| {
                        OperationOutcomeError::error(
                            IssueType::Exception(None),
                            format!("Error evaluating forEach expression: {}", e),
                        )
                    })?,
            );
        } else if let Some(for_each_or_null_fp) = select_statement
            .forEachOrNull
            .as_ref()
            .and_then(|f| f.value.as_ref())
        {
            iterable_context = Some(
                fp_engine
                    .evaluate_with_config(for_each_or_null_fp, vec![&input], fp_config.clone())
                    .await
                    .map_err(|e| {
                        OperationOutcomeError::error(
                            IssueType::Exception(None),
                            format!("Error evaluating forEachOrNull expression: {}", e),
                        )
                    })?,
            );
            set_null = true;
        } else if let Some(_repeat) = select_statement.repeat.as_ref() {
            return Err(OperationOutcomeError::error(
                IssueType::NotSupported(None),
                "repeat is not yet supported".to_string(),
            ));
        }

        let select_context: Vec<&dyn MetaValue> = if let Some(iterable) = iterable_context.as_ref()
        {
            iterable.iter().collect()
        } else {
            vec![&input]
        };

        let mut select_results = Vec::with_capacity(select_context.len());

        if set_null && select_context.is_empty() {
            let mut output_result = OrderMap::new();
            for column in select_statement.column.as_ref().into_iter().flatten() {
                let Some(name) = column.name.value.as_ref().map(|n| n.as_str()) else {
                    return Err(OperationOutcomeError::error(
                        IssueType::Invalid(None),
                        "Column name is required".to_string(),
                    ));
                };
                output_result.insert(name.to_string(), vec![None]);
            }
            select_results.push(output_result);
        }

        for context in select_context {
            let mut output_result = OrderMap::new();
            for column in select_statement.column.as_ref().into_iter().flatten() {
                let Some(path) = column.path.value.as_ref().map(|p| p.as_str()) else {
                    return Err(OperationOutcomeError::error(
                        IssueType::Invalid(None),
                        "Column path is required".to_string(),
                    ));
                };

                let Some(name) = column.name.value.as_ref().map(|n| n.as_str()) else {
                    return Err(OperationOutcomeError::error(
                        IssueType::Invalid(None),
                        "Column name is required".to_string(),
                    ));
                };

                let result = fp_engine
                    .evaluate(path, vec![context; 1])
                    .await
                    .map_err(|e| {
                        OperationOutcomeError::error(
                            IssueType::Exception(None),
                            format!("Error evaluating expression: {}", e),
                        )
                    })?;

                let column_type = column
                    .type_
                    .as_ref()
                    .and_then(|t| t.value.as_ref())
                    .map(|t| t.as_str())
                    // Default to string.
                    .unwrap_or(
                        // If column type is not set than assume it's the first values fhir_type
                        // or default to string if there are no values.
                        result
                            .iter()
                            .peekable()
                            .next()
                            .map(|v| v.fhir_type())
                            .unwrap_or("string"),
                    );

                let column_result = result
                    .iter()
                    .map(|value| conversions::primitives::convert_meta_value(column_type, value))
                    .collect::<Result<Vec<Option<PrimitiveValue>>, OperationOutcomeError>>()?;

                let is_collection = column
                    .collection
                    .as_ref()
                    .and_then(|c| c.value)
                    .unwrap_or(false);

                if is_collection && column_result.len() > 1 {
                    return Err(OperationOutcomeError::error(
                        IssueType::Invalid(None),
                        "Column result is a collection but the column is not marked as a collection"
                            .to_string(),
                    ));
                }

                output_result.insert(name.to_string(), column_result);
            }
            select_results.push(output_result);
        }

        select_statement_results.push(select_results);
    }

    let output_results = cartesian_product(select_statement_results);

    Ok(output_results)
}

fn flatten_results(resource: Vec<Resource>) -> Vec<Box<Resource>> {
    let mut resources = Vec::new();
    for resource in resource {
        match resource {
            Resource::Bundle(bundle) => {
                for entry in bundle.entry.into_iter().flatten() {
                    if let Some(resource) = entry.resource {
                        resources.push(resource);
                    }
                }
            }
            _ => {
                resources.push(Box::new(resource));
            }
        }
    }

    resources
}

async fn process_view_definition<
    CTX: Send + Sync + Clone + 'static,
    Client: FHIRClient<CTX, OperationOutcomeError> + Send + Sync + 'static,
>(
    context: CTX,
    output_format: &OutputFormatCodes,
    client: Arc<Client>,
    view_definition: &ViewDefinition,
    input: &ViewDefinitionRun::Input,
) -> Result<Binary, OperationOutcomeError> {
    let _output_format = get_output_format(input)?;
    let _limit = input
        ._limit
        .as_ref()
        .and_then(|limit| limit.value.clone())
        .unwrap_or(1000);

    let input_ = flatten_results(
        get_resources_to_process(context.clone(), client.as_ref(), view_definition, input).await?,
    );

    let mut tasks = FuturesOrdered::new();

    for resource in input_ {
        let task = async {
            process_resource(context.clone(), client.clone(), view_definition, resource).await
        };

        tasks.push_back(task);
    }

    let mut results = Vec::with_capacity(tasks.len());

    while let Some(result) = tasks.next().await {
        results.push(result?);
    }

    let results = results.into_iter().flatten().collect::<Vec<_>>();

    match output_format {
        OutputFormatCodes::Csv(_) => {
            let data = output::csv::csv(results)?;

            let base64_string: String = general_purpose::STANDARD.encode(&data);

            Ok(Binary {
                data: Some(Box::new(FHIRBase64Binary {
                    value: Some(base64_string),
                    ..Default::default()
                })),
                ..Default::default()
            })
        }
        _ => Err(OperationOutcomeError::error(
            IssueType::NotSupported(None),
            format!("Output format '{:?}' is not supported", output_format),
        )),
    }
}

pub async fn view_definition_run<
    CTX: Send + Sync + Clone + 'static,
    Client: FHIRClient<CTX, OperationOutcomeError> + Send + Sync + 'static,
>(
    context: CTX,
    client: Arc<Client>,
    input: &ViewDefinitionRun::Input,
) -> Result<ViewDefinitionRun::Output, OperationOutcomeError> {
    let output_format = input
        ._format
        .as_ref()
        .and_then(|v| v.value.as_ref())
        .and_then(|s| OutputFormatCodes::try_from(s.to_string()).ok())
        .unwrap_or(OutputFormatCodes::Csv(None));

    let view_definition =
        Arc::new(resolve_view_definition(context.clone(), client.as_ref(), &input).await?);

    let output = process_view_definition(
        context,
        &output_format,
        client,
        view_definition.as_ref(),
        &input,
    )
    .await?;

    Ok(ViewDefinitionRun::Output { return_: output })
}
