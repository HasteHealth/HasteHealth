use base64::{Engine as _, engine::general_purpose};
use chrono::Utc;
use futures::{StreamExt as _, stream::FuturesOrdered};
use haste_fhir_client::FHIRClient;
use haste_fhir_generated_ops::generated::ViewDefinitionRun;
use haste_fhir_model::r4::{
    self,
    generated::{
        resources::{Binary, Bundle, Resource, ResourceType, ViewDefinition, ViewDefinitionSelect},
        terminology::{BoundCode, IssueType, OutputFormatCodes},
        types::{FHIRBase64Binary, FHIRBoolean, Reference},
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

mod compartment;
mod conversions;
mod output;

fn reference_value(reference: &Reference) -> Result<String, OperationOutcomeError> {
    reference
        .reference
        .as_ref()
        .and_then(|r| r.value.clone())
        .ok_or_else(|| {
            OperationOutcomeError::error(
                IssueType::invalid(),
                "Reference.reference is required".to_string(),
            )
        })
}

/// Resolves the `patient` and `group` input parameters into a flat,
/// de-duplicated list of patient reference strings (e.g. `"Patient/123"`).
/// `group` members that aren't Patient references are skipped, since
/// non-Patient compartments `(Practitioner, Device, RelatedPerson)` aren't
/// supported yet.
async fn resolve_patient_references<
    CTX: Send + Sync + Clone + 'static,
    Client: FHIRClient<CTX, OperationOutcomeError> + Send + Sync + 'static,
>(
    context: CTX,
    client: &Client,
    input: &ViewDefinitionRun::Input,
) -> Result<Vec<String>, OperationOutcomeError> {
    let mut references = Vec::new();

    if let Some(patient) = input.patient.as_ref() {
        references.push(reference_value(patient)?);
    }

    for group_reference in input.group.as_ref().into_iter().flatten() {
        let group_reference_value = reference_value(group_reference)?;
        let group_id = group_reference_value
            .rsplit('/')
            .next()
            .unwrap_or(&group_reference_value)
            .to_string();

        let group = client
            .read(context.clone(), ResourceType::Group, group_id.clone())
            .await?
            .ok_or_else(|| {
                OperationOutcomeError::error(
                    IssueType::not_found(),
                    format!("Group not found with id '{group_id}'"),
                )
            })?;

        let Resource::Group(group) = group else {
            return Err(OperationOutcomeError::error(
                IssueType::invalid(),
                format!("Reference '{group_reference_value}' does not point to a Group resource"),
            ));
        };

        for member in group.member.into_iter().flatten() {
            let entity_reference = reference_value(&member.entity)?;
            if entity_reference.starts_with("Patient/") {
                references.push(entity_reference);
            }
        }
    }

    references.sort_unstable();
    references.dedup();

    Ok(references)
}

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
                    IssueType::invalid(),
                    "viewReference.reference is required".to_string(),
                )
            })?
            .value
            .as_ref()
            .ok_or_else(|| {
                OperationOutcomeError::error(
                    IssueType::invalid(),
                    "viewReference.reference.value is required".to_string(),
                )
            })?;

        let reference_pieces = view_definition_reference.split('/').collect::<Vec<_>>();

        let view_definition_id = reference_pieces
            .last()
            .ok_or_else(|| {
                OperationOutcomeError::error(
                    IssueType::invalid(),
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
                    IssueType::not_found(),
                    format!("ViewDefinition not found with id '{view_definition_id:?}'"),
                )
            })?;

        if let Resource::ViewDefinition(view_definition) = result {
            Ok(Cow::Owned(view_definition))
        } else {
            Err(OperationOutcomeError::error(
                IssueType::invalid(),
                "Referenced resource is not a ViewDefinition".to_string(),
            ))
        }
    } else {
        Err(OperationOutcomeError::error(
            IssueType::invalid(),
            "Either viewResource or viewReference must be provided".to_string(),
        ))
    }
}

/// Page size used both for the unfiltered `_since` history scan and for
/// following its pagination to completion.
const HISTORY_PAGE_SIZE: u32 = 1000;

/// Hard cap on the number of resources fetched for a single run when there's
/// no `patient`/`group` scoping to naturally bound the working set (e.g. a
/// view over an entire resource type's history). Without this, a client that
/// omits `_limit` would cause the full history of a resource type to be
/// pulled into memory before any filtering happens. A client that wants more
/// than one page's worth of matching rows should page through results by
/// re-running with `_since` set to the last-seen `lastUpdated` value.
const MAX_RESOURCES_PER_RUN: usize = 50_000;

/// The client-requested `_limit`, if any. Used both to bound how many
/// resources are fetched (see `MAX_RESOURCES_PER_RUN`) and to truncate the
/// final output rows.
fn requested_limit(input: &ViewDefinitionRun::Input) -> Option<usize> {
    input
        .limit
        .as_ref()
        .and_then(|limit| limit.value)
        .and_then(|limit| usize::try_from(limit).ok())
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
        return Ok(input_resources);
    }

    let Some(resource_type) = view_definition.resource.as_str() else {
        return Err(OperationOutcomeError::error(
            IssueType::invalid(),
            "ViewDefinition.resource is required".to_string(),
        ));
    };

    let resource_type = ResourceType::try_from(resource_type).map_err(|e| {
        OperationOutcomeError::error(IssueType::invalid(), format!("Invalid resource type: {e}"))
    })?;

    let since_instant = input.since.as_ref().and_then(|since| since.value.clone());

    let patient_references = resolve_patient_references(context.clone(), client, input).await?;

    if !patient_references.is_empty() {
        let last_updated_filter = since_instant.as_ref().map(|since| format!("gt{since:?}"));

        return compartment::resources_for_patients(
            context,
            client,
            resource_type,
            &patient_references,
            last_updated_filter.as_deref(),
        )
        .await;
    }

    let since = since_instant.unwrap_or(r4::datetime::Instant::Iso8601(Utc::now()));

    let resource_limit = requested_limit(input).map_or(MAX_RESOURCES_PER_RUN, |limit| {
        limit.min(MAX_RESOURCES_PER_RUN)
    });

    let mut combined: Option<Bundle> = None;
    let mut offset = 0u32;
    let mut total_fetched = 0usize;

    loop {
        let mut page = client
            .history_type(
                context.clone(),
                resource_type.clone(),
                vec![
                    ("_since".to_string(), vec![since.to_string()]),
                    ("_count".to_string(), vec![HISTORY_PAGE_SIZE.to_string()]),
                    ("_offset".to_string(), vec![offset.to_string()]),
                ]
                .into(),
            )
            .await?;

        let entry_count = page.entry.as_ref().map_or(0, Vec::len);
        total_fetched += entry_count;

        match combined.as_mut() {
            Some(accumulated) => accumulated
                .entry
                .get_or_insert_with(Vec::new)
                .extend(page.entry.take().into_iter().flatten()),
            None => combined = Some(page),
        }

        if entry_count < HISTORY_PAGE_SIZE as usize || total_fetched >= resource_limit {
            break;
        }

        offset += HISTORY_PAGE_SIZE;
    }

    let mut combined = combined.unwrap_or_default();

    // Guards against the last loop execution which could be an additional
    // 999 resources that may have been fetched in the last loop iteration.
    if let Some(entries) = combined.entry.as_mut() {
        entries.truncate(resource_limit);
    }

    Ok(vec![Resource::Bundle(combined)])
}

fn build_hashmap_fp_variables(viewdefinition: &ViewDefinition) -> HashMap<String, &dyn MetaValue> {
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
    input: Resource,
) -> Result<Vec<OrderMap<String, OutputResults>>, OperationOutcomeError> {
    let fp_engine = FPEngine::new();

    let mut select_statement_results = Vec::with_capacity(view_definition.select.len());

    for select_statement in &view_definition.select {
        let fp_config = Arc::new(
            Config::builder()
                .with_variable_resolver(haste_fhirpath::ExternalConstantResolver::Variable(
                    variables.clone(),
                ))
                .with_resource_id(input.id().clone().unwrap_or_default()),
        );

        let (iterable_context, set_null) =
            build_iterable_context(&fp_engine, fp_config.clone(), select_statement, &input).await?;

        let select_results = process_select_statement(
            &fp_engine,
            fp_config,
            select_statement,
            &input,
            iterable_context,
            set_null,
        )
        .await?;

        select_statement_results.push(select_results);
    }

    let output_results = cartesian_product(select_statement_results);

    Ok(output_results)
}

async fn build_iterable_context<'a>(
    fp_engine: &FPEngine,
    fp_config: Arc<Config<'a>>,
    select_statement: &'a ViewDefinitionSelect,
    input: &'a Resource,
) -> Result<(Option<Vec<haste_fhirpath::Context<'a>>>, bool), OperationOutcomeError> {
    let mut iterable_context = None;
    let mut set_null = false;

    if let Some(for_each_fp) = select_statement
        .forEach
        .as_ref()
        .and_then(|f| f.value.as_ref())
    {
        iterable_context = Some(vec![
            fp_engine
                .evaluate_with_config(for_each_fp, vec![input], fp_config.clone())
                .await
                .map_err(|e| {
                    OperationOutcomeError::error(
                        IssueType::exception(),
                        format!("Error evaluating forEach expression: {e}"),
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
                .evaluate_with_config(for_each_or_null_fp, vec![input], fp_config.clone())
                .await
                .map_err(|e| {
                    OperationOutcomeError::error(
                        IssueType::exception(),
                        format!("Error evaluating forEachOrNull expression: {e}"),
                    )
                })?,
        ]);

        set_null = true;
    } else if let Some(repeat) = select_statement
        .repeat
        .as_ref()
        .map(|r| r.iter().filter_map(|r| r.value.as_ref()))
    {
        let mut repeat_fps = vec![];

        for repeat_fp in repeat {
            let repeat = format!("$this.repeat({repeat_fp})");

            repeat_fps.push(
                fp_engine
                    .evaluate_with_config(&repeat, vec![input], fp_config.clone())
                    .await
                    .map_err(|e| {
                        OperationOutcomeError::error(
                            IssueType::exception(),
                            format!("Error evaluating repeat expression: {e}"),
                        )
                    })?,
            );
        }

        iterable_context = Some(repeat_fps);
    }

    Ok((iterable_context, set_null))
}

async fn process_select_statement<'a>(
    fp_engine: &FPEngine,
    fp_config: Arc<Config<'a>>,
    select_statement: &'a ViewDefinitionSelect,
    input: &'a Resource,
    iterable_context: Option<Vec<haste_fhirpath::Context<'a>>>,
    set_null: bool,
) -> Result<Vec<OrderMap<String, OutputResults>>, OperationOutcomeError> {
    let select_context: Vec<&dyn MetaValue> = if let Some(iterable) = iterable_context.as_ref() {
        iterable
            .iter()
            .flat_map(haste_fhirpath::Context::iter)
            .collect()
    } else {
        vec![input]
    };

    let mut select_results = Vec::with_capacity(select_context.len());

    if set_null && select_context.is_empty() {
        let output_result = build_null_result(select_statement)?;
        select_results.push(output_result);
    }

    for context in select_context {
        let output_result =
            process_select_context(fp_engine, fp_config.clone(), select_statement, context).await?;

        select_results.push(output_result);
    }

    Ok(select_results)
}

fn build_null_result(
    select_statement: &ViewDefinitionSelect,
) -> Result<OrderMap<String, OutputResults>, OperationOutcomeError> {
    let mut output_result = OrderMap::new();

    for column in select_statement.column.as_ref().into_iter().flatten() {
        let Some(name) = column.name.value.as_deref() else {
            return Err(OperationOutcomeError::error(
                IssueType::invalid(),
                "Column name is required".to_string(),
            ));
        };

        output_result.insert(name.to_string(), OutputResults::Scalar(None));
    }

    Ok(output_result)
}

async fn process_select_context<'a>(
    fp_engine: &FPEngine,
    fp_config: Arc<Config<'a>>,
    select_statement: &'a ViewDefinitionSelect,
    context: &'a dyn MetaValue,
) -> Result<OrderMap<String, OutputResults>, OperationOutcomeError> {
    let mut output_result = OrderMap::new();

    for column in select_statement.column.as_ref().into_iter().flatten() {
        let Some(path) = column.path.value.as_deref() else {
            return Err(OperationOutcomeError::error(
                IssueType::invalid(),
                "Column path is required".to_string(),
            ));
        };

        let Some(name) = column.name.value.as_deref() else {
            return Err(OperationOutcomeError::error(
                IssueType::invalid(),
                "Column name is required".to_string(),
            ));
        };

        let result = fp_engine
            .evaluate_with_config(path, vec![context], fp_config.clone())
            .await
            .map_err(|e| {
                OperationOutcomeError::error(
                    IssueType::exception(),
                    format!("Error evaluating expression: {e}"),
                )
            })?;

        let column_type = column
            .type_
            .as_ref()
            .and_then(|t| t.value.as_deref())
            .unwrap_or_else(|| {
                result
                    .iter()
                    .next()
                    .map_or("string", haste_reflect::MetaValue::fhir_type)
            });

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
                    IssueType::invalid(),
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

    Ok(output_result)
}

fn flatten_results(resource: Vec<Resource>) -> Vec<Resource> {
    let mut resources = Vec::new();
    for resource in resource {
        match resource {
            Resource::Bundle(bundle) => {
                for entry in bundle.entry.into_iter().flatten() {
                    if let Some(resource) = entry.resource {
                        resources.push(*resource);
                    }
                }
            }
            _ => {
                resources.push(resource);
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
                    IssueType::exception(),
                    format!("Error evaluating where clause expression: {e}"),
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
                    IssueType::invalid(),
                    format!(
                        "Where clause expression must evaluate to a boolean, got: {}",
                        v.fhir_type()
                    ),
                )),
            })
            .collect::<Result<Vec<_>, _>>()?;

        if bool_results.iter().any(|v| !**v) {
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
    let limit = requested_limit(input);

    let input_ = flatten_results(
        get_resources_to_process(context.clone(), client.as_ref(), view_definition, input).await?,
    );

    let mut tasks = FuturesOrdered::new();

    let where_clauses = view_definition
        .where_
        .as_ref()
        .map_or_else(|| Cow::Owned(Vec::new()), Cow::Borrowed);

    let where_fp_clauses = where_clauses
        .iter()
        .filter_map(|w| w.path.value.as_deref())
        .collect::<Vec<_>>();

    for resource in input_ {
        if passes_where_clauses(
            &FPEngine::new(),
            variables.clone(),
            where_fp_clauses.as_slice(),
            &resource,
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

    let mut results = results.into_iter().flatten().collect::<Vec<_>>();

    if let Some(limit) = limit {
        results.truncate(limit);
    }

    let include_header = input
        .header
        .as_ref()
        .and_then(|header| header.value)
        .unwrap_or(true);

    match output_format {
        binding if binding == &OutputFormatCodes::csv() => {
            let data = output::csv::csv(&results, include_header)?;

            let base64_string: String = general_purpose::STANDARD.encode(&data);

            Ok(Binary {
                data: Some(Box::new(FHIRBase64Binary {
                    value: Some(base64_string),
                    ..Default::default()
                })),
                ..Default::default()
            })
        }
        binding if binding == &OutputFormatCodes::json() => {
            let data = output::json::json(&results)?;

            let base64_string: String = general_purpose::STANDARD.encode(&data);

            Ok(Binary {
                data: Some(Box::new(FHIRBase64Binary {
                    value: Some(base64_string),
                    ..Default::default()
                })),
                ..Default::default()
            })
        }
        binding if binding == &OutputFormatCodes::ndjson() => {
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
            IssueType::not_supported(),
            format!("Output format '{output_format:?}' is not supported"),
        )),
    }
}

/// Runs a FHIR `ViewDefinition` and returns the generated result in the requested
/// output format.
///
/// The output format is read from the `_format` input parameter. If `_format` is
/// absent or does not contain a recognized [`OutputFormatCodes`] value, CSV is
/// used as the default format.
///
/// # Errors
///
/// Returns [`OperationOutcomeError`] if the view definition cannot be resolved
/// or if processing the view definition fails.
pub async fn view_definition_run<
    CTX: Send + Sync + Clone + 'static,
    Client: FHIRClient<CTX, OperationOutcomeError> + Send + Sync + 'static,
>(
    context: CTX,
    client: Arc<Client>,
    input: &ViewDefinitionRun::Input,
) -> Result<ViewDefinitionRun::Output, OperationOutcomeError> {
    let output_format = input
        .format
        .as_ref()
        .and_then(|v| v.value.as_ref())
        .and_then(|s| BoundCode::<OutputFormatCodes>::new(s))
        .unwrap_or(OutputFormatCodes::csv());

    let view_definition =
        Arc::new(resolve_view_definition(context.clone(), client.as_ref(), input).await?);

    let output = process_view_definition(
        context,
        &output_format,
        client,
        view_definition.as_ref(),
        input,
    )
    .await?;

    Ok(ViewDefinitionRun::Output { return_: output })
}
