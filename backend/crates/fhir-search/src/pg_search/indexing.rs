//! Batch indexing into the PG search tables.
//!
//! Indexing runs in two phases, which is what keeps a batch from degenerating
//! into per-row round-trips:
//!
//! 1. **Convert** — every resource's `FHIRPath` expressions are evaluated and
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
    schema::{ColumnType, ParamColumns, ResourceTypeSchema, SchemaRegistry, SharedTable},
};
use crate::{
    IndexFailure, IndexOutcome, IndexResource, ResolvedParameter, SearchParameterResolve,
    indexing_conversion::{
        DateRange, DynamicParameterEntry, InsertableIndex, QuantityRange, ReferenceIndex,
        TokenIndex,
    },
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
    // would collide on the anchor table's primary key, and only the final
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
    /// One slot per column on the anchor table, for the resource-level
    /// parameters every resource carries.
    anchor_row: Vec<Option<ColumnValues>>,
    /// Values bound for the shared EAV tables, each already keyed by the
    /// canonical URL those tables discriminate on — system-level references
    /// mirrored there included.
    dynamic: Vec<(String, InsertableIndex)>,
}

/// One column's value, ready to bind.
///
/// A column exists only for a parameter that cannot repeat, so a resource has
/// one value for it or none; the empty slot carries the absence.
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

/// The parameter set for one (tenant, project, resource type), plus the
/// `code` → canonical URL lookup the reference mirror needs.
struct ParameterSet {
    parameters: Vec<ResolvedParameter>,
    url_by_code: HashMap<String, String>,
}

/// Resolves parameters once per distinct (tenant, project, resource type) in
/// the batch rather than once per resource.
async fn resolve_parameter_sets<Resolver: SearchParameterResolve>(
    resolver: &Resolver,
    resources: &[IndexResource],
) -> Result<HashMap<GroupKey, Arc<ParameterSet>>, OperationOutcomeError> {
    let mut sets: HashMap<GroupKey, Arc<ParameterSet>> = HashMap::new();

    for resource in resources {
        // A delete never evaluates an expression, so resolving for it would
        // only risk a database round-trip that nothing reads.
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

/// Converts every resource in parallel, keeping a single resource's failure
/// (a bad `FHIRPath` expression, an unsupported method) attributed to it rather
/// than aborting the batch.
///
/// A `JoinError` — the task itself panicked — has no resource to attribute the
/// failure to, so it aborts the whole call.
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
            // The displaced versions are superseded by one that indexed
            // cleanly, so the resource reached its correct final state and
            // every version of it counts as handled.
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

            let index =
                resource_to_search_index(fp_engine.clone(), &set.parameters, &resource.resource)
                    .await?;

            let schema = schema_registry.get(&key.resource_type);

            // Singular system-level parameters go into columns on the
            // per-resource-type table, one row per resource.
            let system_row = schema.map(|schema| system_row(schema, &index.system_entries));

            // Resource-level parameters are columns on the anchor, not on the
            // resource type's table, so they are collected against the anchor
            // schema from the same entries.
            let anchor_row = self::system_row(schema_registry.anchor(), &index.system_entries);

            // Project-level parameters go into the shared EAV tables.
            let mut dynamic = index.dynamic_entries;

            // Two cases send a system-level value to the shared tables too,
            // keyed by URL like every other row there — so the code has to be
            // resolved back to the parameter it came from, which only this
            // phase can do, since it is the one holding the parameter set.
            //
            // - No column took it (a column-name collision, or a resource
            //   type with no table). A search for it finds no column either
            //   and reads the shared table, so that is where it has to be.
            // - It is a reference. Reverse-reference lookups (`_revinclude`,
            //   chained search) scan references across resource types, which
            //   the per-type columns can't serve, so they are mirrored into
            //   the shared reference table to keep one place to scan.
            for (code, insertable) in index.system_entries {
                let has_column = schema_registry.anchor().columns_for(&code).is_some()
                    || schema.is_some_and(|schema| schema.columns_for(&code).is_some());

                if (!has_column || matches!(insertable, InsertableIndex::Reference(_)))
                    && let Some(param_url) = set.url_by_code.get(&code)
                {
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
        write,
    })
}

/// Lays one resource's system-level values out by column position, which is
/// what lets the whole batch go in as one fixed-column statement.
fn system_row(
    schema: &ResourceTypeSchema,
    entries: &[(String, InsertableIndex)],
) -> Vec<Option<ColumnValues>> {
    let mut slots: Vec<Option<ColumnValues>> = (0..schema.columns.len()).map(|_| None).collect();

    for (code, insertable) in entries {
        // A parameter with no generated column (an unmapped type, or one that
        // lost a column-name collision) simply has nowhere to go here.
        if let Some(param_columns) = schema.columns_for(code) {
            collect_column_values(schema, param_columns, insertable, &mut slots);
        }
    }

    slots
}

/// The single value a column holds, or nothing when the resource produced
/// none. Only the first value is taken: a column exists only where the
/// cardinality analysis established there can be no second one, so a second
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

/// Fills the slots for one parameter's columns.
///
/// Multi-part types (token, date, reference, quantity) write every part to
/// its own column of the same row.
///
/// A mismatch between the parameter's declared type and the evaluated value's
/// type (which would mean the schema and the conversion disagree) writes
/// nothing rather than a half-filled set of columns.
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
            set(system, text_column(tokens.iter().map(TokenIndex::system)));
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
// `search_dynamic_*` rows
// ---------------------------------------------------------------------------
//
// Rows are accumulated as one array per column rather than as a list of rows,
// because that is the shape the insert binds: a whole table's worth of rows
// goes in as a handful of arrays through `unnest`, however many rows it holds.
// A resource builds its own `DynamicBatch` during conversion (in parallel),
// and the write phase merges them by concatenating the arrays.

/// The (tenant, project, `resource_type`, `resource_id`, `param_url`) prefix every
/// `search_dynamic_*` row carries.
#[derive(Default)]
struct KeyColumns {
    tenant: Vec<String>,
    project: Vec<String>,
    resource_type: Vec<String>,
    resource_id: Vec<String>,
    param_url: Vec<String>,
}

impl KeyColumns {
    fn push(&mut self, key: &ResourceKey, param_url: &str) {
        self.tenant.push(key.tenant.clone());
        self.project.push(key.project.clone());
        self.resource_type.push(key.resource_type.clone());
        self.resource_id.push(key.resource_id.clone());
        self.param_url.push(param_url.to_string());
    }

    fn is_empty(&self) -> bool {
        self.tenant.is_empty()
    }
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

/// `INSERT INTO <table> (<keys>, <values>) SELECT * FROM unnest($1…$n)`.
///
/// Every parameter is cast to its column's array type. The casts are not
/// decoration: a column whose values are all NULL gives Postgres nothing to
/// infer the array type from.
fn dynamic_insert_sql(table: &str, shared_table: SharedTable) -> String {
    let mut columns = String::from("tenant, project, resource_type, resource_id, param_url");
    let mut arrays = String::from("$1::text[], $2::text[], $3::text[], $4::text[], $5::text[]");

    for (offset, column) in shared_table.value_columns().iter().enumerate() {
        let _ = write!(columns, ", {}", column.name);
        let _ = write!(arrays, ", ${}::{}[]", offset + 6, column.sql_type);
    }

    format!("INSERT INTO {table} ({columns}) SELECT * FROM unnest({arrays})")
}

/// `search_dynamic_string` and `search_dynamic_uri`, which share a shape.
#[derive(Default)]
struct TextRows {
    keys: KeyColumns,
    value: Vec<String>,
}

impl TextRows {
    fn push(&mut self, key: &ResourceKey, param_url: &str, value: &str) {
        self.keys.push(key, param_url);
        self.value.push(value.to_string());
    }

    async fn insert(
        &self,
        conn: &mut sqlx::PgConnection,
        registry: &SchemaRegistry,
        table: SharedTable,
    ) -> Result<(), OperationOutcomeError> {
        if self.keys.is_empty() {
            return Ok(());
        }

        let name = registry.shared_table_name(table);

        let sql = dynamic_insert_sql(&name, table);
        bind_keys(sqlx::query(&sql), &self.keys)
            .bind(&self.value)
            .execute(conn)
            .await
            .map_err(PgSearchError::from)?;
        Ok(())
    }
}

#[derive(Default)]
struct NumberRows {
    keys: KeyColumns,
    value: Vec<f64>,
}

impl NumberRows {
    fn push(&mut self, key: &ResourceKey, param_url: &str, value: f64) {
        self.keys.push(key, param_url);
        self.value.push(value);
    }

    async fn insert(
        &self,
        conn: &mut sqlx::PgConnection,
        registry: &SchemaRegistry,
        table: SharedTable,
    ) -> Result<(), OperationOutcomeError> {
        if self.keys.is_empty() {
            return Ok(());
        }

        let name = registry.shared_table_name(table);

        let sql = dynamic_insert_sql(&name, table);
        bind_keys(sqlx::query(&sql), &self.keys)
            .bind(&self.value)
            .execute(conn)
            .await
            .map_err(PgSearchError::from)?;
        Ok(())
    }
}

#[derive(Default)]
struct TokenRows {
    keys: KeyColumns,
    system: Vec<Option<String>>,
    code: Vec<Option<String>>,
}

impl TokenRows {
    fn push(&mut self, key: &ResourceKey, param_url: &str, token: &TokenIndex) {
        self.keys.push(key, param_url);
        self.system.push(token.system().map(str::to_string));
        self.code.push(token.code().map(str::to_string));
    }

    async fn insert(
        &self,
        conn: &mut sqlx::PgConnection,
        registry: &SchemaRegistry,
        table: SharedTable,
    ) -> Result<(), OperationOutcomeError> {
        if self.keys.is_empty() {
            return Ok(());
        }

        let name = registry.shared_table_name(table);

        let sql = dynamic_insert_sql(&name, table);
        bind_keys(sqlx::query(&sql), &self.keys)
            .bind(&self.system)
            .bind(&self.code)
            .execute(conn)
            .await
            .map_err(PgSearchError::from)?;
        Ok(())
    }
}

#[derive(Default)]
struct DateRows {
    keys: KeyColumns,
    start_ms: Vec<i64>,
    end_ms: Vec<i64>,
}

impl DateRows {
    fn push(&mut self, key: &ResourceKey, param_url: &str, range: &DateRange) {
        self.keys.push(key, param_url);
        self.start_ms.push(range.start);
        self.end_ms.push(range.end);
    }

    async fn insert(
        &self,
        conn: &mut sqlx::PgConnection,
        registry: &SchemaRegistry,
        table: SharedTable,
    ) -> Result<(), OperationOutcomeError> {
        if self.keys.is_empty() {
            return Ok(());
        }

        let name = registry.shared_table_name(table);

        let sql = dynamic_insert_sql(&name, table);
        bind_keys(sqlx::query(&sql), &self.keys)
            .bind(&self.start_ms)
            .bind(&self.end_ms)
            .execute(conn)
            .await
            .map_err(PgSearchError::from)?;
        Ok(())
    }
}

#[derive(Default)]
struct ReferenceRows {
    keys: KeyColumns,
    target_resource_type: Vec<Option<String>>,
    target_id: Vec<Option<String>>,
}

impl ReferenceRows {
    fn push(&mut self, key: &ResourceKey, param_url: &str, reference: &ReferenceIndex) {
        self.keys.push(key, param_url);
        self.target_resource_type
            .push(reference.resource_type().map(str::to_string));
        self.target_id.push(reference.id().map(str::to_string));
    }

    async fn insert(
        &self,
        conn: &mut sqlx::PgConnection,
        registry: &SchemaRegistry,
        table: SharedTable,
    ) -> Result<(), OperationOutcomeError> {
        if self.keys.is_empty() {
            return Ok(());
        }

        let name = registry.shared_table_name(table);

        let sql = dynamic_insert_sql(&name, table);
        bind_keys(sqlx::query(&sql), &self.keys)
            .bind(&self.target_resource_type)
            .bind(&self.target_id)
            .execute(conn)
            .await
            .map_err(PgSearchError::from)?;
        Ok(())
    }
}

#[derive(Default)]
struct QuantityRows {
    keys: KeyColumns,
    start_value: Vec<f64>,
    start_system: Vec<Option<String>>,
    start_code: Vec<Option<String>>,
    end_value: Vec<f64>,
}

impl QuantityRows {
    fn push(&mut self, key: &ResourceKey, param_url: &str, quantity: &QuantityRange) {
        self.keys.push(key, param_url);
        self.start_value.push(quantity.start_value());
        self.start_system
            .push(quantity.start_system().map(str::to_string));
        self.start_code
            .push(quantity.start_code().map(str::to_string));
        self.end_value.push(quantity.end_value());
    }

    async fn insert(
        &self,
        conn: &mut sqlx::PgConnection,
        registry: &SchemaRegistry,
        table: SharedTable,
    ) -> Result<(), OperationOutcomeError> {
        if self.keys.is_empty() {
            return Ok(());
        }

        let name = registry.shared_table_name(table);

        let sql = dynamic_insert_sql(&name, table);
        bind_keys(sqlx::query(&sql), &self.keys)
            .bind(&self.start_value)
            .bind(&self.end_value)
            .bind(&self.start_system)
            .bind(&self.start_code)
            .execute(conn)
            .await
            .map_err(PgSearchError::from)?;
        Ok(())
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
    /// Routes one parameter's evaluated values to the table its type belongs in.
    fn collect(&mut self, key: &ResourceKey, param_url: &str, insertable: &InsertableIndex) {
        match insertable {
            InsertableIndex::String(values) => {
                for value in values {
                    self.string.push(key, param_url, value);
                }
            }
            InsertableIndex::URI(values) => {
                for value in values {
                    self.uri.push(key, param_url, value);
                }
            }
            InsertableIndex::Number(values) => {
                for value in values {
                    self.number.push(key, param_url, *value);
                }
            }
            InsertableIndex::Token(tokens) => {
                for token in tokens {
                    self.token.push(key, param_url, token);
                }
            }
            InsertableIndex::Date(ranges) => {
                for range in ranges {
                    self.date.push(key, param_url, range);
                }
            }
            InsertableIndex::Reference(references) => {
                for reference in references {
                    self.reference.push(key, param_url, reference);
                }
            }
            InsertableIndex::Quantity(quantities) => {
                for quantity in quantities {
                    self.quantity.push(key, param_url, quantity);
                }
            }

            // A project-level parameter carries its own URL and a value slot
            // per type, so each entry routes independently of this one's.
            InsertableIndex::DynamicParameters(entries) => {
                for entry in entries {
                    self.collect_dynamic_parameter(key, entry);
                }
            }

            // Composite and Special have no PG representation yet; Meta is
            // internal to the Elasticsearch document shape.
            InsertableIndex::Composite(_)
            | InsertableIndex::Special(_)
            | InsertableIndex::Meta(_) => {}
        }
    }

    fn collect_dynamic_parameter(&mut self, key: &ResourceKey, entry: &DynamicParameterEntry) {
        let url = entry.url.as_str();
        let value = &entry.value;

        for item in value.string.iter().flatten() {
            self.string.push(key, url, item);
        }
        for item in value.uri.iter().flatten() {
            self.uri.push(key, url, item);
        }
        for item in value.number.iter().flatten() {
            self.number.push(key, url, *item);
        }
        for item in value.token.iter().flatten() {
            self.token.push(key, url, item);
        }
        for item in value.date.iter().flatten() {
            self.date.push(key, url, item);
        }
        for item in value.reference.iter().flatten() {
            self.reference.push(key, url, item);
        }
        for item in value.quantity.iter().flatten() {
            self.quantity.push(key, url, item);
        }
    }

    /// One statement per table, each binding a fixed set of arrays however
    /// many rows it carries.
    async fn insert(
        &self,
        conn: &mut sqlx::PgConnection,
        registry: &SchemaRegistry,
    ) -> Result<(), OperationOutcomeError> {
        self.string
            .insert(&mut *conn, registry, SharedTable::String)
            .await?;
        self.uri
            .insert(&mut *conn, registry, SharedTable::Uri)
            .await?;
        self.number
            .insert(&mut *conn, registry, SharedTable::Number)
            .await?;
        self.token
            .insert(&mut *conn, registry, SharedTable::Token)
            .await?;
        self.date
            .insert(&mut *conn, registry, SharedTable::Date)
            .await?;
        self.reference
            .insert(&mut *conn, registry, SharedTable::Reference)
            .await?;
        self.quantity
            .insert(&mut *conn, registry, SharedTable::Quantity)
            .await?;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Phase 2: write
// ---------------------------------------------------------------------------

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

    // Every resource's EAV values, transposed into the column arrays each
    // table's insert binds. Only Vec pushes — the expensive part (evaluating
    // FHIRPath) already happened in parallel.
    let mut dynamic = DynamicBatch::default();
    for entry in &converted {
        for (param_url, insertable) in entry.write.iter().flat_map(|w| &w.dynamic) {
            dynamic.collect(&entry.key, param_url, insertable);
        }
    }

    // Grouped by resource type: each per-resource-type table has its own
    // column list, so it needs its own statement.
    let mut system_rows: HashMap<&str, Vec<SystemRow<'_>>> = HashMap::new();
    for entry in &converted {
        if let Some(slots) = entry.write.as_ref().and_then(|w| w.system_row.as_ref()) {
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

    let mut tx = pool.begin().await.map_err(|e| {
        OperationOutcomeError::fatal(
            IssueType::exception(),
            format!("Failed to begin PG search transaction: {e}"),
        )
    })?;

    // Clears the previous version of every resource in the batch — the
    // re-indexed ones and the deleted ones alike — before re-inserting.
    delete_previous_versions(tx.as_mut(), schema_registry, &converted).await?;
    insert_anchors(tx.as_mut(), schema_registry, &converted).await?;

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

/// The batch's resource keys, as one array per column.
#[derive(Default)]
struct AnchorColumns {
    tenant: Vec<String>,
    project: Vec<String>,
    resource_type: Vec<String>,
    resource_id: Vec<String>,
}

impl AnchorColumns {
    fn collect<'a>(keys: impl Iterator<Item = &'a ResourceKey>) -> Self {
        let mut columns = AnchorColumns::default();
        for key in keys {
            columns.tenant.push(key.tenant.clone());
            columns.project.push(key.project.clone());
            columns.resource_type.push(key.resource_type.clone());
            columns.resource_id.push(key.resource_id.clone());
        }
        columns
    }

    fn bind<'a>(&'a self, query: PgQuery<'a>) -> PgQuery<'a> {
        query
            .bind(&self.tenant)
            .bind(&self.project)
            .bind(&self.resource_type)
            .bind(&self.resource_id)
    }
}

/// Clears every trace of the previous version of each resource in the batch:
/// the shared-table rows, the per-resource-type row, then the anchor.
///
/// There are no foreign keys to cascade through: one would fire a
/// referential-integrity trigger for all ~145 resource type tables on every
/// deleted row. Instead each table the batch touches gets its own statement.
async fn delete_previous_versions(
    conn: &mut sqlx::PgConnection,
    schema_registry: &SchemaRegistry,
    converted: &[Converted],
) -> Result<(), OperationOutcomeError> {
    let anchors = AnchorColumns::collect(converted.iter().map(|entry| &entry.key));

    for table in SharedTable::ALL.map(|table| schema_registry.shared_table_name(table)) {
        let sql = format!(
            "DELETE FROM {table} t \
             USING unnest($1::text[], $2::text[], $3::text[], $4::text[]) \
                 AS k(tenant, project, resource_type, resource_id) \
             WHERE t.tenant = k.tenant AND t.project = k.project \
                 AND t.resource_type = k.resource_type AND t.resource_id = k.resource_id"
        );
        anchors
            .bind(sqlx::query(&sql))
            .execute(&mut *conn)
            .await
            .map_err(PgSearchError::from)?;
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
        .map_err(PgSearchError::from)?;
    }

    let sql = format!(
        "DELETE FROM {resource_table} sr \
         USING unnest($1::text[], $2::text[], $3::text[], $4::text[]) \
             AS k(tenant, project, resource_type, resource_id) \
         WHERE sr.tenant = k.tenant AND sr.project = k.project \
             AND sr.resource_type = k.resource_type AND sr.resource_id = k.resource_id",
        resource_table = schema_registry.resource_table_name(),
    );

    anchors
        .bind(sqlx::query(&sql))
        .execute(conn)
        .await
        .map_err(PgSearchError::from)?;

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

/// Inserts the anchor rows, which every other table's rows hang off.
///
/// The anchor also carries the resource-level parameters' columns, so this
/// binds them the same way the resource type insert binds its own.
async fn insert_anchors(
    conn: &mut sqlx::PgConnection,
    schema_registry: &SchemaRegistry,
    converted: &[Converted],
) -> Result<(), OperationOutcomeError> {
    let anchor = schema_registry.anchor();

    let rows: Vec<(&Converted, &ConvertedWrite)> = converted
        .iter()
        .filter_map(|entry| entry.write.as_ref().map(|write| (entry, write)))
        .collect();

    if rows.is_empty() {
        return Ok(());
    }

    let per_row = ANCHOR_FIXED_COLUMNS + anchor.columns.len();

    let mut column_list = String::from("tenant, project, resource_type, resource_id, version_id");
    for column in &anchor.columns {
        let _ = write!(column_list, ", \"{}\"", column.name);
    }

    for chunk in rows.chunks((MAX_BIND_PARAMS / per_row).max(1)) {
        let sql = format!(
            "INSERT INTO {} ({column_list}) VALUES {}",
            anchor.table_name,
            values_clause(chunk.len(), per_row),
        );

        let mut query = sqlx::query(&sql);

        for (entry, write) in chunk {
            query = query
                .bind(&entry.key.tenant)
                .bind(&entry.key.project)
                .bind(&entry.key.resource_type)
                .bind(&entry.key.resource_id)
                .bind(&entry.version_id);

            query = bind_columns(query, &write.anchor_row, &anchor.columns);
        }

        query.execute(&mut *conn).await.map_err(|e| {
            OperationOutcomeError::fatal(
                IssueType::exception(),
                format!(
                    "Failed to insert anchor rows into {}: {e}",
                    anchor.table_name
                ),
            )
        })?;
    }

    Ok(())
}

/// `tenant, project, resource_type, resource_id, version_id` — the columns the
/// anchor carries ahead of its generated ones.
const ANCHOR_FIXED_COLUMNS: usize = 5;

/// Binds one row's generated column values, in schema order.
///
/// An absent value still has to carry the column's type, or Postgres cannot
/// infer the parameter.
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
        let sql = system_insert_sql(schema, &column_list, chunk.len(), per_row);
        let mut query = sqlx::query(&sql);

        for row in chunk {
            query = query
                .bind(&row.key.tenant)
                .bind(&row.key.project)
                .bind(&row.key.resource_id)
                .bind(row.version_id);

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

#[cfg(test)]
mod tests {
    use super::*;
    use haste_repository::types::SupportedFHIRVersions;

    const IDENTITY_COLUMNS: [&str; 5] = [
        "tenant",
        "project",
        "resource_type",
        "resource_id",
        "param_url",
    ];

    /// The columns of `INSERT INTO <table> (...)`.
    fn inserted_columns(sql: &str) -> Vec<String> {
        let open = sql.find('(').expect("insert names its columns");
        let close = sql[open..].find(')').expect("unterminated column list") + open;
        sql[open + 1..close]
            .split(',')
            .map(|column| column.trim().to_string())
            .collect()
    }

    /// The columns of the `CREATE TABLE`, which is the first statement in the
    /// DDL (the `CREATE INDEX`es follow it).
    fn declared_columns(ddl: &str) -> Vec<String> {
        let open = ddl.find('(').expect("create table names its columns");
        let close = ddl.find("\n);").expect("unterminated create table");
        ddl[open + 1..close]
            .split(',')
            .filter_map(|line| line.split_whitespace().next().map(str::to_string))
            .collect()
    }

    /// The insert and the `CREATE TABLE` must name exactly the same columns in
    /// the same order. Hardcoding a list on either side — which is how
    /// `target_uri` once came to be written to a table that had no such
    /// column — fails here rather than on the first resource indexed.
    #[test]
    fn every_shared_insert_matches_its_table_definition() {
        for table in SharedTable::ALL {
            let name = table.table_name(SupportedFHIRVersions::R4);
            let expected: Vec<String> = IDENTITY_COLUMNS
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

            // Each value column is bound as an array of its declared type;
            // the identity columns take $1..$5.
            let sql = dynamic_insert_sql(&name, table);
            for (offset, column) in table.value_columns().iter().enumerate() {
                assert!(
                    sql.contains(&format!("${}::{}[]", offset + 6, column.sql_type)),
                    "{name}: {} is bound at the wrong position or type",
                    column.name
                );
            }
        }
    }
}
