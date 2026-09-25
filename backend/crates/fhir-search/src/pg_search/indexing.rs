//! Batch indexing into the PG search tables, in two phases:
//!
//! 1. **Convert**: evaluate every resource's `FHIRPath` expressions into rows,
//!    in parallel and without the database. Per-resource failures are caught
//!    here and reported against that resource.
//! 2. **Write**: one set-based statement per table for the whole batch, in a
//!    single transaction (~15 statements for 1000 resources). Postgres aborts
//!    the transaction on any error, so a failure here fails the batch and the
//!    worker retries it.

use std::collections::HashMap;
use std::fmt::Write as _;
use std::sync::Arc;

use haste_fhir_model::r4::generated::terminology::IssueType;
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhirpath::FPEngine;
use haste_repository::types::FHIRMethod;
use sqlx::{Pool, Postgres, Row, postgres::PgRow};

use super::{
    PgSearchError, keys, resource_to_search_index,
    schema::{
        ColumnDef, ColumnType, ParamColumns, ResourceTypeSchema, SHARED_TABLES, SchemaRegistry,
        SharedTable, column_index, resource_table_name, shared_table_name, shared_value_columns,
    },
};
use crate::{
    IndexFailure, IndexOutcome, IndexResource, ResolvedParameter, SearchParameterResolve,
    indexing_conversion::{
        DateRange, DynamicParameterEntry, InsertableIndex, QuantityRange, ReferenceIndex,
        TokenIndex,
    },
};

/// Postgres's bind limit per statement. Only the `VALUES` inserts can reach
/// it; the `unnest` inserts bind a fixed number of arrays.
const MAX_BIND_PARAMS: usize = 65535;

/// `res_key, scope`, before a type table's generated columns.
const TYPE_TABLE_FIXED_COLUMNS: usize = 2;

/// The anchor's fixed columns, before its generated ones. `res_key` is
/// allocated by the table.
const ANCHOR_FIXED_COLUMNS: [&str; 6] = [
    "tenant",
    "project",
    "resource_type",
    "resource_id",
    "version_id",
    "sequence",
];

/// Leading columns of every shared-table insert, bound as `$1`, `$2`.
const SHARED_KEY_COLUMNS: [&str; 2] = ["res_key", "param_identity"];

type PgQuery<'a> = sqlx::query::Query<'a, Postgres, sqlx::postgres::PgArguments>;

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

    // A batch may hold several versions of one resource; only the last is
    // indexed, and the earlier ones share its outcome.
    let (resources, superseded) = dedupe_keep_last(resources);

    // Resolve once per (tenant, project, type): a cold cache hits the database.
    let parameter_sets = resolve_parameter_sets(parameter_resolver.as_ref(), &resources).await?;

    let (converted, failed) = convert_batch(
        fp_engine,
        schema_registry.clone(),
        &parameter_sets,
        resources,
        superseded,
    )
    .await?;

    if !failed.is_empty() {
        tracing::error!(
            "PG search: {} failed item(s) out of {}.",
            failed.len(),
            total
        );
    }

    write_batch(pool, schema_registry, converted).await?;

    Ok(IndexOutcome {
        succeeded: total - failed.len(),
        failed,
    })
}

// ---------------------------------------------------------------------------
// Identity
// ---------------------------------------------------------------------------

/// The anchor's primary key.
#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct ResourceKey {
    tenant: String,
    project: String,
    resource_type: String,
    resource_id: String,
}

fn resource_key(resource: &IndexResource) -> ResourceKey {
    ResourceKey {
        tenant: resource.tenant.as_ref().to_string(),
        project: resource.project.as_ref().to_string(),
        resource_type: resource.resource_type.as_ref().to_string(),
        resource_id: resource.id.as_ref().to_string(),
    }
}

/// Keeps each resource's last occurrence (in its first position) and returns
/// the earlier versions it displaced.
fn dedupe_keep_last(
    resources: Vec<IndexResource>,
) -> (Vec<IndexResource>, HashMap<ResourceKey, Vec<IndexResource>>) {
    let capacity = resources.len();
    let (_, kept, superseded) = resources.into_iter().fold(
        (
            HashMap::<ResourceKey, usize>::new(),
            Vec::with_capacity(capacity),
            HashMap::<ResourceKey, Vec<IndexResource>>::new(),
        ),
        |(mut position, mut kept, mut superseded), resource| {
            let key = resource_key(&resource);
            if let Some(&index) = position.get(&key) {
                let previous = std::mem::replace(&mut kept[index], resource);
                superseded.entry(key).or_default().push(previous);
            } else {
                position.insert(key, kept.len());
                kept.push(resource);
            }
            (position, kept, superseded)
        },
    );

    (kept, superseded)
}

// ---------------------------------------------------------------------------
// Phase 1: convert
// ---------------------------------------------------------------------------

/// One resource's rows, ready to bind.
struct Converted {
    key: ResourceKey,
    version_id: String,
    sequence: i64,
    /// `None` for a delete.
    write: Option<ConvertedWrite>,
}

struct ConvertedWrite {
    /// One slot per type table column; `None` if the type has no table.
    type_row: Option<Vec<Option<ColumnValue>>>,
    /// One slot per anchor column.
    anchor_row: Vec<Option<ColumnValue>>,
    /// Shared-table values, keyed by parameter URL.
    dynamic: Vec<(String, InsertableIndex)>,
}

/// A column's single value.
enum ColumnValue {
    Text(String),
    BigInt(i64),
    Double(f64),
}

/// (tenant, project, resource type): what a parameter set is resolved for.
type GroupKey = (String, String, String);

fn group_key(resource: &IndexResource) -> GroupKey {
    (
        resource.tenant.as_ref().to_string(),
        resource.project.as_ref().to_string(),
        resource.resource_type.as_ref().to_string(),
    )
}

/// The parameters for one group, plus `code` → URL for routing column-less
/// values to the shared tables.
struct ParameterSet {
    parameters: Vec<ResolvedParameter>,
    url_by_code: HashMap<String, String>,
}

/// Resolves each distinct group once. Deletes evaluate nothing and are
/// skipped.
async fn resolve_parameter_sets<Resolver: SearchParameterResolve>(
    resolver: &Resolver,
    resources: &[IndexResource],
) -> Result<HashMap<GroupKey, Arc<ParameterSet>>, OperationOutcomeError> {
    let mut sets = HashMap::new();

    for resource in resources {
        let group = group_key(resource);
        if matches!(resource.fhir_method, FHIRMethod::Delete) || sets.contains_key(&group) {
            continue;
        }

        let parameters = resolver
            .by_resource_type(&resource.tenant, &resource.project, &resource.resource_type)
            .await?;
        let set = parameter_set(resource, parameters)?;
        sets.insert(group, Arc::new(set));
    }

    Ok(sets)
}

/// Fails if two parameters share an identity, since each would answer the
/// other's searches.
fn parameter_set(
    resource: &IndexResource,
    parameters: Vec<ResolvedParameter>,
) -> Result<ParameterSet, OperationOutcomeError> {
    keys::check_identities(
        resource.tenant.as_ref(),
        resource.project.as_ref(),
        parameters
            .iter()
            .filter_map(|p| p.search_parameter.url.value.as_deref()),
    )
    .map_err(|(first, second)| {
        OperationOutcomeError::fatal(
            IssueType::exception(),
            format!(
                "Search parameters '{first}' and '{second}' share a PG search identity; \
                 neither can be indexed until one is renamed."
            ),
        )
    })?;

    let url_by_code = parameters
        .iter()
        .filter_map(|p| {
            let code = p.search_parameter.code.value.as_deref()?;
            let url = p.search_parameter.url.value.as_deref()?;
            Some((code.to_string(), url.to_string()))
        })
        .collect();

    Ok(ParameterSet {
        parameters,
        url_by_code,
    })
}

/// Converts every resource in parallel. A resource's own failure is reported
/// against it (and its superseded versions); a panicked task fails the batch.
async fn convert_batch(
    fp_engine: Arc<FPEngine>,
    schema_registry: Arc<SchemaRegistry>,
    parameter_sets: &HashMap<GroupKey, Arc<ParameterSet>>,
    resources: Vec<IndexResource>,
    mut superseded: HashMap<ResourceKey, Vec<IndexResource>>,
) -> Result<(Vec<Converted>, Vec<IndexFailure>), OperationOutcomeError> {
    let tasks: Vec<_> = resources
        .into_iter()
        .map(|resource| {
            let fp_engine = fp_engine.clone();
            let schema_registry = schema_registry.clone();
            let parameters = parameter_sets.get(&group_key(&resource)).cloned();

            tokio::spawn(async move {
                let result = convert_resource(
                    &fp_engine,
                    &schema_registry,
                    parameters.as_deref(),
                    &resource,
                )
                .await;
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

        let displaced = superseded
            .remove(&resource_key(&resource))
            .unwrap_or_default();

        match result {
            // The final version is indexed, so the displaced ones are handled.
            Ok(entry) => converted.push(entry),
            Err(error) => {
                failed.extend(displaced.into_iter().map(superseded_failure));
                failed.push(IndexFailure { resource, error });
            }
        }
    }

    Ok((converted, failed))
}

fn superseded_failure(resource: IndexResource) -> IndexFailure {
    IndexFailure {
        resource,
        error: OperationOutcomeError::fatal(
            IssueType::exception(),
            "Superseded by a later version in the same batch that failed to index.".to_string(),
        ),
    }
}

async fn convert_resource(
    fp_engine: &Arc<FPEngine>,
    schema_registry: &SchemaRegistry,
    parameter_set: Option<&ParameterSet>,
    resource: &IndexResource,
) -> Result<Converted, OperationOutcomeError> {
    let write = match &resource.fhir_method {
        FHIRMethod::Create | FHIRMethod::Update => {
            let set = parameter_set.ok_or_else(|| {
                OperationOutcomeError::fatal(
                    IssueType::exception(),
                    format!(
                        "No search parameters resolved for resource type '{}'.",
                        resource.resource_type.as_ref()
                    ),
                )
            })?;
            Some(convert_write(fp_engine, schema_registry, set, resource).await?)
        }
        FHIRMethod::Delete => None,
        method @ FHIRMethod::Read => {
            return Err(PgSearchError::UnsupportedFHIRMethod((*method).clone()).into());
        }
    };

    Ok(Converted {
        key: resource_key(resource),
        version_id: resource.version_id.as_ref().to_string(),
        sequence: resource.sequence,
        write,
    })
}

async fn convert_write(
    fp_engine: &Arc<FPEngine>,
    schema_registry: &SchemaRegistry,
    set: &ParameterSet,
    resource: &IndexResource,
) -> Result<ConvertedWrite, OperationOutcomeError> {
    let index = resource_to_search_index(
        fp_engine.clone(),
        &set.parameters,
        &resource.resource,
        resource.resource_type.as_ref(),
    )
    .await?;

    let anchor = &schema_registry.anchor;
    let schema = schema_registry.schemas.get(resource.resource_type.as_ref());
    let type_row = schema.map(|schema| column_slots(schema, &index.system_entries));
    let anchor_row = column_slots(anchor, &index.system_entries);

    // A singular value with no column (name clash, or no type table) goes to
    // the shared tables, keyed by URL.
    let has_column = |code: &str| {
        anchor.parameters.contains_key(code)
            || schema.is_some_and(|schema| schema.parameters.contains_key(code))
    };
    let column_less = index
        .system_entries
        .into_iter()
        .filter(|(code, _)| !has_column(code))
        .filter_map(|(code, insertable)| Some((set.url_by_code.get(&code)?.clone(), insertable)));

    Ok(ConvertedWrite {
        type_row,
        anchor_row,
        dynamic: index
            .dynamic_entries
            .into_iter()
            .chain(column_less)
            .collect(),
    })
}

/// Lays a resource's values out by column position, so the batch can bind one
/// fixed column list.
fn column_slots(
    schema: &ResourceTypeSchema,
    entries: &[(String, InsertableIndex)],
) -> Vec<Option<ColumnValue>> {
    entries
        .iter()
        .filter_map(|(code, insertable)| {
            Some(column_values(schema.parameters.get(code)?, insertable))
        })
        .flatten()
        .flatten()
        .fold(
            (0..schema.columns.len()).map(|_| None).collect(),
            |mut slots: Vec<Option<ColumnValue>>, (name, value)| {
                if let Some(index) = column_index(schema, name) {
                    slots[index] = Some(value);
                }
                slots
            },
        )
}

/// Each column's value for one parameter (at most four columns). Returns
/// nothing if the value's type doesn't match the columns.
fn column_values<'a>(
    columns: &'a ParamColumns,
    insertable: &InsertableIndex,
) -> [Option<(&'a str, ColumnValue)>; 4] {
    let at = |name: &'a String, value: Option<ColumnValue>| value.map(|v| (name.as_str(), v));

    match (columns, insertable) {
        (ParamColumns::String { value }, InsertableIndex::String(values))
        | (ParamColumns::Uri { value }, InsertableIndex::URI(values)) => [
            at(value, first_text(values.iter().map(|v| Some(v.as_str())))),
            None,
            None,
            None,
        ],
        (ParamColumns::Number { value }, InsertableIndex::Number(numbers)) => [
            at(value, first_double(numbers.iter().copied())),
            None,
            None,
            None,
        ],
        (ParamColumns::Token { system, code }, InsertableIndex::Token(tokens)) => [
            // `_id` has no system column.
            system
                .as_ref()
                .and_then(|system| at(system, first_text(tokens.iter().map(TokenIndex::system)))),
            at(code, first_text(tokens.iter().map(TokenIndex::code))),
            None,
            None,
        ],
        (ParamColumns::Date { start, end }, InsertableIndex::Date(ranges)) => [
            at(start, first_bigint(ranges.iter().map(|d| d.start))),
            at(end, first_bigint(ranges.iter().map(|d| d.end))),
            None,
            None,
        ],
        (
            ParamColumns::Reference {
                target_type,
                target_id,
            },
            InsertableIndex::Reference(references),
        ) => [
            at(
                target_type,
                first_text(references.iter().map(ReferenceIndex::resource_type)),
            ),
            at(
                target_id,
                first_text(references.iter().map(ReferenceIndex::id)),
            ),
            None,
            None,
        ],
        (
            ParamColumns::Quantity {
                start,
                end,
                system,
                code,
            },
            InsertableIndex::Quantity(quantities),
        ) => [
            at(
                start,
                first_double(quantities.iter().map(QuantityRange::start_value)),
            ),
            at(
                end,
                first_double(quantities.iter().map(QuantityRange::end_value)),
            ),
            at(
                system,
                first_text(quantities.iter().map(QuantityRange::start_system)),
            ),
            at(
                code,
                first_text(quantities.iter().map(QuantityRange::start_code)),
            ),
        ],
        _ => [None, None, None, None],
    }
}

// A column only exists where a second value is impossible, so the first value
// is the only one.

fn first_text<'a>(mut values: impl Iterator<Item = Option<&'a str>>) -> Option<ColumnValue> {
    values
        .next()
        .flatten()
        .map(|value| ColumnValue::Text(value.to_string()))
}

fn first_bigint(mut values: impl Iterator<Item = i64>) -> Option<ColumnValue> {
    values.next().map(ColumnValue::BigInt)
}

fn first_double(mut values: impl Iterator<Item = f64>) -> Option<ColumnValue> {
    values.next().map(ColumnValue::Double)
}

// ---------------------------------------------------------------------------
// Shared-table rows
// ---------------------------------------------------------------------------
//
// Rows are held column-wise, one `Vec` per column, which is the shape the
// `unnest` insert binds.

/// A shared-table row's key: its resource and parameter.
#[derive(Clone, Copy)]
struct RowKey {
    res_key: i64,
    param_identity: i64,
}

#[derive(Default)]
struct KeyColumns {
    res_key: Vec<i64>,
    param_identity: Vec<i64>,
}

/// Rows for the string or uri table.
#[derive(Default)]
struct TextRows {
    keys: KeyColumns,
    value: Vec<String>,
}

#[derive(Default)]
struct NumberRows {
    keys: KeyColumns,
    value: Vec<f64>,
}

#[derive(Default)]
struct TokenRows {
    keys: KeyColumns,
    system: Vec<Option<String>>,
    code: Vec<Option<String>>,
}

#[derive(Default)]
struct DateRows {
    keys: KeyColumns,
    start_ms: Vec<i64>,
    end_ms: Vec<i64>,
}

#[derive(Default)]
struct ReferenceRows {
    keys: KeyColumns,
    target_resource_type: Vec<Option<String>>,
    target_id: Vec<Option<String>>,
}

#[derive(Default)]
struct QuantityRows {
    keys: KeyColumns,
    start_value: Vec<f64>,
    end_value: Vec<f64>,
    start_system: Vec<Option<String>>,
    start_code: Vec<Option<String>>,
}

/// The batch's rows for every shared table.
#[derive(Default)]
struct SharedRows {
    string: TextRows,
    uri: TextRows,
    number: NumberRows,
    token: TokenRows,
    date: DateRows,
    reference: ReferenceRows,
    quantity: QuantityRows,
}

/// A borrowed value column, bound as one array.
enum ValueColumn<'a> {
    Text(&'a [String]),
    OptionalText(&'a [Option<String>]),
    BigInt(&'a [i64]),
    Double(&'a [f64]),
}

fn row_key(res_key: i64, resource: &ResourceKey, param_url: &str) -> RowKey {
    RowKey {
        res_key,
        param_identity: keys::param_identity(&resource.tenant, &resource.project, param_url),
    }
}

/// Adds one parameter's values to the table for their type.
fn add_parameter_rows(
    rows: &mut SharedRows,
    res_key: i64,
    resource: &ResourceKey,
    param_url: &str,
    insertable: &InsertableIndex,
) {
    let key = || row_key(res_key, resource, param_url);

    match insertable {
        InsertableIndex::String(values) => extend_text(&mut rows.string, key(), values),
        InsertableIndex::URI(values) => extend_text(&mut rows.uri, key(), values),
        InsertableIndex::Number(values) => extend_numbers(&mut rows.number, key(), values),
        InsertableIndex::Token(values) => extend_tokens(&mut rows.token, key(), values),
        InsertableIndex::Date(values) => extend_dates(&mut rows.date, key(), values),
        InsertableIndex::Reference(values) => {
            extend_references(&mut rows.reference, key(), values);
        }
        InsertableIndex::Quantity(values) => extend_quantities(&mut rows.quantity, key(), values),
        // Project-level parameters: each entry carries its own URL and values.
        InsertableIndex::DynamicParameters(entries) => entries
            .iter()
            .for_each(|entry| add_dynamic_entry_rows(rows, res_key, resource, entry)),
        // No PG representation; `Meta` is Elasticsearch-only.
        InsertableIndex::Composite(_) | InsertableIndex::Special(_) | InsertableIndex::Meta(_) => {}
    }
}

fn add_dynamic_entry_rows(
    rows: &mut SharedRows,
    res_key: i64,
    resource: &ResourceKey,
    entry: &DynamicParameterEntry,
) {
    let key = row_key(res_key, resource, &entry.url);
    let value = &entry.value;

    extend_text(
        &mut rows.string,
        key,
        value.string.as_deref().unwrap_or_default(),
    );
    extend_text(&mut rows.uri, key, value.uri.as_deref().unwrap_or_default());
    extend_numbers(
        &mut rows.number,
        key,
        value.number.as_deref().unwrap_or_default(),
    );
    extend_tokens(
        &mut rows.token,
        key,
        value.token.as_deref().unwrap_or_default(),
    );
    extend_dates(
        &mut rows.date,
        key,
        value.date.as_deref().unwrap_or_default(),
    );
    extend_references(
        &mut rows.reference,
        key,
        value.reference.as_deref().unwrap_or_default(),
    );
    extend_quantities(
        &mut rows.quantity,
        key,
        value.quantity.as_deref().unwrap_or_default(),
    );
}

fn extend_keys(keys: &mut KeyColumns, key: RowKey, count: usize) {
    keys.res_key.extend(std::iter::repeat_n(key.res_key, count));
    keys.param_identity
        .extend(std::iter::repeat_n(key.param_identity, count));
}

fn extend_text(rows: &mut TextRows, key: RowKey, values: &[String]) {
    extend_keys(&mut rows.keys, key, values.len());
    rows.value.extend(values.iter().cloned());
}

fn extend_numbers(rows: &mut NumberRows, key: RowKey, values: &[f64]) {
    extend_keys(&mut rows.keys, key, values.len());
    rows.value.extend_from_slice(values);
}

fn extend_tokens(rows: &mut TokenRows, key: RowKey, tokens: &[TokenIndex]) {
    extend_keys(&mut rows.keys, key, tokens.len());
    rows.system
        .extend(tokens.iter().map(|t| t.system().map(str::to_string)));
    rows.code
        .extend(tokens.iter().map(|t| t.code().map(str::to_string)));
}

fn extend_dates(rows: &mut DateRows, key: RowKey, ranges: &[DateRange]) {
    extend_keys(&mut rows.keys, key, ranges.len());
    rows.start_ms.extend(ranges.iter().map(|r| r.start));
    rows.end_ms.extend(ranges.iter().map(|r| r.end));
}

fn extend_references(rows: &mut ReferenceRows, key: RowKey, references: &[ReferenceIndex]) {
    extend_keys(&mut rows.keys, key, references.len());
    rows.target_resource_type.extend(
        references
            .iter()
            .map(|r| r.resource_type().map(str::to_string)),
    );
    rows.target_id
        .extend(references.iter().map(|r| r.id().map(str::to_string)));
}

fn extend_quantities(rows: &mut QuantityRows, key: RowKey, quantities: &[QuantityRange]) {
    extend_keys(&mut rows.keys, key, quantities.len());
    rows.start_value
        .extend(quantities.iter().map(QuantityRange::start_value));
    rows.end_value
        .extend(quantities.iter().map(QuantityRange::end_value));
    rows.start_system.extend(
        quantities
            .iter()
            .map(|q| q.start_system().map(str::to_string)),
    );
    rows.start_code.extend(
        quantities
            .iter()
            .map(|q| q.start_code().map(str::to_string)),
    );
}

/// `INSERT INTO <table> (<keys>, <values>) SELECT * FROM unnest($1, ..., $n)`.
/// Every array is cast to its type, since an all-NULL array gives Postgres
/// nothing to infer from.
fn dynamic_insert_sql(table: &str, shared_table: SharedTable) -> String {
    let value_columns = shared_value_columns(shared_table);

    let columns = SHARED_KEY_COLUMNS
        .into_iter()
        .chain(value_columns.iter().map(|column| column.name))
        .collect::<Vec<_>>()
        .join(", ");

    let arrays = ["$1::int8[]".to_string(), "$2::int8[]".to_string()]
        .into_iter()
        .chain(value_columns.iter().enumerate().map(|(offset, column)| {
            format!(
                "${}::{}[]",
                offset + SHARED_KEY_COLUMNS.len() + 1,
                column.sql_type
            )
        }))
        .collect::<Vec<_>>()
        .join(", ");

    format!("INSERT INTO {table} ({columns}) SELECT * FROM unnest({arrays})")
}

/// One statement per non-empty shared table.
async fn insert_shared_rows(
    conn: &mut sqlx::PgConnection,
    registry: &SchemaRegistry,
    rows: &SharedRows,
) -> Result<(), OperationOutcomeError> {
    let tables: [(SharedTable, &KeyColumns, &[ValueColumn<'_>]); 7] = [
        (
            SharedTable::String,
            &rows.string.keys,
            &[ValueColumn::Text(&rows.string.value)],
        ),
        (
            SharedTable::Uri,
            &rows.uri.keys,
            &[ValueColumn::Text(&rows.uri.value)],
        ),
        (
            SharedTable::Number,
            &rows.number.keys,
            &[ValueColumn::Double(&rows.number.value)],
        ),
        (
            SharedTable::Token,
            &rows.token.keys,
            &[
                ValueColumn::OptionalText(&rows.token.system),
                ValueColumn::OptionalText(&rows.token.code),
            ],
        ),
        (
            SharedTable::Date,
            &rows.date.keys,
            &[
                ValueColumn::BigInt(&rows.date.start_ms),
                ValueColumn::BigInt(&rows.date.end_ms),
            ],
        ),
        (
            SharedTable::Reference,
            &rows.reference.keys,
            &[
                ValueColumn::OptionalText(&rows.reference.target_resource_type),
                ValueColumn::OptionalText(&rows.reference.target_id),
            ],
        ),
        (
            SharedTable::Quantity,
            &rows.quantity.keys,
            &[
                ValueColumn::Double(&rows.quantity.start_value),
                ValueColumn::Double(&rows.quantity.end_value),
                ValueColumn::OptionalText(&rows.quantity.start_system),
                ValueColumn::OptionalText(&rows.quantity.start_code),
            ],
        ),
    ];

    for (table, keys, values) in tables {
        if !keys.res_key.is_empty() {
            insert_shared_table(&mut *conn, registry, table, keys, values).await?;
        }
    }
    Ok(())
}

/// Binds the key arrays, then `values` in [`shared_value_columns`] order.
async fn insert_shared_table(
    conn: &mut sqlx::PgConnection,
    registry: &SchemaRegistry,
    table: SharedTable,
    keys: &KeyColumns,
    values: &[ValueColumn<'_>],
) -> Result<(), OperationOutcomeError> {
    debug_assert_eq!(values.len(), shared_value_columns(table).len());

    let sql = dynamic_insert_sql(&shared_table_name(&registry.version, table), table);
    let query = sqlx::query(&sql)
        .bind(keys.res_key.as_slice())
        .bind(keys.param_identity.as_slice());

    values
        .iter()
        .fold(query, |query, column| match column {
            ValueColumn::Text(values) => query.bind(*values),
            ValueColumn::OptionalText(values) => query.bind(*values),
            ValueColumn::BigInt(values) => query.bind(*values),
            ValueColumn::Double(values) => query.bind(*values),
        })
        .execute(conn)
        .await
        .map_err(PgSearchError::from)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Phase 2: write
// ---------------------------------------------------------------------------

/// One resource's row on its type table.
struct TypeRow<'a> {
    res_key: i64,
    key: &'a ResourceKey,
    slots: &'a [Option<ColumnValue>],
}

/// A resource's anchor row after the upsert.
struct AnchorKey {
    res_key: i64,
    resource_type: String,
    /// Indexed before this batch, so its old rows must be cleared.
    existed: bool,
}

/// Writes the batch in one transaction.
///
/// The anchor goes first: its upsert allocates or keeps each `res_key` and
/// reports which resources already existed. Only those and deletes have old
/// rows to clear. Any failure aborts the transaction and the worker retries
/// the batch.
async fn write_batch(
    pool: &Pool<Postgres>,
    schema_registry: &SchemaRegistry,
    mut converted: Vec<Converted>,
) -> Result<(), OperationOutcomeError> {
    if converted.is_empty() {
        return Ok(());
    }

    // One lock order for every writer, so overlapping batches queue instead of
    // deadlocking.
    converted.sort_by(|a, b| a.key.cmp(&b.key));

    let (deletes, writes): (Vec<&Converted>, Vec<&Converted>) =
        converted.iter().partition(|entry| entry.write.is_none());

    let mut tx = pool.begin().await.map_err(|e| {
        OperationOutcomeError::fatal(
            IssueType::exception(),
            format!("Failed to begin PG search transaction: {e}"),
        )
    })?;

    let deleted = delete_anchors(tx.as_mut(), schema_registry, &deletes).await?;
    let anchors = upsert_anchors(tx.as_mut(), schema_registry, &writes).await?;

    // Deleted resources, plus re-indexed ones' previous rows.
    let stale: Vec<(i64, String)> = deleted
        .into_iter()
        .chain(
            anchors
                .values()
                .filter(|anchor| anchor.existed)
                .map(|anchor| (anchor.res_key, anchor.resource_type.clone())),
        )
        .collect();
    clear_rows(tx.as_mut(), schema_registry, &stale).await?;

    // A resource missing from `anchors` has a newer version already indexed
    // and keeps its rows.
    let (shared_rows, type_rows) = writes
        .iter()
        .filter_map(|entry| Some((entry, entry.write.as_ref()?, anchors.get(&entry.key)?)))
        .fold(
            (
                SharedRows::default(),
                HashMap::<&str, Vec<TypeRow<'_>>>::new(),
            ),
            |(mut shared_rows, mut type_rows), (entry, write, anchor)| {
                for (param_url, insertable) in &write.dynamic {
                    add_parameter_rows(
                        &mut shared_rows,
                        anchor.res_key,
                        &entry.key,
                        param_url,
                        insertable,
                    );
                }
                if let Some(slots) = &write.type_row {
                    type_rows
                        .entry(entry.key.resource_type.as_str())
                        .or_default()
                        .push(TypeRow {
                            res_key: anchor.res_key,
                            key: &entry.key,
                            slots,
                        });
                }
                (shared_rows, type_rows)
            },
        );

    for (resource_type, rows) in &type_rows {
        if let Some(schema) = schema_registry.schemas.get(*resource_type) {
            insert_type_rows(tx.as_mut(), schema, rows).await?;
        }
    }

    insert_shared_rows(tx.as_mut(), schema_registry, &shared_rows).await?;

    tx.commit().await.map_err(|e| {
        OperationOutcomeError::fatal(
            IssueType::exception(),
            format!("Failed to commit PG search transaction: {e}"),
        )
    })
}

/// Deletes the anchor rows of deleted resources and returns their
/// `(res_key, resource_type)`. A delete older than the indexed version is a
/// replay and is ignored.
async fn delete_anchors(
    conn: &mut sqlx::PgConnection,
    schema_registry: &SchemaRegistry,
    deletes: &[&Converted],
) -> Result<Vec<(i64, String)>, OperationOutcomeError> {
    if deletes.is_empty() {
        return Ok(Vec::new());
    }

    let column = |field: fn(&ResourceKey) -> &str| -> Vec<&str> {
        deletes.iter().map(|entry| field(&entry.key)).collect()
    };
    let sequence: Vec<i64> = deletes.iter().map(|entry| entry.sequence).collect();

    let sql = format!(
        "DELETE FROM {anchor} sr \
         USING unnest($1::text[], $2::text[], $3::text[], $4::text[], $5::int8[]) \
             AS k(tenant, project, resource_type, resource_id, sequence) \
         WHERE sr.tenant = k.tenant AND sr.project = k.project \
             AND sr.resource_type = k.resource_type AND sr.resource_id = k.resource_id \
             AND sr.sequence <= k.sequence \
         RETURNING sr.res_key, sr.resource_type",
        anchor = resource_table_name(&schema_registry.version),
    );

    let rows = sqlx::query(&sql)
        .bind(column(|key| &key.tenant))
        .bind(column(|key| &key.project))
        .bind(column(|key| &key.resource_type))
        .bind(column(|key| &key.resource_id))
        .bind(sequence)
        .fetch_all(conn)
        .await
        .map_err(PgSearchError::from)?;

    Ok(rows
        .iter()
        .map(|row| (row.get("res_key"), row.get("resource_type")))
        .collect())
}

/// Upserts the anchor rows of written resources and returns each `res_key`.
///
/// An existing resource keeps its key. The update is skipped (and the resource
/// left out of the result) when a newer version is already indexed, so replays
/// can't roll back; the same version passes, so a full reindex rewrites
/// everything.
async fn upsert_anchors(
    conn: &mut sqlx::PgConnection,
    schema_registry: &SchemaRegistry,
    writes: &[&Converted],
) -> Result<HashMap<ResourceKey, AnchorKey>, OperationOutcomeError> {
    let mut anchors = HashMap::with_capacity(writes.len());
    if writes.is_empty() {
        return Ok(anchors);
    }

    let anchor = &schema_registry.anchor;
    let per_row = ANCHOR_FIXED_COLUMNS.len() + anchor.columns.len();

    let column_list =
        anchor
            .columns
            .iter()
            .fold(ANCHOR_FIXED_COLUMNS.join(", "), |mut list, column| {
                let _ = write!(list, ", \"{}\"", column.name);
                list
            });
    let updates = anchor.columns.iter().fold(
        String::from("version_id = EXCLUDED.version_id, sequence = EXCLUDED.sequence"),
        |mut updates, column| {
            let _ = write!(updates, ", \"{0}\" = EXCLUDED.\"{0}\"", column.name);
            updates
        },
    );

    for chunk in writes.chunks((MAX_BIND_PARAMS / per_row).max(1)) {
        let sql = format!(
            "INSERT INTO {table} ({column_list}) VALUES {values} \
             ON CONFLICT (tenant, project, resource_type, resource_id) DO UPDATE SET {updates} \
             WHERE {table}.sequence <= EXCLUDED.sequence \
             RETURNING tenant, project, resource_type, resource_id, res_key, \
                 (xmax = 0) AS inserted",
            table = anchor.table_name,
            values = values_clause(chunk.len(), per_row),
        );

        let query = chunk.iter().fold(sqlx::query(&sql), |query, entry| {
            let query = query
                .bind(&entry.key.tenant)
                .bind(&entry.key.project)
                .bind(&entry.key.resource_type)
                .bind(&entry.key.resource_id)
                .bind(&entry.version_id)
                .bind(entry.sequence);
            match &entry.write {
                Some(write) => bind_columns(query, &write.anchor_row, &anchor.columns),
                None => query,
            }
        });

        let rows = query.fetch_all(&mut *conn).await.map_err(|e| {
            OperationOutcomeError::fatal(
                IssueType::exception(),
                format!(
                    "Failed to upsert anchor rows into {}: {e}",
                    anchor.table_name
                ),
            )
        })?;

        anchors.extend(rows.iter().map(anchor_entry));
    }

    Ok(anchors)
}

fn anchor_entry(row: &PgRow) -> (ResourceKey, AnchorKey) {
    let key = ResourceKey {
        tenant: row.get("tenant"),
        project: row.get("project"),
        resource_type: row.get("resource_type"),
        resource_id: row.get("resource_id"),
    };
    let inserted: bool = row.get("inserted");
    let anchor = AnchorKey {
        res_key: row.get("res_key"),
        resource_type: key.resource_type.clone(),
        existed: !inserted,
    };
    (key, anchor)
}

/// Deletes these resources' rows from the shared tables and their type
/// tables. No foreign keys: a cascade would fire a trigger per table (~145)
/// per deleted row, so each table gets one `res_key = ANY($1)` statement.
async fn clear_rows(
    conn: &mut sqlx::PgConnection,
    schema_registry: &SchemaRegistry,
    stale: &[(i64, String)],
) -> Result<(), OperationOutcomeError> {
    if stale.is_empty() {
        return Ok(());
    }

    let all: Vec<i64> = stale.iter().map(|(res_key, _)| *res_key).collect();
    for table in SHARED_TABLES {
        let name = shared_table_name(&schema_registry.version, table);
        delete_by_res_key(&mut *conn, &name, &all).await?;
    }

    let by_type = stale.iter().fold(
        HashMap::<&str, Vec<i64>>::new(),
        |mut by_type, (res_key, resource_type)| {
            by_type.entry(resource_type).or_default().push(*res_key);
            by_type
        },
    );

    for (resource_type, res_keys) in by_type {
        // Only types with a table; a failed probe would abort the transaction.
        if let Some(schema) = schema_registry.schemas.get(resource_type) {
            delete_by_res_key(&mut *conn, &schema.table_name, &res_keys).await?;
        }
    }

    Ok(())
}

async fn delete_by_res_key(
    conn: &mut sqlx::PgConnection,
    table: &str,
    res_keys: &[i64],
) -> Result<(), OperationOutcomeError> {
    sqlx::query(&format!("DELETE FROM {table} WHERE res_key = ANY($1)"))
        .bind(res_keys)
        .execute(conn)
        .await
        .map_err(PgSearchError::from)?;
    Ok(())
}

/// Binds one row's column values in schema order. A missing value is still
/// bound as a typed NULL so Postgres can infer the parameter type.
fn bind_columns<'q>(
    query: PgQuery<'q>,
    slots: &'q [Option<ColumnValue>],
    columns: &[ColumnDef],
) -> PgQuery<'q> {
    slots
        .iter()
        .zip(columns)
        .fold(query, |query, (slot, column)| match slot {
            Some(ColumnValue::Text(value)) => query.bind(value),
            Some(ColumnValue::BigInt(value)) => query.bind(*value),
            Some(ColumnValue::Double(value)) => query.bind(*value),
            None => match column.column_type {
                ColumnType::Text => query.bind(None::<String>),
                ColumnType::BigInt => query.bind(None::<i64>),
                ColumnType::Double => query.bind(None::<f64>),
            },
        })
}

/// `($1, $2, ...), ($n, ...)` for a multi-row `VALUES`.
fn values_clause(rows: usize, per_row: usize) -> String {
    (0..rows).fold(String::new(), |values, row| {
        let separator = if row > 0 { ", " } else { "" };
        let values = (0..per_row).fold(values + separator + "(", |mut values, offset| {
            if offset > 0 {
                values.push_str(", ");
            }
            let _ = write!(values, "${}", row * per_row + offset + 1);
            values
        });
        values + ")"
    })
}

async fn insert_type_rows(
    conn: &mut sqlx::PgConnection,
    schema: &ResourceTypeSchema,
    rows: &[TypeRow<'_>],
) -> Result<(), OperationOutcomeError> {
    let per_row = TYPE_TABLE_FIXED_COLUMNS + schema.columns.len();
    // Column names are `[a-z0-9_]` (see `code_to_column_base`); no escaping
    // needed.
    let column_list =
        schema
            .columns
            .iter()
            .fold(String::from("res_key, scope"), |mut list, column| {
                let _ = write!(list, ", \"{}\"", column.name);
                list
            });

    for chunk in rows.chunks((MAX_BIND_PARAMS / per_row).max(1)) {
        let sql = format!(
            "INSERT INTO {} ({column_list}) VALUES {}",
            schema.table_name,
            values_clause(chunk.len(), per_row),
        );

        let query = chunk.iter().fold(sqlx::query(&sql), |query, row| {
            let query = query
                .bind(row.res_key)
                .bind(keys::scope_key(&row.key.tenant, &row.key.project));
            bind_columns(query, row.slots, &schema.columns)
        });

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

#[cfg(test)]
mod tests {
    use super::*;
    use haste_repository::types::SupportedFHIRVersions;

    /// The columns of `INSERT INTO <table> (...)`.
    fn inserted_columns(sql: &str) -> Vec<String> {
        let open = sql.find('(').expect("insert names its columns");
        let close = sql[open..].find(')').expect("unterminated column list") + open;
        sql[open + 1..close]
            .split(',')
            .map(|column| column.trim().to_string())
            .collect()
    }

    /// The columns of the `CREATE TABLE`, the DDL's first statement.
    fn declared_columns(ddl: &str) -> Vec<String> {
        let open = ddl.find('(').expect("create table names its columns");
        let close = ddl.find("\n);").expect("unterminated create table");
        ddl[open + 1..close]
            .split(',')
            .filter_map(|line| line.split_whitespace().next().map(str::to_string))
            .collect()
    }

    /// Each insert must name the same columns, in the same order, as its
    /// `CREATE TABLE`, and bind each at its declared type.
    #[test]
    fn every_shared_insert_matches_its_table_definition() {
        for table in SHARED_TABLES {
            let name = shared_table_name(&SupportedFHIRVersions::R4, table);
            let expected: Vec<String> = SHARED_KEY_COLUMNS
                .iter()
                .map(|column| (*column).to_string())
                .chain(
                    shared_value_columns(table)
                        .iter()
                        .map(|column| column.name.to_string()),
                )
                .collect();

            let sql = dynamic_insert_sql(&name, table);
            assert_eq!(
                inserted_columns(&sql),
                expected,
                "{name}: the insert's columns"
            );
            assert_eq!(
                declared_columns(&crate::pg_search::migration::shared_table_sql_for_test(
                    SupportedFHIRVersions::R4,
                    table,
                )),
                expected,
                "{name}: the table's columns"
            );

            for (offset, column) in shared_value_columns(table).iter().enumerate() {
                assert!(
                    sql.contains(&format!(
                        "${}::{}[]",
                        offset + SHARED_KEY_COLUMNS.len() + 1,
                        column.sql_type
                    )),
                    "{name}: {} is bound at the wrong position or type",
                    column.name
                );
            }
        }
    }
}
