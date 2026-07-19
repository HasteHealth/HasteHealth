use base64::{Engine as _, engine::general_purpose};
use chrono::Utc;
use futures::{StreamExt as _, stream::FuturesOrdered};
use haste_fhir_client::FHIRClient;
use haste_fhir_generated_ops::generated::ViewDefinitionRun;
use haste_fhir_model::r4::{
    self,
    generated::{
        resources::{Binary, Resource, ResourceType, ViewDefinition},
        terminology::{BoundCode, IssueType, OutputFormatCodes},
        types::{FHIRBase64Binary, FHIRBoolean},
    },
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhirpath::{Config, FPEngine};
use haste_reflect::MetaValue;
use itertools::Itertools as _;
use ordermap::OrderMap;
use serde::{Deserialize, Serialize};
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
                    IssueType::INVALID,
                    "viewReference.reference is required".to_string(),
                )
            })?
            .value
            .as_ref()
            .ok_or_else(|| {
                OperationOutcomeError::error(
                    IssueType::INVALID,
                    "viewReference.reference.value is required".to_string(),
                )
            })?;

        let reference_pieces = view_definition_reference.split('/').collect::<Vec<_>>();

        let view_definition_id = reference_pieces
            .last()
            .ok_or_else(|| {
                OperationOutcomeError::error(
                    IssueType::INVALID,
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
                    IssueType::NOT_FOUND,
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
                IssueType::INVALID,
                "Referenced resource is not a ViewDefinition".to_string(),
            ))
        }
    } else {
        Err(OperationOutcomeError::error(
            IssueType::INVALID,
            "Either viewResource or viewReference must be provided".to_string(),
        ))
    }
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

        let Some(resource_type) = view_definition.resource.as_str() else {
            return Err(OperationOutcomeError::error(
                IssueType::INVALID,
                "ViewDefinition.resource is required".to_string(),
            ));
        };

        let resource_type = ResourceType::try_from(resource_type).map_err(|e| {
            OperationOutcomeError::error(
                IssueType::INVALID,
                format!("Invalid resource type: {}", e),
            )
        })?;

        let result = client
            .history_type(
                context,
                resource_type,
                vec![
                    ("_since".to_string(), vec![since.to_string()]),
                    ("_count".to_string(), vec!["1000".to_string()]),
                ]
                .into(),
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
    select_statement_results: Vec<Vec<OrderMap<String, OutputResults>>>,
) -> Vec<OrderMap<String, OutputResults>> {
    let mut output_results = Vec::new();

    for combination in select_statement_results
        .into_iter()
        .multi_cartesian_product()
    {
        let mut combined_result = OrderMap::new();

        for result in combination {
            for (key, value) in result {
                combined_result.insert(key, value);
            }
        }

        output_results.push(combined_result);
    }

    output_results
}

// Need to distinguish between a scalar value and a collection of values for each column in the output. This enum helps to represent that distinction.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
enum OutputResults {
    Scalar(Option<PrimitiveValue>),
    Collection(Vec<Option<PrimitiveValue>>),
}

async fn process_resource<
    CTX: Send + Sync + Clone + 'static,
    Client: FHIRClient<CTX, OperationOutcomeError> + Send + Sync + 'static,
>(
    _context: CTX,
    _client: Arc<Client>,
    variables: Arc<HashMap<String, &dyn MetaValue>>,
    view_definition: &ViewDefinition,
    input: Box<Resource>,
) -> Result<Vec<OrderMap<String, OutputResults>>, OperationOutcomeError> {
    let fp_engine = FPEngine::new();

    let mut select_statement_results = Vec::with_capacity(view_definition.select.len());

    for select_statement in view_definition.select.iter() {
        let fp_config = Arc::new(
            Config::builder()
                .with_variable_resolver(haste_fhirpath::ExternalConstantResolver::Variable(
                    variables.clone(),
                ))
                .with_resource_id(input.id().clone().unwrap_or("".to_string())),
        );

        let mut iterable_context = None;
        let mut set_null = false;

        if let Some(for_each_fp) = select_statement
            .forEach
            .as_ref()
            .and_then(|f| f.value.as_ref())
        {
            iterable_context = Some(vec![
                fp_engine
                    .evaluate_with_config(for_each_fp, vec![&input], fp_config.clone())
                    .await
                    .map_err(|e| {
                        OperationOutcomeError::error(
                            IssueType::EXCEPTION,
                            format!("Error evaluating forEach expression: {}", e),
                        )
                    })?,
            ]);
        } else if let Some(for_each_or_null_fp) = select_statement
            .forEachOrNull
            .as_ref()
            .and_then(|f| f.value.as_ref())
        {
            iterable_context = Some(vec![
                fp_engine
                    .evaluate_with_config(for_each_or_null_fp, vec![&input], fp_config.clone())
                    .await
                    .map_err(|e| {
                        OperationOutcomeError::error(
                            IssueType::EXCEPTION,
                            format!("Error evaluating forEachOrNull expression: {}", e),
                        )
                    })?,
            ]);
            set_null = true;
        } else if let Some(_repeat) = select_statement
            .repeat
            .as_ref()
            .map(|r| r.iter().filter_map(|r| r.value.as_ref()))
        {
            let mut repeat_fps = vec![];
            for repeat_fp in _repeat {
                let repeat = format!("$this.repeat({})", repeat_fp);
                repeat_fps.push(
                    fp_engine
                        .evaluate_with_config(&repeat, vec![&input], fp_config.clone())
                        .await
                        .map_err(|e| {
                            OperationOutcomeError::error(
                                IssueType::EXCEPTION,
                                format!("Error evaluating repeat expression: {}", e),
                            )
                        })?,
                );
            }

            iterable_context = Some(repeat_fps);
        }

        let select_context: Vec<&dyn MetaValue> = if let Some(iterable) = iterable_context.as_ref()
        {
            iterable
                .iter()
                .flat_map(|item| item.iter())
                .collect::<Vec<&dyn MetaValue>>()
        } else {
            vec![&input]
        };

        let mut select_results = Vec::with_capacity(select_context.len());

        if set_null && select_context.is_empty() {
            let mut output_result = OrderMap::new();
            for column in select_statement.column.as_ref().into_iter().flatten() {
                let Some(name) = column.name.value.as_ref().map(|n| n.as_str()) else {
                    return Err(OperationOutcomeError::error(
                        IssueType::INVALID,
                        "Column name is required".to_string(),
                    ));
                };
                output_result.insert(name.to_string(), OutputResults::Scalar(None));
            }
            select_results.push(output_result);
        }

        for context in select_context {
            let mut output_result = OrderMap::new();
            for column in select_statement.column.as_ref().into_iter().flatten() {
                let Some(path) = column.path.value.as_ref().map(|p| p.as_str()) else {
                    return Err(OperationOutcomeError::error(
                        IssueType::INVALID,
                        "Column path is required".to_string(),
                    ));
                };

                let Some(name) = column.name.value.as_ref().map(|n| n.as_str()) else {
                    return Err(OperationOutcomeError::error(
                        IssueType::INVALID,
                        "Column name is required".to_string(),
                    ));
                };

                let result = fp_engine
                    .evaluate_with_config(path, vec![context; 1], fp_config.clone())
                    .await
                    .map_err(|e| {
                        OperationOutcomeError::error(
                            IssueType::EXCEPTION,
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

                let mut column_result = result
                    .iter()
                    .map(|value| conversions::primitives::convert_meta_value(column_type, value))
                    .collect::<Result<Vec<Option<PrimitiveValue>>, OperationOutcomeError>>()?;

                let is_collection = column
                    .collection
                    .as_ref()
                    .and_then(|c| c.value)
                    .unwrap_or(false);

                let insert_value = if is_collection {
                    OutputResults::Collection(column_result)
                } else {
                    if column_result.len() > 1 {
                        return Err(OperationOutcomeError::error(
                            IssueType::INVALID,
                            "Column result is a collection but the column is not marked as a collection"
                                .to_string(),
                        ));
                    }

                    let mut singular_value = None;

                    if let Some(first_value) = column_result.get_mut(0) {
                        std::mem::swap(&mut singular_value, first_value);
                    }

                    OutputResults::Scalar(singular_value)
                };

                output_result.insert(name.to_string(), insert_value);
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

async fn passes_where_clauses(
    fp_engine: &FPEngine,
    variables: Arc<HashMap<String, &dyn MetaValue>>,
    where_clauses: &[&str],
    resource: &Resource,
) -> Result<bool, OperationOutcomeError> {
    for where_clause in where_clauses {
        let result = fp_engine
            .evaluate_with_config(
                where_clause,
                vec![resource],
                Arc::new(Config::builder().with_variable_resolver(
                    haste_fhirpath::ExternalConstantResolver::Variable(variables.clone()),
                )),
            )
            .await
            .map_err(|e| {
                OperationOutcomeError::error(
                    IssueType::EXCEPTION,
                    format!("Error evaluating where clause expression: {}", e),
                )
            })?;

        let bool_results = result
            .iter()
            .map(|v| match v.fhir_type() {
                "boolean" => Ok(v
                    .as_any()
                    .downcast_ref::<FHIRBoolean>()
                    .and_then(|b| b.value.as_ref())
                    .unwrap_or(&false)),
                "http://hl7.org/fhirpath/System.Boolean" => {
                    Ok(v.as_any().downcast_ref::<bool>().unwrap_or(&false))
                }
                _ => Err(OperationOutcomeError::error(
                    IssueType::INVALID,
                    format!(
                        "Where clause expression must evaluate to a boolean, got: {}",
                        v.fhir_type()
                    ),
                )),
            })
            .collect::<Result<Vec<_>, _>>()?;

        if bool_results.iter().any(|v| **v == false) {
            return Ok(false);
        }
    }

    Ok(true)
}

async fn process_view_definition<
    CTX: Send + Sync + Clone + 'static,
    Client: FHIRClient<CTX, OperationOutcomeError> + Send + Sync + 'static,
>(
    context: CTX,
    output_format: &BoundCode<OutputFormatCodes>,
    client: Arc<Client>,
    view_definition: &ViewDefinition,
    input: &ViewDefinitionRun::Input,
) -> Result<Binary, OperationOutcomeError> {
    let variables = Arc::new(build_hashmap_fp_variables(view_definition));
    let _limit = input
        ._limit
        .as_ref()
        .and_then(|limit| limit.value.clone())
        .unwrap_or(1000);

    let input_ = flatten_results(
        get_resources_to_process(context.clone(), client.as_ref(), view_definition, input).await?,
    );

    let mut tasks = FuturesOrdered::new();

    let where_clauses = view_definition
        .where_
        .as_ref()
        .map(|w| Cow::Borrowed(w))
        .unwrap_or_else(|| Cow::Owned(vec![]));

    let where_fp_clauses = where_clauses
        .iter()
        .filter_map(|w| w.path.value.as_ref())
        .map(|s| s.as_str())
        .collect::<Vec<_>>();

    for resource in input_ {
        if passes_where_clauses(
            &FPEngine::new(),
            variables.clone(),
            where_fp_clauses.as_slice(),
            resource.as_ref(),
        )
        .await?
        {
            tasks.push_back(async {
                process_resource(
                    context.clone(),
                    client.clone(),
                    variables.clone(),
                    view_definition,
                    resource,
                )
                .await
            });
        }
    }

    let mut results = Vec::with_capacity(tasks.len());

    while let Some(result) = tasks.next().await {
        results.push(result?);
    }

    let results = results.into_iter().flatten().collect::<Vec<_>>();

    match output_format {
        binding if binding == &OutputFormatCodes::CSV => {
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
        binding if binding == &OutputFormatCodes::JSON => {
            let data = output::json::json(results)?;

            let base64_string: String = general_purpose::STANDARD.encode(&data);

            Ok(Binary {
                data: Some(Box::new(FHIRBase64Binary {
                    value: Some(base64_string),
                    ..Default::default()
                })),
                ..Default::default()
            })
        }
        binding if binding == &OutputFormatCodes::NDJSON => {
            let data = output::ndjson::ndjson(results)?;
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
            IssueType::NOT_SUPPORTED,
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
        .and_then(|s| BoundCode::<OutputFormatCodes>::new(s))
        .unwrap_or(OutputFormatCodes::CSV);

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
