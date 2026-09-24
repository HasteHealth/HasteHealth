//! Batch indexing into the PG search tables.
//!
//! Two phases, which is what keeps a batch off per-row round-trips:
//!
//! 1. **Convert** — every resource's `FHIRPath` expressions are evaluated and
//!    flattened into the rows it will occupy, in parallel and with no database
//!    involved. Everything that can fail for one resource fails here, where it
//!    can be attributed to that resource alone.
//! 2. **Write** — set-based: one statement per table for the whole batch. A
//!    thousand-resource batch is roughly fifteen statements, not ten thousand.
//!
//! The split is what makes the single transaction correct. Postgres aborts a
//! transaction on the first failed statement, so there is no per-resource
//! recovery inside one: a phase 2 failure fails the batch and the worker
//! retries it.

use std::collections::HashMap;
use std::fmt::Write as _;
use std::sync::Arc;

use haste_fhir_model::r4::generated::terminology::IssueType;
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhirpath::FPEngine;
use haste_repository::types::FHIRMethod;
use sqlx::{Pool, Postgres, Row};

use super::{
    PgSearchError, keys, resource_to_search_index,
    schema::{ColumnType, ParamColumns, ResourceTypeSchema, SchemaRegistry, SharedTable},
};
use crate::{
    IndexFailure, IndexOutcome, IndexResource, ResolvedParameter, SearchParameterResolve,
    indexing_conversion::{
        DateRange, DynamicParameterEntry, InsertableIndex, QuantityRange, ReferenceIndex,
        TokenIndex,
    },
};

/// Postgres caps one extended-query message at 65535 binds. Only the
/// per-resource-type insert can approach it; every other statement binds a
/// fixed number of arrays however many rows it writes.
const MAX_BIND_PARAMS: usize = 65535;

/// `res_key, scope`, ahead of a per-resource-type table's generated columns.
const SYSTEM_FIXED_COLUMNS: usize = 2;

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

    // A batch can carry several versions of one resource (a create and a later
    // update in the same sequence window), which would collide on the anchor's
    // primary key. Only the final state belongs in the index, so earlier
    // versions are set aside and accounted for with the one superseding them.
    let (resources, superseded) = dedupe_keep_last(resources);

    // Resolving awaits, and on a cold project cache it queries the database.
    // Once per (tenant, project, resource type) keeps that off the per-resource
    // path.
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

/// The anchor table's primary key, which the upsert maps to the `res_key` every
/// other table is keyed by.
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

/// Keeps each resource's last occurrence, returning the versions it displaced
/// so they can share their survivor's outcome.
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

/// One resource's rows, ready to bind. Everything fallible is already done.
struct Converted {
    key: ResourceKey,
    version_id: String,
    sequence: i64,
    /// `None` for a delete, whose row set is just the anchor removal.
    write: Option<ConvertedWrite>,
}

struct ConvertedWrite {
    /// One slot per column in the resource type's schema, empty where the
    /// resource produced no value. `None` when the type has no table.
    system_row: Option<Vec<Option<ColumnValues>>>,
    /// One slot per anchor column, for the resource-level parameters.
    anchor_row: Vec<Option<ColumnValues>>,
    /// Values for the shared tables, keyed by the canonical URL their identity
    /// hashes from.
    dynamic: Vec<(String, InsertableIndex)>,
}

/// One column's value, ready to bind. A column exists only where the parameter
/// cannot repeat, so there is one value or none.
enum ColumnValues {
    Text(String),
    BigInt(i64),
    Double(f64),
}

/// Identifies the parameter set a resource is indexed against.
type GroupKey = (String, String, String);

fn group_key(resource: &IndexResource) -> GroupKey {
    (
        resource.tenant.as_ref().to_string(),
        resource.project.as_ref().to_string(),
        resource.resource_type.as_ref().to_string(),
    )
}

/// The parameters for one (tenant, project, resource type), plus the `code` →
/// URL lookup that routing a column-less value to the shared tables needs.
struct ParameterSet {
    parameters: Vec<ResolvedParameter>,
    url_by_code: HashMap<String, String>,
}

/// Resolves once per distinct (tenant, project, resource type), not per
/// resource.
async fn resolve_parameter_sets<Resolver: SearchParameterResolve>(
    resolver: &Resolver,
    resources: &[IndexResource],
) -> Result<HashMap<GroupKey, Arc<ParameterSet>>, OperationOutcomeError> {
    let mut sets: HashMap<GroupKey, Arc<ParameterSet>> = HashMap::new();

    for resource in resources {
        // A delete evaluates no expression, so resolving for it would only risk
        // a round-trip nothing reads.
        if matches!(resource.fhir_method, FHIRMethod::Delete) {
            continue;
        }

        let group = group_key(resource);
        if sets.contains_key(&group) {
            continue;
        }

        let parameters = resolver
            .by_resource_type(&resource.tenant, &resource.project, &resource.resource_type)
            .await?;

        // Shared-table rows are told apart by a hash of the parameter's URL, so
        // two sharing one would answer each other's searches.
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

        sets.insert(
            group,
            Arc::new(ParameterSet {
                parameters,
                url_by_code,
            }),
        );
    }

    Ok(sets)
}

/// Converts every resource in parallel, keeping one resource's failure (a bad
/// `FHIRPath` expression, an unsupported method) attributed to it rather than
/// aborting the batch. A `JoinError` has no resource to attribute to, so it
/// aborts the whole call.
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
            .remove(&ResourceKey::from_resource(&resource))
            .unwrap_or_default();

        match result {
            // A clean index means the resource reached its final state, so the
            // displaced versions count as handled too.
            Ok(entry) => converted.push(entry),

            Err(error) => {
                failed.extend(displaced.into_iter().map(|stale| {
                    IndexFailure {
                        resource: stale,
                        error: OperationOutcomeError::fatal(
                            IssueType::exception(),
                            "Superseded by a later version in the same batch that failed to index."
                                .to_string(),
                        ),
                    }
                }));
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

    let write = match &resource.fhir_method {
        FHIRMethod::Create | FHIRMethod::Update => {
            let Some(set) = parameter_set else {
                return Err(OperationOutcomeError::fatal(
                    IssueType::exception(),
                    format!(
                        "No search parameters resolved for resource type '{}'.",
                        key.resource_type
                    ),
                ));
            };

            let index = resource_to_search_index(
                fp_engine.clone(),
                &set.parameters,
                &resource.resource,
                resource.resource_type.as_ref(),
            )
            .await?;

            let schema = schema_registry.get(&key.resource_type);

            // Singular system-level parameters: one row of columns on the
            // per-resource-type table.
            let system_row = schema.map(|schema| system_row(schema, &index.system_entries));

            // Resource-level parameters are anchor columns, so they collect
            // against the anchor schema from the same entries.
            let anchor_row = self::system_row(schema_registry.anchor(), &index.system_entries);

            // Project-level parameters go to the shared tables.
            let mut dynamic = index.dynamic_entries;

            // A single-valued parameter no column took (a name collision, or a
            // type with no table) goes to the shared tables, which a search for
            // it reads too. Those rows are keyed by URL, so the code has to be
            // resolved back through the parameter set only this phase holds.
            for (code, insertable) in index.system_entries {
                let has_column = schema_registry.anchor().columns_for(&code).is_some()
                    || schema.is_some_and(|schema| schema.columns_for(&code).is_some());

                if !has_column && let Some(param_url) = set.url_by_code.get(&code) {
                    dynamic.push((param_url.clone(), insertable));
                }
            }

            Some(ConvertedWrite {
                system_row,
                anchor_row,
                dynamic,
            })
        }

        FHIRMethod::Delete => None,

        method @ FHIRMethod::Read => {
            return Err(OperationOutcomeError::from(
                PgSearchError::UnsupportedFHIRMethod((*method).clone()),
            ));
        }
    };

    Ok(Converted {
        key,
        version_id,
        sequence: resource.sequence,
        write,
    })
}

/// Lays one resource's system-level values out by column position, so the whole
/// batch goes in as one fixed-column statement.
fn system_row(
    schema: &ResourceTypeSchema,
    entries: &[(String, InsertableIndex)],
) -> Vec<Option<ColumnValues>> {
    let mut slots: Vec<Option<ColumnValues>> = (0..schema.columns.len()).map(|_| None).collect();

    for (code, insertable) in entries {
        // A parameter with no generated column has nowhere to go here.
        if let Some(param_columns) = schema.columns_for(code) {
            collect_column_values(schema, param_columns, insertable, &mut slots);
        }
    }

    slots
}

/// The single value a column holds, or nothing. Only the first is taken: a
/// column exists only where the cardinality analysis ruled out a second, so one
/// would mean the classification and the data disagree.
fn text_column<'a>(mut values: impl Iterator<Item = Option<&'a str>>) -> Option<ColumnValues> {
    values
        .next()
        .flatten()
        .map(|value| ColumnValues::Text(value.to_string()))
}

fn bigint_column(mut values: impl Iterator<Item = i64>) -> Option<ColumnValues> {
    values.next().map(ColumnValues::BigInt)
}

fn double_column(mut values: impl Iterator<Item = f64>) -> Option<ColumnValues> {
    values.next().map(ColumnValues::Double)
}

/// Fills the slots for one parameter's columns. Multi-part types (token, date,
/// reference, quantity) write each part to its own column of the same row.
///
/// A declared type that does not match the evaluated value's — the schema and
/// the conversion disagreeing — writes nothing rather than half a row.
fn collect_column_values(
    schema: &ResourceTypeSchema,
    param_columns: &ParamColumns,
    insertable: &InsertableIndex,
    slots: &mut [Option<ColumnValues>],
) {
    let mut set = |name: &str, value: Option<ColumnValues>| {
        if let (Some(index), Some(value)) = (schema.column_index(name), value) {
            slots[index] = Some(value);
        }
    };

    match (param_columns, insertable) {
        (ParamColumns::String { value }, InsertableIndex::String(strings)) => {
            set(value, text_column(strings.iter().map(|s| Some(s.as_str()))));
        }

        (ParamColumns::Uri { value }, InsertableIndex::URI(uris)) => {
            set(value, text_column(uris.iter().map(|u| Some(u.as_str()))));
        }

        (ParamColumns::Number { value }, InsertableIndex::Number(numbers)) => {
            set(value, double_column(numbers.iter().copied()));
        }

        (ParamColumns::Token { system, code }, InsertableIndex::Token(tokens)) => {
            // `_id` has no system column: it reads the anchor's key.
            if let Some(system) = system {
                set(system, text_column(tokens.iter().map(TokenIndex::system)));
            }
            set(code, text_column(tokens.iter().map(TokenIndex::code)));
        }

        (ParamColumns::Date { start, end }, InsertableIndex::Date(ranges)) => {
            set(start, bigint_column(ranges.iter().map(|d| d.start)));
            set(end, bigint_column(ranges.iter().map(|d| d.end)));
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
                text_column(references.iter().map(ReferenceIndex::resource_type)),
            );
            set(
                target_id,
                text_column(references.iter().map(ReferenceIndex::id)),
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
                double_column(quantities.iter().map(QuantityRange::start_value)),
            );
            set(
                end,
                double_column(quantities.iter().map(QuantityRange::end_value)),
            );
            set(
                system,
                text_column(quantities.iter().map(QuantityRange::start_system)),
            );
            set(
                code,
                text_column(quantities.iter().map(QuantityRange::start_code)),
            );
        }

        _ => {}
    }
}

// ---------------------------------------------------------------------------
// Shared-table rows
// ---------------------------------------------------------------------------
//
// Rows accumulate as one array per column, the shape the insert binds: a whole
// table's worth goes in as a handful of arrays through `unnest`. A resource
// builds its own `DynamicBatch` while converting, and the write phase merges
// them by concatenating the arrays.

/// The resource a shared-table row belongs to and the parameter it is a value
/// of.
#[derive(Clone, Copy)]
struct RowKey {
    res_key: i64,
    param_identity: i64,
}

/// The (`res_key`, `param_identity`) prefix every shared-table row carries.
#[derive(Default)]
struct KeyColumns {
    res_key: Vec<i64>,
    param_identity: Vec<i64>,
}

impl KeyColumns {
    fn push(&mut self, key: RowKey) {
        self.res_key.push(key.res_key);
        self.param_identity.push(key.param_identity);
    }

    fn is_empty(&self) -> bool {
        self.res_key.is_empty()
    }
}

type PgQuery<'a> = sqlx::query::Query<'a, Postgres, sqlx::postgres::PgArguments>;

/// The key columns every shared-table insert leads with, bound as `$1`, `$2`.
const SHARED_KEY_COLUMNS: [&str; 2] = ["res_key", "param_identity"];

/// One shared table's rows for the whole batch, held column-wise: the shape the
/// insert binds through `unnest`.
trait SharedRows {
    fn keys(&self) -> &KeyColumns;

    /// Binds the value arrays, in `SharedTable::value_columns()` order.
    fn bind<'a>(&'a self, query: PgQuery<'a>) -> PgQuery<'a>;
}

/// `INSERT INTO <table> (<keys>, <values>) SELECT * FROM unnest($1…$n)`.
///
/// Every bind is cast to its column's array type: an all-NULL column gives
/// Postgres nothing to infer it from.
fn dynamic_insert_sql(table: &str, shared_table: SharedTable) -> String {
    let mut columns = SHARED_KEY_COLUMNS.join(", ");
    let mut arrays = String::from("$1::int8[], $2::int8[]");

    for (offset, column) in shared_table.value_columns().iter().enumerate() {
        let _ = write!(columns, ", {}", column.name);
        let _ = write!(
            arrays,
            ", ${}::{}[]",
            offset + SHARED_KEY_COLUMNS.len() + 1,
            column.sql_type
        );
    }

    format!("INSERT INTO {table} ({columns}) SELECT * FROM unnest({arrays})")
}

/// Writes one shared table's rows, or nothing when it has none.
async fn insert_shared_rows(
    conn: &mut sqlx::PgConnection,
    registry: &SchemaRegistry,
    table: SharedTable,
    rows: &impl SharedRows,
) -> Result<(), OperationOutcomeError> {
    let keys = rows.keys();
    if keys.is_empty() {
        return Ok(());
    }

    let sql = dynamic_insert_sql(&registry.shared_table_name(table), table);
    let query = sqlx::query(&sql)
        .bind(&keys.res_key)
        .bind(&keys.param_identity);

    rows.bind(query)
        .execute(conn)
        .await
        .map_err(PgSearchError::from)?;
    Ok(())
}

/// The string and uri tables, which share a shape.
#[derive(Default)]
struct TextRows {
    keys: KeyColumns,
    value: Vec<String>,
}

impl TextRows {
    fn push(&mut self, key: RowKey, value: &str) {
        self.keys.push(key);
        self.value.push(value.to_string());
    }
}

impl SharedRows for TextRows {
    fn keys(&self) -> &KeyColumns {
        &self.keys
    }

    fn bind<'a>(&'a self, query: PgQuery<'a>) -> PgQuery<'a> {
        query.bind(&self.value)
    }
}

#[derive(Default)]
struct NumberRows {
    keys: KeyColumns,
    value: Vec<f64>,
}

impl NumberRows {
    fn push(&mut self, key: RowKey, value: f64) {
        self.keys.push(key);
        self.value.push(value);
    }
}

impl SharedRows for NumberRows {
    fn keys(&self) -> &KeyColumns {
        &self.keys
    }

    fn bind<'a>(&'a self, query: PgQuery<'a>) -> PgQuery<'a> {
        query.bind(&self.value)
    }
}

#[derive(Default)]
struct TokenRows {
    keys: KeyColumns,
    system: Vec<Option<String>>,
    code: Vec<Option<String>>,
}

impl TokenRows {
    fn push(&mut self, key: RowKey, token: &TokenIndex) {
        self.keys.push(key);
        self.system.push(token.system().map(str::to_string));
        self.code.push(token.code().map(str::to_string));
    }
}

impl SharedRows for TokenRows {
    fn keys(&self) -> &KeyColumns {
        &self.keys
    }

    fn bind<'a>(&'a self, query: PgQuery<'a>) -> PgQuery<'a> {
        query.bind(&self.system).bind(&self.code)
    }
}

#[derive(Default)]
struct DateRows {
    keys: KeyColumns,
    start_ms: Vec<i64>,
    end_ms: Vec<i64>,
}

impl DateRows {
    fn push(&mut self, key: RowKey, range: &DateRange) {
        self.keys.push(key);
        self.start_ms.push(range.start);
        self.end_ms.push(range.end);
    }
}

impl SharedRows for DateRows {
    fn keys(&self) -> &KeyColumns {
        &self.keys
    }

    fn bind<'a>(&'a self, query: PgQuery<'a>) -> PgQuery<'a> {
        query.bind(&self.start_ms).bind(&self.end_ms)
    }
}

#[derive(Default)]
struct ReferenceRows {
    keys: KeyColumns,
    target_resource_type: Vec<Option<String>>,
    target_id: Vec<Option<String>>,
}

impl ReferenceRows {
    fn push(&mut self, key: RowKey, reference: &ReferenceIndex) {
        self.keys.push(key);
        self.target_resource_type
            .push(reference.resource_type().map(str::to_string));
        self.target_id.push(reference.id().map(str::to_string));
    }
}

impl SharedRows for ReferenceRows {
    fn keys(&self) -> &KeyColumns {
        &self.keys
    }

    fn bind<'a>(&'a self, query: PgQuery<'a>) -> PgQuery<'a> {
        query.bind(&self.target_resource_type).bind(&self.target_id)
    }
}

#[derive(Default)]
struct QuantityRows {
    keys: KeyColumns,
    start_value: Vec<f64>,
    end_value: Vec<f64>,
    start_system: Vec<Option<String>>,
    start_code: Vec<Option<String>>,
}

impl QuantityRows {
    fn push(&mut self, key: RowKey, quantity: &QuantityRange) {
        self.keys.push(key);
        self.start_value.push(quantity.start_value());
        self.end_value.push(quantity.end_value());
        self.start_system
            .push(quantity.start_system().map(str::to_string));
        self.start_code
            .push(quantity.start_code().map(str::to_string));
    }
}

impl SharedRows for QuantityRows {
    fn keys(&self) -> &KeyColumns {
        &self.keys
    }

    fn bind<'a>(&'a self, query: PgQuery<'a>) -> PgQuery<'a> {
        query
            .bind(&self.start_value)
            .bind(&self.end_value)
            .bind(&self.start_system)
            .bind(&self.start_code)
    }
}

/// The whole batch's rows for the shared EAV tables.
#[derive(Default)]
struct DynamicBatch {
    string: TextRows,
    uri: TextRows,
    number: NumberRows,
    token: TokenRows,
    date: DateRows,
    reference: ReferenceRows,
    quantity: QuantityRows,
}

impl DynamicBatch {
    /// Routes one parameter's values to the table for its type, keyed to the
    /// resource `res_key` identifies.
    fn collect(
        &mut self,
        res_key: i64,
        resource: &ResourceKey,
        param_url: &str,
        insertable: &InsertableIndex,
    ) {
        let key = RowKey {
            res_key,
            param_identity: keys::param_identity(&resource.tenant, &resource.project, param_url),
        };

        match insertable {
            InsertableIndex::String(values) => {
                for value in values {
                    self.string.push(key, value);
                }
            }
            InsertableIndex::URI(values) => {
                for value in values {
                    self.uri.push(key, value);
                }
            }
            InsertableIndex::Number(values) => {
                for value in values {
                    self.number.push(key, *value);
                }
            }
            InsertableIndex::Token(tokens) => {
                for token in tokens {
                    self.token.push(key, token);
                }
            }
            InsertableIndex::Date(ranges) => {
                for range in ranges {
                    self.date.push(key, range);
                }
            }
            InsertableIndex::Reference(references) => {
                for reference in references {
                    self.reference.push(key, reference);
                }
            }
            InsertableIndex::Quantity(quantities) => {
                for quantity in quantities {
                    self.quantity.push(key, quantity);
                }
            }

            // A project-level parameter carries its own URL and a slot per
            // type, so each entry routes independently.
            InsertableIndex::DynamicParameters(entries) => {
                for entry in entries {
                    self.collect_dynamic_parameter(res_key, resource, entry);
                }
            }

            // Composite and Special have no PG representation; Meta is internal
            // to the Elasticsearch document shape.
            InsertableIndex::Composite(_)
            | InsertableIndex::Special(_)
            | InsertableIndex::Meta(_) => {}
        }
    }

    fn collect_dynamic_parameter(
        &mut self,
        res_key: i64,
        resource: &ResourceKey,
        entry: &DynamicParameterEntry,
    ) {
        let key = RowKey {
            res_key,
            param_identity: keys::param_identity(&resource.tenant, &resource.project, &entry.url),
        };
        let value = &entry.value;

        for item in value.string.iter().flatten() {
            self.string.push(key, item);
        }
        for item in value.uri.iter().flatten() {
            self.uri.push(key, item);
        }
        for item in value.number.iter().flatten() {
            self.number.push(key, *item);
        }
        for item in value.token.iter().flatten() {
            self.token.push(key, item);
        }
        for item in value.date.iter().flatten() {
            self.date.push(key, item);
        }
        for item in value.reference.iter().flatten() {
            self.reference.push(key, item);
        }
        for item in value.quantity.iter().flatten() {
            self.quantity.push(key, item);
        }
    }

    /// One statement per table, each binding a fixed set of arrays.
    async fn insert(
        &self,
        conn: &mut sqlx::PgConnection,
        registry: &SchemaRegistry,
    ) -> Result<(), OperationOutcomeError> {
        insert_shared_rows(&mut *conn, registry, SharedTable::String, &self.string).await?;
        insert_shared_rows(&mut *conn, registry, SharedTable::Uri, &self.uri).await?;
        insert_shared_rows(&mut *conn, registry, SharedTable::Number, &self.number).await?;
        insert_shared_rows(&mut *conn, registry, SharedTable::Token, &self.token).await?;
        insert_shared_rows(&mut *conn, registry, SharedTable::Date, &self.date).await?;
        insert_shared_rows(
            &mut *conn,
            registry,
            SharedTable::Reference,
            &self.reference,
        )
        .await?;
        insert_shared_rows(&mut *conn, registry, SharedTable::Quantity, &self.quantity).await?;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Phase 2: write
// ---------------------------------------------------------------------------

/// One resource's row on its per-resource-type table.
struct SystemRow<'a> {
    res_key: i64,
    key: &'a ResourceKey,
    slots: &'a [Option<ColumnValues>],
}

/// Writes the whole converted batch in one transaction.
///
/// The anchor goes first, since its upsert hands out each `res_key`, keeps an
/// existing resource's, and reports whether it was already indexed. Only those
/// and the deletes have rows elsewhere to clear, so a batch of new resources
/// skips the deletes entirely.
///
/// All-or-nothing by necessity: Postgres aborts on the first failed statement,
/// so there is no per-resource recovery. A failure returns `Err`, the worker
/// leaves its sequence lock, and the batch is retried.
async fn write_batch(
    pool: &Pool<Postgres>,
    schema_registry: &SchemaRegistry,
    mut converted: Vec<Converted>,
) -> Result<(), OperationOutcomeError> {
    if converted.is_empty() {
        return Ok(());
    }

    // Overlapping batches would otherwise lock the same anchor rows in
    // different orders. One order for every writer keeps them queuing rather
    // than deadlocking.
    converted.sort_by(|a, b| a.key.cmp(&b.key));

    let (deletes, writes): (Vec<&Converted>, Vec<&Converted>) =
        converted.iter().partition(|entry| entry.write.is_none());

    let mut tx = pool.begin().await.map_err(|e| {
        OperationOutcomeError::fatal(
            IssueType::exception(),
            format!("Failed to begin PG search transaction: {e}"),
        )
    })?;

    // To clear: every deleted resource's rows, and every re-indexed one's
    // previous version.
    let mut stale = delete_anchors(tx.as_mut(), schema_registry, &deletes).await?;

    let keys = upsert_anchors(tx.as_mut(), schema_registry, &writes).await?;
    stale.extend(
        keys.values()
            .filter(|anchor| anchor.existed)
            .map(|anchor| (anchor.res_key, anchor.resource_type.clone())),
    );

    clear_rows(tx.as_mut(), schema_registry, &stale).await?;

    // A resource the upsert did not return already has a newer version indexed
    // and keeps it, rows and all.
    let mut dynamic = DynamicBatch::default();
    let mut system_rows: HashMap<&str, Vec<SystemRow<'_>>> = HashMap::new();
    for entry in &writes {
        let (Some(write), Some(anchor)) = (entry.write.as_ref(), keys.get(&entry.key)) else {
            continue;
        };

        for (param_url, insertable) in &write.dynamic {
            dynamic.collect(anchor.res_key, &entry.key, param_url, insertable);
        }

        if let Some(slots) = write.system_row.as_ref() {
            system_rows
                .entry(entry.key.resource_type.as_str())
                .or_default()
                .push(SystemRow {
                    res_key: anchor.res_key,
                    key: &entry.key,
                    slots,
                });
        }
    }

    for (resource_type, rows) in &system_rows {
        if let Some(schema) = schema_registry.get(resource_type) {
            insert_system_rows(tx.as_mut(), schema, rows).await?;
        }
    }

    dynamic.insert(tx.as_mut(), schema_registry).await?;

    tx.commit().await.map_err(|e| {
        OperationOutcomeError::fatal(
            IssueType::exception(),
            format!("Failed to commit PG search transaction: {e}"),
        )
    })?;

    Ok(())
}

/// A resource's anchor row as the upsert left it.
struct AnchorKey {
    res_key: i64,
    resource_type: String,
    /// Indexed before this batch, so its previous version's rows are still in
    /// place.
    existed: bool,
}

/// Removes every deleted resource's anchor row, returning the keys whose rows
/// elsewhere go with it. A delete older than the indexed version is a replay
/// and leaves it in place.
async fn delete_anchors(
    conn: &mut sqlx::PgConnection,
    schema_registry: &SchemaRegistry,
    deletes: &[&Converted],
) -> Result<Vec<(i64, String)>, OperationOutcomeError> {
    if deletes.is_empty() {
        return Ok(Vec::new());
    }

    let mut tenant = Vec::with_capacity(deletes.len());
    let mut project = Vec::with_capacity(deletes.len());
    let mut resource_type = Vec::with_capacity(deletes.len());
    let mut resource_id = Vec::with_capacity(deletes.len());
    let mut sequence = Vec::with_capacity(deletes.len());
    for entry in deletes {
        tenant.push(entry.key.tenant.as_str());
        project.push(entry.key.project.as_str());
        resource_type.push(entry.key.resource_type.as_str());
        resource_id.push(entry.key.resource_id.as_str());
        sequence.push(entry.sequence);
    }

    let sql = format!(
        "DELETE FROM {anchor} sr \
         USING unnest($1::text[], $2::text[], $3::text[], $4::text[], $5::int8[]) \
             AS k(tenant, project, resource_type, resource_id, sequence) \
         WHERE sr.tenant = k.tenant AND sr.project = k.project \
             AND sr.resource_type = k.resource_type AND sr.resource_id = k.resource_id \
             AND sr.sequence <= k.sequence \
         RETURNING sr.res_key, sr.resource_type",
        anchor = schema_registry.resource_table_name(),
    );

    let rows = sqlx::query(&sql)
        .bind(&tenant)
        .bind(&project)
        .bind(&resource_type)
        .bind(&resource_id)
        .bind(&sequence)
        .fetch_all(conn)
        .await
        .map_err(PgSearchError::from)?;

    Ok(rows
        .iter()
        .map(|row| (row.get("res_key"), row.get("resource_type")))
        .collect())
}

/// Inserts or updates every written resource's anchor row, returning each
/// `res_key`.
///
/// An existing resource keeps its key, so a re-index only rewrites the rows
/// hanging off it. A newer indexed version skips the update and drops the
/// resource from the result, so a replay cannot roll it back; the same version
/// passes, which is what lets a reset reindex rewrite everything.
///
/// The anchor also carries the resource-level parameters' columns, bound the
/// same way the resource type insert binds its own.
async fn upsert_anchors(
    conn: &mut sqlx::PgConnection,
    schema_registry: &SchemaRegistry,
    writes: &[&Converted],
) -> Result<HashMap<ResourceKey, AnchorKey>, OperationOutcomeError> {
    let mut keys = HashMap::with_capacity(writes.len());
    if writes.is_empty() {
        return Ok(keys);
    }

    let anchor = schema_registry.anchor();
    let per_row = ANCHOR_FIXED_COLUMNS.len() + anchor.columns.len();

    let mut column_list = ANCHOR_FIXED_COLUMNS.join(", ");
    let mut updates =
        String::from("version_id = EXCLUDED.version_id, sequence = EXCLUDED.sequence");
    for column in &anchor.columns {
        let _ = write!(column_list, ", \"{}\"", column.name);
        let _ = write!(updates, ", \"{0}\" = EXCLUDED.\"{0}\"", column.name);
    }

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

        let mut query = sqlx::query(&sql);
        for entry in chunk {
            query = query
                .bind(&entry.key.tenant)
                .bind(&entry.key.project)
                .bind(&entry.key.resource_type)
                .bind(&entry.key.resource_id)
                .bind(&entry.version_id)
                .bind(entry.sequence);

            if let Some(write) = entry.write.as_ref() {
                query = bind_columns(query, &write.anchor_row, &anchor.columns);
            }
        }

        let rows = query.fetch_all(&mut *conn).await.map_err(|e| {
            OperationOutcomeError::fatal(
                IssueType::exception(),
                format!(
                    "Failed to upsert anchor rows into {}: {e}",
                    anchor.table_name
                ),
            )
        })?;

        for row in rows {
            let key = ResourceKey {
                tenant: row.get("tenant"),
                project: row.get("project"),
                resource_type: row.get("resource_type"),
                resource_id: row.get("resource_id"),
            };
            let inserted: bool = row.get("inserted");
            keys.insert(
                key.clone(),
                AnchorKey {
                    res_key: row.get("res_key"),
                    resource_type: key.resource_type,
                    existed: !inserted,
                },
            );
        }
    }

    Ok(keys)
}

/// The anchor's own columns, ahead of its generated ones. Not `res_key`, which
/// the anchor allocates.
const ANCHOR_FIXED_COLUMNS: [&str; 6] = [
    "tenant",
    "project",
    "resource_type",
    "resource_id",
    "version_id",
    "sequence",
];

/// Clears these resources' rows outside the anchor: their shared-table rows and
/// their per-resource-type row.
///
/// Every lookup is by `res_key` on a `BIGINT` index, and nothing runs when
/// there is nothing to clear — the common case of a batch of new resources.
///
/// No foreign keys to cascade through: one would fire a referential-integrity
/// trigger for ~145 tables per deleted row, so each table gets its own
/// statement instead.
async fn clear_rows(
    conn: &mut sqlx::PgConnection,
    schema_registry: &SchemaRegistry,
    stale: &[(i64, String)],
) -> Result<(), OperationOutcomeError> {
    if stale.is_empty() {
        return Ok(());
    }

    let all: Vec<i64> = stale.iter().map(|(res_key, _)| *res_key).collect();
    for table in SharedTable::ALL {
        delete_by_res_key(&mut *conn, &schema_registry.shared_table_name(table), &all).await?;
    }

    let mut by_type: HashMap<&str, Vec<i64>> = HashMap::new();
    for (res_key, resource_type) in stale {
        by_type
            .entry(resource_type.as_str())
            .or_default()
            .push(*res_key);
    }

    for (resource_type, res_keys) in by_type {
        // A resource type with no generated table has no row to clear, and the
        // registry has to say so: probing and ignoring the error would abort
        // the transaction.
        if let Some(schema) = schema_registry.get(resource_type) {
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

/// Binds one row's generated column values, in schema order. An absent value
/// still carries the column's type, or Postgres cannot infer the bind.
fn bind_columns<'q>(
    mut query: sqlx::query::Query<'q, sqlx::Postgres, sqlx::postgres::PgArguments>,
    slots: &'q [Option<ColumnValues>],
    columns: &[super::schema::ColumnDef],
) -> sqlx::query::Query<'q, sqlx::Postgres, sqlx::postgres::PgArguments> {
    for (slot, column) in slots.iter().zip(columns) {
        query = match slot {
            Some(ColumnValues::Text(value)) => query.bind(value),
            Some(ColumnValues::BigInt(value)) => query.bind(*value),
            Some(ColumnValues::Double(value)) => query.bind(*value),
            None => match column.column_type {
                ColumnType::Text => query.bind(None::<String>),
                ColumnType::BigInt => query.bind(None::<i64>),
                ColumnType::Double => query.bind(None::<f64>),
            },
        };
    }
    query
}

/// `($1, $2, ...), ($n, ...)` for a multi-row `VALUES` insert.
fn values_clause(rows: usize, per_row: usize) -> String {
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

    values
}

async fn insert_system_rows(
    conn: &mut sqlx::PgConnection,
    schema: &ResourceTypeSchema,
    rows: &[SystemRow<'_>],
) -> Result<(), OperationOutcomeError> {
    if rows.is_empty() {
        return Ok(());
    }

    let per_row = SYSTEM_FIXED_COLUMNS + schema.columns.len();
    let column_list = system_column_list(schema);

    for chunk in rows.chunks((MAX_BIND_PARAMS / per_row).max(1)) {
        let sql = format!(
            "INSERT INTO {} ({column_list}) VALUES {}",
            schema.table_name,
            values_clause(chunk.len(), per_row),
        );
        let mut query = sqlx::query(&sql);

        for row in chunk {
            query = query
                .bind(row.res_key)
                .bind(keys::scope_key(&row.key.tenant, &row.key.project));

            query = bind_columns(query, row.slots, &schema.columns);
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
    // `code_to_column_base` folds everything outside `[a-z0-9_]`, so a name
    // cannot contain a quote to escape.
    let mut list = String::from("res_key, scope");
    for column in &schema.columns {
        let _ = write!(list, ", \"{}\"", column.name);
    }
    list
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

    /// The insert and the `CREATE TABLE` must name the same columns in the same
    /// order. A list hardcoded on either side — how `target_uri` once came to
    /// be written to a table without it — fails here, not on the first resource
    /// indexed.
    #[test]
    fn every_shared_insert_matches_its_table_definition() {
        for table in SharedTable::ALL {
            let name = table.table_name(&SupportedFHIRVersions::R4);
            let expected: Vec<String> = SHARED_KEY_COLUMNS
                .iter()
                .map(|column| (*column).to_string())
                .chain(
                    table
                        .value_columns()
                        .iter()
                        .map(|column| column.name.to_string()),
                )
                .collect();

            assert_eq!(
                inserted_columns(&dynamic_insert_sql(&name, table)),
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

            // Each value column binds an array of its declared type; the keys
            // take $1 and $2.
            let sql = dynamic_insert_sql(&name, table);
            for (offset, column) in table.value_columns().iter().enumerate() {
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
