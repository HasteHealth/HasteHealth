//! Batch indexing into the PG search tables.
//!
//! Indexing runs in two phases, which is what keeps a batch from degenerating
//! into per-row round-trips:
//!
//! 1. **Convert** — every resource's FHIRPath expressions are evaluated and
//!    flattened into the exact rows it will occupy, in parallel across the
//!    runtime and with no database involved. Everything that can fail for a
//!    single resource fails here, where it can be attributed to that resource
//!    without touching the batch around it.
//! 2. **Write** — the converted rows are written set-based: one statement per
//!    table for the whole batch rather than one per row. A thousand-resource
//!    batch is roughly fifteen statements, not ten thousand.
//!
//! The split is also what makes the single transaction correct. Postgres
//! aborts a transaction on the first failed statement, so per-resource error
//! recovery *inside* one is not possible — a failure in phase 2 fails the
//! whole batch, and the worker retries it.

use std::collections::HashMap;
use std::fmt::Write as _;
use std::sync::Arc;

use haste_fhir_model::r4::generated::terminology::IssueType;
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhirpath::FPEngine;
use haste_repository::types::FHIRMethod;
use sqlx::{Pool, Postgres};

use super::{
    PgSearchError, resource_to_search_index,
    schema::{ColumnType, ParamColumns, ResourceTypeSchema, SchemaRegistry},
};
use crate::{
    IndexFailure, IndexOutcome, IndexResource, ResolvedParameter, SearchParameterResolve,
    indexing_conversion::InsertableIndex,
};

/// Postgres caps one extended-query message at 65535 bound parameters. Only
/// the per-resource-type insert can approach it: every other statement binds a
/// fixed number of arrays regardless of how many rows it writes.
const MAX_BIND_PARAMS: usize = 65535;

/// `tenant, project, resource_id, version_id` — the columns every
/// per-resource-type table carries ahead of its generated value columns.
const SYSTEM_FIXED_COLUMNS: usize = 4;

pub async fn index_resources<Resolver: SearchParameterResolve + 'static>(
    pool: &Pool<Postgres>,
    parameter_resolver: &Arc<Resolver>,
    schema_registry: &Arc<SchemaRegistry>,
    fp_engine: Arc<FPEngine>,
    resources: Vec<IndexResource>,
) -> Result<IndexOutcome, OperationOutcomeError> {
    let total = resources.len();
    tracing::trace!("PG search: indexing {} resources", total);

    if resources.is_empty() {
        return Ok(IndexOutcome {
            succeeded: 0,
            failed: Vec::new(),
        });
    }

    // A batch can carry several versions of the same resource (a create and a
    // later update land in the same sequence window). Writing them set-based
    // would collide on `search_resource`'s primary key, and only the final
    // state belongs in the index anyway, so earlier versions are set aside
    // here and accounted for alongside the version that supersedes them.
    let (resources, superseded) = dedupe_keep_last(resources);

    // Resolving parameters awaits, and on a cold project cache it queries the
    // database. Doing it once per (tenant, project, resource type) group up
    // front keeps that off the per-resource path entirely.
    let parameter_sets = resolve_parameter_sets(parameter_resolver.as_ref(), &resources).await?;

    let (converted, failed) = convert_batch(
        fp_engine,
        schema_registry.clone(),
        &parameter_sets,
        resources,
        superseded,
    )
    .await?;

    let succeeded = total - failed.len();

    if !failed.is_empty() {
        tracing::error!(
            "PG search: {} failed item(s) out of {}.",
            failed.len(),
            total
        );
    }

    write_batch(pool, schema_registry, converted).await?;

    Ok(IndexOutcome { succeeded, failed })
}

// ---------------------------------------------------------------------------
// Identity
// ---------------------------------------------------------------------------

/// The identity of one indexed resource — the composite key every search
/// table is keyed by.
#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct ResourceKey {
    tenant: String,
    project: String,
    resource_type: String,
    resource_id: String,
}

impl ResourceKey {
    fn from_resource(resource: &IndexResource) -> Self {
        ResourceKey {
            tenant: resource.tenant.as_ref().to_string(),
            project: resource.project.as_ref().to_string(),
            resource_type: resource.resource_type.as_ref().to_string(),
            resource_id: resource.id.as_ref().to_string(),
        }
    }
}

/// Keeps only the last occurrence of each resource, returning the versions it
/// displaced so they can share their survivor's outcome.
fn dedupe_keep_last(
    resources: Vec<IndexResource>,
) -> (Vec<IndexResource>, HashMap<ResourceKey, Vec<IndexResource>>) {
    let mut position: HashMap<ResourceKey, usize> = HashMap::new();
    let mut kept: Vec<IndexResource> = Vec::with_capacity(resources.len());
    let mut superseded: HashMap<ResourceKey, Vec<IndexResource>> = HashMap::new();

    for resource in resources {
        let key = ResourceKey::from_resource(&resource);

        if let Some(&index) = position.get(&key) {
            let previous = std::mem::replace(&mut kept[index], resource);
            superseded.entry(key).or_default().push(previous);
        } else {
            position.insert(key, kept.len());
            kept.push(resource);
        }
    }

    (kept, superseded)
}

// ---------------------------------------------------------------------------
// Phase 1: convert
// ---------------------------------------------------------------------------

/// One resource's rows, ready to bind. Everything fallible has already
/// happened by the time this exists.
struct Converted {
    key: ResourceKey,
    version_id: String,
    /// `None` for a delete, whose row set is just the anchor removal.
    write: Option<ConvertedWrite>,
}

struct ConvertedWrite {
    /// One slot per column in the resource type's schema, `None` where this
    /// resource produced no value for that column. `None` overall when the
    /// resource type has no generated table.
    system_row: Option<Vec<Option<ColumnValues>>>,
    dynamic: DynamicRows,
}

/// One column's worth of values, ready to bind as a PostgreSQL array.
///
/// `Text` carries `Option<String>` because a token's system (and a
/// reference's type) is genuinely absent for some values, and the NULL has to
/// stay in place so the parallel arrays keep lining up.
enum ColumnValues {
    Text(Vec<Option<String>>),
    BigInt(Vec<i64>),
    Double(Vec<f64>),
}

/// The parameter set for one (tenant, project, resource type), plus the
/// `code` → canonical URL lookup the reference mirror needs. Both halves are
/// behind an `Arc` so a task takes a handle rather than a copy.
#[derive(Clone)]
struct ParameterSet {
    parameters: Arc<Vec<ResolvedParameter>>,
    url_by_code: Arc<HashMap<String, String>>,
}

/// Resolves parameters once per distinct (tenant, project, resource type) in
/// the batch rather than once per resource.
async fn resolve_parameter_sets<Resolver: SearchParameterResolve>(
    resolver: &Resolver,
    resources: &[IndexResource],
) -> Result<HashMap<(String, String, String), ParameterSet>, OperationOutcomeError> {
    let mut sets: HashMap<(String, String, String), ParameterSet> = HashMap::new();

    for resource in resources {
        // A delete never evaluates an expression, so resolving for it would
        // only risk a database round-trip that nothing reads.
        if matches!(resource.fhir_method, FHIRMethod::Delete) {
            continue;
        }

        let group = (
            resource.tenant.as_ref().to_string(),
            resource.project.as_ref().to_string(),
            resource.resource_type.as_ref().to_string(),
        );

        if sets.contains_key(&group) {
            continue;
        }

        let parameters = resolver
            .by_resource_type(&resource.tenant, &resource.project, &resource.resource_type)
            .await?;

        let url_by_code = parameters
            .iter()
            .filter_map(|p| {
                let code = p.search_parameter.code.value.as_deref()?;
                let url = p.search_parameter.url.value.as_deref()?;
                Some((code.to_string(), url.to_string()))
            })
            .collect();

        sets.insert(
            group,
            ParameterSet {
                parameters: Arc::new(parameters),
                url_by_code: Arc::new(url_by_code),
            },
        );
    }

    Ok(sets)
}

/// Converts every resource in parallel, keeping a single resource's failure
/// (a bad FHIRPath expression, an unsupported method) attributed to it rather
/// than aborting the batch.
///
/// A `JoinError` — the task itself panicked — has no resource to attribute the
/// failure to, so it aborts the whole call.
async fn convert_batch(
    fp_engine: Arc<FPEngine>,
    schema_registry: Arc<SchemaRegistry>,
    parameter_sets: &HashMap<(String, String, String), ParameterSet>,
    resources: Vec<IndexResource>,
    mut superseded: HashMap<ResourceKey, Vec<IndexResource>>,
) -> Result<(Vec<Converted>, Vec<IndexFailure>), OperationOutcomeError> {
    let tasks: Vec<_> = resources
        .into_iter()
        .map(|resource| {
            let fp_engine = fp_engine.clone();
            let schema_registry = schema_registry.clone();
            let set = parameter_sets
                .get(&(
                    resource.tenant.as_ref().to_string(),
                    resource.project.as_ref().to_string(),
                    resource.resource_type.as_ref().to_string(),
                ))
                .cloned();

            tokio::spawn(async move {
                let result =
                    convert_resource(&fp_engine, &schema_registry, set.as_ref(), &resource).await;
                (resource, result)
            })
        })
        .collect();

    let mut converted = Vec::with_capacity(tasks.len());
    let mut failed = Vec::new();

    for task in tasks {
        let (resource, result) = task.await.map_err(|e| {
            OperationOutcomeError::fatal(
                IssueType::exception(),
                format!("PG search indexing task panicked: {e}"),
            )
        })?;

        let key = ResourceKey::from_resource(&resource);
        let displaced = superseded.remove(&key).unwrap_or_default();

        match result {
            Ok(entry) => {
                // The displaced versions are superseded by one that indexed
                // cleanly, so the resource reached its correct final state and
                // every version of it counts as handled.
                converted.push(entry);
                drop(displaced);
            }
            Err(error) => {
                for stale in displaced {
                    failed.push(IndexFailure {
                        resource: stale,
                        error: OperationOutcomeError::fatal(
                            IssueType::exception(),
                            "Superseded by a later version in the same batch that failed to index."
                                .to_string(),
                        ),
                    });
                }
                failed.push(IndexFailure { resource, error });
            }
        }
    }

    Ok((converted, failed))
}

async fn convert_resource(
    fp_engine: &Arc<FPEngine>,
    schema_registry: &SchemaRegistry,
    parameter_set: Option<&ParameterSet>,
    resource: &IndexResource,
) -> Result<Converted, OperationOutcomeError> {
    let key = ResourceKey::from_resource(resource);
    let version_id = resource.version_id.as_ref().to_string();

    match &resource.fhir_method {
        FHIRMethod::Create | FHIRMethod::Update => {
            let Some(ParameterSet {
                parameters,
                url_by_code,
            }) = parameter_set
            else {
                return Err(OperationOutcomeError::fatal(
                    IssueType::exception(),
                    format!(
                        "No search parameters resolved for resource type '{}'.",
                        key.resource_type
                    ),
                ));
            };

            let index =
                resource_to_search_index(fp_engine.clone(), parameters, &resource.resource).await?;

            let schema = schema_registry.get(&key.resource_type);

            // System-level parameters go into dedicated columns on the
            // per-resource-type table, one row per resource with an array per
            // column.
            let system_row = schema.map(|schema| {
                let mut slots: Vec<Option<ColumnValues>> =
                    (0..schema.columns.len()).map(|_| None).collect();

                for (code, insertable) in &index.system_entries {
                    // A parameter with no generated column (an unmapped type,
                    // or one that lost a column-name collision) simply has
                    // nowhere to go here.
                    let Some(param_columns) = schema.columns_for(code) else {
                        continue;
                    };

                    collect_column_values(schema, param_columns, insertable, &mut slots);
                }

                slots
            });

            let mut dynamic = DynamicRows::default();

            // Project-level parameters go into the shared EAV tables.
            for (param_url, insertable) in &index.dynamic_entries {
                dynamic.collect(param_url, insertable);
            }

            // Reverse-reference lookups (`_revinclude`, chained search) scan
            // references across resource types, which the per-type columns
            // can't serve. Mirroring system-level references into the shared
            // reference table keeps one place to scan.
            for (code, insertable) in &index.system_entries {
                let InsertableIndex::Reference(references) = insertable else {
                    continue;
                };

                // The mirror is keyed by URL like every other row in that
                // table, so the code has to be resolved back to the parameter
                // it came from.
                let Some(param_url) = url_by_code.get(code) else {
                    continue;
                };

                for reference in references {
                    dynamic.push_reference(param_url, reference);
                }
            }

            Ok(Converted {
                key,
                version_id,
                write: Some(ConvertedWrite {
                    system_row,
                    dynamic,
                }),
            })
        }

        FHIRMethod::Delete => Ok(Converted {
            key,
            version_id,
            write: None,
        }),

        method @ FHIRMethod::Read => Err(OperationOutcomeError::from(
            PgSearchError::UnsupportedFHIRMethod((*method).clone()),
        )),
    }
}

/// Fills the slots for one parameter's columns.
///
/// Multi-part types (token, date, reference, quantity) are written as
/// *parallel* arrays: position `i` of each column belongs to the same logical
/// value, which is what lets queries recombine them with `unnest(a, b)`.
///
/// A mismatch between the parameter's declared type and the evaluated value's
/// type (which would mean the schema and the conversion disagree) writes
/// nothing rather than a half-filled set of parallel arrays.
fn collect_column_values(
    schema: &ResourceTypeSchema,
    param_columns: &ParamColumns,
    insertable: &InsertableIndex,
    slots: &mut [Option<ColumnValues>],
) {
    let mut set = |name: &str, values: ColumnValues| {
        if let Some(index) = schema.column_index(name) {
            slots[index] = Some(values);
        }
    };

    match (param_columns, insertable) {
        (ParamColumns::String { value: column }, InsertableIndex::String(strings)) => {
            set(
                column,
                ColumnValues::Text(strings.iter().map(|s| Some(s.clone())).collect()),
            );
        }

        (ParamColumns::Uri { value: column }, InsertableIndex::URI(uris)) => {
            set(
                column,
                ColumnValues::Text(uris.iter().map(|u| Some(u.clone())).collect()),
            );
        }

        (ParamColumns::Number { value: column }, InsertableIndex::Number(numbers)) => {
            set(column, ColumnValues::Double(numbers.clone()));
        }

        (ParamColumns::Token { system, code }, InsertableIndex::Token(tokens)) => {
            set(
                system,
                ColumnValues::Text(
                    tokens
                        .iter()
                        .map(|t| t.system().map(str::to_string))
                        .collect(),
                ),
            );
            set(
                code,
                ColumnValues::Text(
                    tokens
                        .iter()
                        .map(|t| t.code().map(str::to_string))
                        .collect(),
                ),
            );
        }

        (ParamColumns::Date { start, end }, InsertableIndex::Date(ranges)) => {
            set(
                start,
                ColumnValues::BigInt(ranges.iter().map(|d| d.start).collect()),
            );
            set(
                end,
                ColumnValues::BigInt(ranges.iter().map(|d| d.end).collect()),
            );
        }

        (
            ParamColumns::Reference {
                target_type,
                target_id,
            },
            InsertableIndex::Reference(references),
        ) => {
            set(
                target_type,
                ColumnValues::Text(
                    references
                        .iter()
                        .map(|r| r.resource_type().map(str::to_string))
                        .collect(),
                ),
            );
            set(
                target_id,
                ColumnValues::Text(
                    references
                        .iter()
                        .map(|r| r.id().map(str::to_string))
                        .collect(),
                ),
            );
        }

        (
            ParamColumns::Quantity {
                start,
                end,
                system,
                code,
            },
            InsertableIndex::Quantity(quantities),
        ) => {
            set(
                start,
                ColumnValues::Double(
                    quantities
                        .iter()
                        .map(crate::indexing_conversion::QuantityRange::start_value)
                        .collect(),
                ),
            );
            set(
                end,
                ColumnValues::Double(
                    quantities
                        .iter()
                        .map(crate::indexing_conversion::QuantityRange::end_value)
                        .collect(),
                ),
            );
            set(
                system,
                ColumnValues::Text(
                    quantities
                        .iter()
                        .map(|q| q.start_system().map(str::to_string))
                        .collect(),
                ),
            );
            set(
                code,
                ColumnValues::Text(
                    quantities
                        .iter()
                        .map(|q| q.start_code().map(str::to_string))
                        .collect(),
                ),
            );
        }

        _ => {}
    }
}

// ---------------------------------------------------------------------------
// Flattened `search_dynamic_*` rows
// ---------------------------------------------------------------------------

struct StringRow {
    param_url: String,
    value: String,
}

struct NumberRow {
    param_url: String,
    value: f64,
}

struct TokenRow {
    param_url: String,
    system: Option<String>,
    code: Option<String>,
}

struct DateRow {
    param_url: String,
    start_ms: i64,
    end_ms: i64,
}

struct ReferenceRow {
    param_url: String,
    target_resource_type: Option<String>,
    target_id: Option<String>,
    target_uri: Option<String>,
}

struct QuantityRow {
    param_url: String,
    start_value: f64,
    start_system: Option<String>,
    start_code: Option<String>,
    end_value: f64,
    end_system: Option<String>,
    end_code: Option<String>,
}

/// One resource's rows across the shared EAV tables.
#[derive(Default)]
struct DynamicRows {
    string: Vec<StringRow>,
    uri: Vec<StringRow>,
    number: Vec<NumberRow>,
    token: Vec<TokenRow>,
    date: Vec<DateRow>,
    reference: Vec<ReferenceRow>,
    quantity: Vec<QuantityRow>,
}

impl DynamicRows {
    fn collect(&mut self, param_url: &str, insertable: &InsertableIndex) {
        match insertable {
            InsertableIndex::String(values) => {
                for value in values {
                    self.string.push(StringRow {
                        param_url: param_url.to_string(),
                        value: value.clone(),
                    });
                }
            }

            InsertableIndex::URI(values) => {
                for value in values {
                    self.uri.push(StringRow {
                        param_url: param_url.to_string(),
                        value: value.clone(),
                    });
                }
            }

            InsertableIndex::Number(values) => {
                for value in values {
                    self.number.push(NumberRow {
                        param_url: param_url.to_string(),
                        value: *value,
                    });
                }
            }

            InsertableIndex::Token(tokens) => {
                for token in tokens {
                    self.token.push(TokenRow {
                        param_url: param_url.to_string(),
                        system: token.system().map(str::to_string),
                        code: token.code().map(str::to_string),
                    });
                }
            }

            InsertableIndex::Date(ranges) => {
                for range in ranges {
                    self.date.push(DateRow {
                        param_url: param_url.to_string(),
                        start_ms: range.start,
                        end_ms: range.end,
                    });
                }
            }

            InsertableIndex::Reference(references) => {
                for reference in references {
                    self.push_reference(param_url, reference);
                }
            }

            InsertableIndex::Quantity(quantities) => {
                for quantity in quantities {
                    self.quantity.push(QuantityRow {
                        param_url: param_url.to_string(),
                        start_value: quantity.start_value(),
                        start_system: quantity.start_system().map(str::to_string),
                        start_code: quantity.start_code().map(str::to_string),
                        end_value: quantity.end_value(),
                        end_system: quantity.end_system().map(str::to_string),
                        end_code: quantity.end_code().map(str::to_string),
                    });
                }
            }

            // A project-level parameter carries its own URL and a value slot
            // per type, so each entry routes independently.
            InsertableIndex::DynamicParameters(entries) => {
                for entry in entries {
                    self.collect_dynamic_parameter(entry);
                }
            }

            // Composite and Special have no PG representation yet; Meta is
            // internal to the Elasticsearch document shape.
            InsertableIndex::Composite(_)
            | InsertableIndex::Special(_)
            | InsertableIndex::Meta(_) => {}
        }
    }

    fn collect_dynamic_parameter(
        &mut self,
        entry: &crate::indexing_conversion::DynamicParameterEntry,
    ) {
        let url = entry.url.as_str();

        if let Some(values) = &entry.value.string {
            self.collect(url, &InsertableIndex::String(values.clone()));
        }
        if let Some(values) = &entry.value.uri {
            self.collect(url, &InsertableIndex::URI(values.clone()));
        }
        if let Some(values) = &entry.value.number {
            self.collect(url, &InsertableIndex::Number(values.clone()));
        }
        if let Some(tokens) = &entry.value.token {
            for token in tokens {
                self.token.push(TokenRow {
                    param_url: url.to_string(),
                    system: token.system().map(str::to_string),
                    code: token.code().map(str::to_string),
                });
            }
        }
        if let Some(dates) = &entry.value.date {
            for date in dates {
                self.date.push(DateRow {
                    param_url: url.to_string(),
                    start_ms: date.start,
                    end_ms: date.end,
                });
            }
        }
        if let Some(references) = &entry.value.reference {
            for reference in references {
                self.push_reference(url, reference);
            }
        }
        if let Some(quantities) = &entry.value.quantity {
            for quantity in quantities {
                self.quantity.push(QuantityRow {
                    param_url: url.to_string(),
                    start_value: quantity.start_value(),
                    start_system: quantity.start_system().map(str::to_string),
                    start_code: quantity.start_code().map(str::to_string),
                    end_value: quantity.end_value(),
                    end_system: quantity.end_system().map(str::to_string),
                    end_code: quantity.end_code().map(str::to_string),
                });
            }
        }
    }

    fn push_reference(
        &mut self,
        param_url: &str,
        reference: &crate::indexing_conversion::ReferenceIndex,
    ) {
        self.reference.push(ReferenceRow {
            param_url: param_url.to_string(),
            target_resource_type: reference.resource_type().map(str::to_string),
            target_id: reference.id().map(str::to_string),
            target_uri: reference.uri().map(str::to_string),
        });
    }
}

// ---------------------------------------------------------------------------
// Phase 2: write
// ---------------------------------------------------------------------------

/// The (tenant, project, resource_type, resource_id, param_url) prefix every
/// `search_dynamic_*` row carries, accumulated across the batch as one array
/// per column so it can be bound to a single `unnest`.
#[derive(Default)]
struct KeyColumns {
    tenant: Vec<String>,
    project: Vec<String>,
    resource_type: Vec<String>,
    resource_id: Vec<String>,
    param_url: Vec<String>,
}

impl KeyColumns {
    fn push(&mut self, key: &ResourceKey, param_url: String) {
        self.tenant.push(key.tenant.clone());
        self.project.push(key.project.clone());
        self.resource_type.push(key.resource_type.clone());
        self.resource_id.push(key.resource_id.clone());
        self.param_url.push(param_url);
    }

    fn is_empty(&self) -> bool {
        self.tenant.is_empty()
    }
}

/// Every `search_dynamic_*` row in the batch, transposed into the column
/// arrays each table's insert binds.
#[derive(Default)]
struct DynamicBatch {
    string_keys: KeyColumns,
    string_value: Vec<String>,

    uri_keys: KeyColumns,
    uri_value: Vec<String>,

    number_keys: KeyColumns,
    number_value: Vec<f64>,

    token_keys: KeyColumns,
    token_system: Vec<Option<String>>,
    token_code: Vec<Option<String>>,

    date_keys: KeyColumns,
    date_start: Vec<i64>,
    date_end: Vec<i64>,

    reference_keys: KeyColumns,
    reference_target_type: Vec<Option<String>>,
    reference_target_id: Vec<Option<String>>,
    reference_target_uri: Vec<Option<String>>,

    quantity_keys: KeyColumns,
    quantity_start_value: Vec<f64>,
    quantity_start_system: Vec<Option<String>>,
    quantity_start_code: Vec<Option<String>>,
    quantity_end_value: Vec<f64>,
    quantity_end_system: Vec<Option<String>>,
    quantity_end_code: Vec<Option<String>>,
}

impl DynamicBatch {
    fn extend(&mut self, key: &ResourceKey, rows: DynamicRows) {
        for row in rows.string {
            self.string_keys.push(key, row.param_url);
            self.string_value.push(row.value);
        }
        for row in rows.uri {
            self.uri_keys.push(key, row.param_url);
            self.uri_value.push(row.value);
        }
        for row in rows.number {
            self.number_keys.push(key, row.param_url);
            self.number_value.push(row.value);
        }
        for row in rows.token {
            self.token_keys.push(key, row.param_url);
            self.token_system.push(row.system);
            self.token_code.push(row.code);
        }
        for row in rows.date {
            self.date_keys.push(key, row.param_url);
            self.date_start.push(row.start_ms);
            self.date_end.push(row.end_ms);
        }
        for row in rows.reference {
            self.reference_keys.push(key, row.param_url);
            self.reference_target_type.push(row.target_resource_type);
            self.reference_target_id.push(row.target_id);
            self.reference_target_uri.push(row.target_uri);
        }
        for row in rows.quantity {
            self.quantity_keys.push(key, row.param_url);
            self.quantity_start_value.push(row.start_value);
            self.quantity_start_system.push(row.start_system);
            self.quantity_start_code.push(row.start_code);
            self.quantity_end_value.push(row.end_value);
            self.quantity_end_system.push(row.end_system);
            self.quantity_end_code.push(row.end_code);
        }
    }
}

/// One resource's row on its per-resource-type table.
struct SystemRow<'a> {
    key: &'a ResourceKey,
    version_id: &'a str,
    slots: &'a [Option<ColumnValues>],
}

/// Writes the whole converted batch in one transaction.
///
/// The transaction is all-or-nothing by necessity: Postgres aborts it on the
/// first failed statement, so there is no per-resource recovery to be had
/// here. A failure returns `Err`, the worker leaves its sequence lock where it
/// was, and the batch is retried.
async fn write_batch(
    pool: &Pool<Postgres>,
    schema_registry: &SchemaRegistry,
    mut converted: Vec<Converted>,
) -> Result<(), OperationOutcomeError> {
    if converted.is_empty() {
        return Ok(());
    }

    // Two batches that touch overlapping resources would otherwise take the
    // same row locks in different orders. Sorting gives every writer one
    // order, which is what keeps them queuing rather than deadlocking.
    converted.sort_by(|a, b| a.key.cmp(&b.key));

    let mut tx = pool.begin().await.map_err(|e| {
        OperationOutcomeError::fatal(
            IssueType::exception(),
            format!("Failed to begin PG search transaction: {e}"),
        )
    })?;

    // Clears the previous version of every resource in the batch — the
    // re-indexed ones and the deleted ones alike — before re-inserting.
    delete_previous_versions(tx.as_mut(), schema_registry, &converted).await?;

    insert_anchors(tx.as_mut(), &converted).await?;

    // Grouped by resource type: each per-resource-type table has its own
    // column list, so it needs its own statement.
    let mut system_rows: HashMap<&str, Vec<SystemRow<'_>>> = HashMap::new();
    let mut dynamic = DynamicBatch::default();

    for entry in &converted {
        let Some(write) = &entry.write else {
            continue;
        };

        if let Some(slots) = &write.system_row {
            system_rows
                .entry(entry.key.resource_type.as_str())
                .or_default()
                .push(SystemRow {
                    key: &entry.key,
                    version_id: &entry.version_id,
                    slots,
                });
        }
    }

    for (resource_type, rows) in &system_rows {
        let Some(schema) = schema_registry.get(resource_type) else {
            continue;
        };
        insert_system_rows(tx.as_mut(), schema, rows).await?;
    }

    // The dynamic rows are moved out of `converted`, so this has to come after
    // everything that borrows from it.
    drop(system_rows);
    for entry in converted {
        if let Some(write) = entry.write {
            dynamic.extend(&entry.key, write.dynamic);
        }
    }

    insert_dynamic(tx.as_mut(), &dynamic).await?;

    tx.commit().await.map_err(|e| {
        OperationOutcomeError::fatal(
            IssueType::exception(),
            format!("Failed to commit PG search transaction: {e}"),
        )
    })?;

    Ok(())
}

/// The keys of one resource type's rows on its per-resource-type table.
/// `resource_type` is a constant on those tables, so it is not part of the key.
#[derive(Default)]
struct TypedKeys<'a> {
    tenant: Vec<&'a str>,
    project: Vec<&'a str>,
    resource_id: Vec<&'a str>,
}

/// The seven shared EAV tables, which every resource clears the same way.
const DYNAMIC_TABLES: [&str; 7] = [
    "search_dynamic_string",
    "search_dynamic_uri",
    "search_dynamic_number",
    "search_dynamic_token",
    "search_dynamic_date",
    "search_dynamic_reference",
    "search_dynamic_quantity",
];

/// Clears every trace of the previous version of each resource in the batch:
/// the dynamic rows, the per-resource-type row, then the anchor.
///
/// This used to be a single delete that let `ON DELETE CASCADE` find the
/// children. It no longer can — the cascade fired a referential-integrity
/// trigger for all ~145 tables referencing `search_resource` on *every* row
/// deleted, whatever the resource's type. Targeting the tables the batch
/// actually touches trades that for one statement per table.
async fn delete_previous_versions(
    conn: &mut sqlx::PgConnection,
    schema_registry: &SchemaRegistry,
    converted: &[Converted],
) -> Result<(), OperationOutcomeError> {
    let mut tenant = Vec::with_capacity(converted.len());
    let mut project = Vec::with_capacity(converted.len());
    let mut resource_type = Vec::with_capacity(converted.len());
    let mut resource_id = Vec::with_capacity(converted.len());

    for entry in converted {
        tenant.push(entry.key.tenant.clone());
        project.push(entry.key.project.clone());
        resource_type.push(entry.key.resource_type.clone());
        resource_id.push(entry.key.resource_id.clone());
    }

    for table in DYNAMIC_TABLES {
        sqlx::query(&format!(
            "DELETE FROM {table} t \
             USING unnest($1::text[], $2::text[], $3::text[], $4::text[]) \
                 AS k(tenant, project, resource_type, resource_id) \
             WHERE t.tenant = k.tenant AND t.project = k.project \
                 AND t.resource_type = k.resource_type AND t.resource_id = k.resource_id"
        ))
        .bind(&tenant)
        .bind(&project)
        .bind(&resource_type)
        .bind(&resource_id)
        .execute(&mut *conn)
        .await
        .map_err(wrap_sqlx)?;
    }

    // The per-resource-type tables, one statement each for the types this
    // batch actually carries. `resource_type` is a constant on each of these
    // tables, so the key is (tenant, project, resource_id) — which is exactly
    // the primary key.
    let mut by_type: HashMap<&str, TypedKeys<'_>> = HashMap::new();
    for entry in converted {
        let group = by_type.entry(entry.key.resource_type.as_str()).or_default();
        group.tenant.push(&entry.key.tenant);
        group.project.push(&entry.key.project);
        group.resource_id.push(&entry.key.resource_id);
    }

    for (resource_type, keys) in by_type {
        // A resource type with no generated table has no row to clear. The
        // registry is what decides that — probing for the table and ignoring
        // the error would not work, because Postgres aborts the whole
        // transaction on a failed statement.
        let Some(schema) = schema_registry.get(resource_type) else {
            continue;
        };

        sqlx::query(&format!(
            "DELETE FROM {} t \
             USING unnest($1::text[], $2::text[], $3::text[]) \
                 AS k(tenant, project, resource_id) \
             WHERE t.tenant = k.tenant AND t.project = k.project \
                 AND t.resource_id = k.resource_id",
            schema.table_name
        ))
        .bind(&keys.tenant)
        .bind(&keys.project)
        .bind(&keys.resource_id)
        .execute(&mut *conn)
        .await
        .map_err(wrap_sqlx)?;
    }

    sqlx::query(
        "DELETE FROM search_resource sr \
         USING unnest($1::text[], $2::text[], $3::text[], $4::text[]) \
             AS k(tenant, project, resource_type, resource_id) \
         WHERE sr.tenant = k.tenant AND sr.project = k.project \
             AND sr.resource_type = k.resource_type AND sr.resource_id = k.resource_id",
    )
    .bind(&tenant)
    .bind(&project)
    .bind(&resource_type)
    .bind(&resource_id)
    .execute(conn)
    .await
    .map_err(wrap_sqlx)?;

    Ok(())
}

/// Inserts the anchor rows. The per-resource-type rows and the dynamic rows
/// both have a foreign key pointing here, so these have to land first.
async fn insert_anchors(
    conn: &mut sqlx::PgConnection,
    converted: &[Converted],
) -> Result<(), OperationOutcomeError> {
    let mut tenant = Vec::new();
    let mut project = Vec::new();
    let mut resource_type = Vec::new();
    let mut resource_id = Vec::new();
    let mut version_id = Vec::new();

    for entry in converted {
        if entry.write.is_none() {
            continue;
        }

        tenant.push(entry.key.tenant.clone());
        project.push(entry.key.project.clone());
        resource_type.push(entry.key.resource_type.clone());
        resource_id.push(entry.key.resource_id.clone());
        version_id.push(entry.version_id.clone());
    }

    if tenant.is_empty() {
        return Ok(());
    }

    sqlx::query(
        "INSERT INTO search_resource (tenant, project, resource_type, resource_id, version_id) \
         SELECT * FROM unnest($1::text[], $2::text[], $3::text[], $4::text[], $5::text[])",
    )
    .bind(&tenant)
    .bind(&project)
    .bind(&resource_type)
    .bind(&resource_id)
    .bind(&version_id)
    .execute(conn)
    .await
    .map_err(wrap_sqlx)?;

    Ok(())
}

/// Writes every resource of one type as a single multi-row insert.
///
/// This is the one statement that binds per row rather than per column: the
/// value columns are already arrays, and `unnest` cannot produce a ragged
/// array of arrays, so the rows go in as a `VALUES` list — chunked to stay
/// under the bind-parameter cap.
async fn insert_system_rows(
    conn: &mut sqlx::PgConnection,
    schema: &ResourceTypeSchema,
    rows: &[SystemRow<'_>],
) -> Result<(), OperationOutcomeError> {
    if rows.is_empty() {
        return Ok(());
    }

    let per_row = SYSTEM_FIXED_COLUMNS + schema.columns.len();
    let rows_per_chunk = (MAX_BIND_PARAMS / per_row).max(1);
    let column_list = system_column_list(schema);

    for chunk in rows.chunks(rows_per_chunk) {
        let sql = system_insert_sql(schema, &column_list, chunk.len(), per_row);
        let mut query = sqlx::query(&sql);

        for row in chunk {
            query = query
                .bind(&row.key.tenant)
                .bind(&row.key.project)
                .bind(&row.key.resource_id)
                .bind(row.version_id);

            for (slot, column) in row.slots.iter().zip(&schema.columns) {
                query = match slot {
                    Some(ColumnValues::Text(values)) => query.bind(values),
                    Some(ColumnValues::BigInt(values)) => query.bind(values),
                    Some(ColumnValues::Double(values)) => query.bind(values),
                    // A parameter this resource produced no entry for. The
                    // NULL still has to carry the column's type, or Postgres
                    // rejects the parameter.
                    None => match column.column_type {
                        ColumnType::TextArray => query.bind(None::<Vec<Option<String>>>),
                        ColumnType::BigIntArray => query.bind(None::<Vec<i64>>),
                        ColumnType::DoubleArray => query.bind(None::<Vec<f64>>),
                    },
                };
            }
        }

        query.execute(&mut *conn).await.map_err(|e| {
            OperationOutcomeError::fatal(
                IssueType::exception(),
                format!(
                    "Failed to insert system search columns into {}: {e}",
                    schema.table_name
                ),
            )
        })?;
    }

    Ok(())
}

fn system_column_list(schema: &ResourceTypeSchema) -> String {
    // Column names come from `code_to_column_base`, which folds everything
    // outside `[a-z0-9_]`, so they cannot contain a quote to escape.
    let mut list = String::from("tenant, project, resource_id, version_id");
    for column in &schema.columns {
        let _ = write!(list, ", \"{}\"", column.name);
    }
    list
}

fn system_insert_sql(
    schema: &ResourceTypeSchema,
    column_list: &str,
    rows: usize,
    per_row: usize,
) -> String {
    let mut values = String::new();

    for row in 0..rows {
        if row > 0 {
            values.push_str(", ");
        }
        values.push('(');
        for offset in 0..per_row {
            if offset > 0 {
                values.push_str(", ");
            }
            let _ = write!(values, "${}", row * per_row + offset + 1);
        }
        values.push(')');
    }

    format!(
        "INSERT INTO {} ({column_list}) VALUES {values}",
        schema.table_name
    )
}

/// Writes every `search_dynamic_*` row in the batch — one statement per table,
/// each binding a fixed set of arrays however many rows it carries.
async fn insert_dynamic(
    conn: &mut sqlx::PgConnection,
    batch: &DynamicBatch,
) -> Result<(), OperationOutcomeError> {
    if !batch.string_keys.is_empty() {
        bind_keys(
            sqlx::query(
                "INSERT INTO search_dynamic_string (tenant, project, resource_type, resource_id, param_url, value) \
                 SELECT * FROM unnest($1::text[], $2::text[], $3::text[], $4::text[], $5::text[], $6::text[])",
            ),
            &batch.string_keys,
        )
        .bind(&batch.string_value)
        .execute(&mut *conn)
        .await
        .map_err(wrap_sqlx)?;
    }

    if !batch.uri_keys.is_empty() {
        bind_keys(
            sqlx::query(
                "INSERT INTO search_dynamic_uri (tenant, project, resource_type, resource_id, param_url, value) \
                 SELECT * FROM unnest($1::text[], $2::text[], $3::text[], $4::text[], $5::text[], $6::text[])",
            ),
            &batch.uri_keys,
        )
        .bind(&batch.uri_value)
        .execute(&mut *conn)
        .await
        .map_err(wrap_sqlx)?;
    }

    if !batch.number_keys.is_empty() {
        bind_keys(
            sqlx::query(
                "INSERT INTO search_dynamic_number (tenant, project, resource_type, resource_id, param_url, value) \
                 SELECT * FROM unnest($1::text[], $2::text[], $3::text[], $4::text[], $5::text[], $6::float8[])",
            ),
            &batch.number_keys,
        )
        .bind(&batch.number_value)
        .execute(&mut *conn)
        .await
        .map_err(wrap_sqlx)?;
    }

    if !batch.token_keys.is_empty() {
        bind_keys(
            sqlx::query(
                "INSERT INTO search_dynamic_token (tenant, project, resource_type, resource_id, param_url, system, code) \
                 SELECT * FROM unnest($1::text[], $2::text[], $3::text[], $4::text[], $5::text[], $6::text[], $7::text[])",
            ),
            &batch.token_keys,
        )
        .bind(&batch.token_system)
        .bind(&batch.token_code)
        .execute(&mut *conn)
        .await
        .map_err(wrap_sqlx)?;
    }

    if !batch.date_keys.is_empty() {
        bind_keys(
            sqlx::query(
                "INSERT INTO search_dynamic_date (tenant, project, resource_type, resource_id, param_url, start_ms, end_ms) \
                 SELECT * FROM unnest($1::text[], $2::text[], $3::text[], $4::text[], $5::text[], $6::int8[], $7::int8[])",
            ),
            &batch.date_keys,
        )
        .bind(&batch.date_start)
        .bind(&batch.date_end)
        .execute(&mut *conn)
        .await
        .map_err(wrap_sqlx)?;
    }

    if !batch.reference_keys.is_empty() {
        bind_keys(
            sqlx::query(
                "INSERT INTO search_dynamic_reference (tenant, project, resource_type, resource_id, param_url, target_resource_type, target_id, target_uri) \
                 SELECT * FROM unnest($1::text[], $2::text[], $3::text[], $4::text[], $5::text[], $6::text[], $7::text[], $8::text[])",
            ),
            &batch.reference_keys,
        )
        .bind(&batch.reference_target_type)
        .bind(&batch.reference_target_id)
        .bind(&batch.reference_target_uri)
        .execute(&mut *conn)
        .await
        .map_err(wrap_sqlx)?;
    }

    if !batch.quantity_keys.is_empty() {
        bind_keys(
            sqlx::query(
                "INSERT INTO search_dynamic_quantity (tenant, project, resource_type, resource_id, param_url, start_value, start_system, start_code, end_value, end_system, end_code) \
                 SELECT * FROM unnest($1::text[], $2::text[], $3::text[], $4::text[], $5::text[], $6::float8[], $7::text[], $8::text[], $9::float8[], $10::text[], $11::text[])",
            ),
            &batch.quantity_keys,
        )
        .bind(&batch.quantity_start_value)
        .bind(&batch.quantity_start_system)
        .bind(&batch.quantity_start_code)
        .bind(&batch.quantity_end_value)
        .bind(&batch.quantity_end_system)
        .bind(&batch.quantity_end_code)
        .execute(&mut *conn)
        .await
        .map_err(wrap_sqlx)?;
    }

    Ok(())
}

type PgQuery<'a> = sqlx::query::Query<'a, Postgres, sqlx::postgres::PgArguments>;

fn bind_keys<'a>(query: PgQuery<'a>, keys: &'a KeyColumns) -> PgQuery<'a> {
    query
        .bind(&keys.tenant)
        .bind(&keys.project)
        .bind(&keys.resource_type)
        .bind(&keys.resource_id)
        .bind(&keys.param_url)
}

fn wrap_sqlx(e: sqlx::Error) -> OperationOutcomeError {
    OperationOutcomeError::fatal(
        IssueType::exception(),
        format!("PG search indexing failed: {e}"),
    )
}
