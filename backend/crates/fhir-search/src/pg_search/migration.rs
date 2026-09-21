use std::fmt::Write as _;

use haste_fhir_model::r4::generated::terminology::IssueType;
use haste_fhir_operation_error::OperationOutcomeError;
use sqlx::{Pool, Postgres};

use super::schema::{ColumnDef, IndexKind, ResourceTypeSchema, SchemaRegistry, SharedTable};

/// Creates the shared tables, then a per-resource-type table for every schema
/// in `registry`. Idempotent — safe to re-run on an existing database.
///
/// New search parameters added by a later release show up as new columns on
/// the existing tables via `ADD COLUMN IF NOT EXISTS`, so an upgrade never
/// requires a reindex to *add* a parameter (existing rows keep NULL until the
/// resource is next indexed).
///
/// # Errors
///
/// Returns an error if the migration lock cannot be taken, or if any of the
/// DDL fails.
pub async fn run_migration(
    pool: &Pool<Postgres>,
    registry: &SchemaRegistry,
) -> Result<(), OperationOutcomeError> {
    // `CREATE TABLE IF NOT EXISTS` is not atomic against a concurrent create:
    // two servers starting together both see the table missing, both create
    // it, and one fails on `pg_type_typname_nsp_index`. Across ~145 tables
    // that is close to certain. An advisory lock serializes the whole
    // migration instead, so the second server waits and then finds everything
    // already in place.
    let mut lock = pool.acquire().await.map_err(|e| {
        wrap(
            "Failed to acquire a connection for the PG search migration",
            &e,
        )
    })?;

    sqlx::query("SELECT pg_advisory_lock($1)")
        .bind(MIGRATION_LOCK_KEY)
        .execute(&mut *lock)
        .await
        .map_err(|e| wrap("Failed to take the PG search migration lock", &e))?;

    // The DDL itself runs on the pool rather than on `lock`. The mutual
    // exclusion still holds: any other server blocks on `pg_advisory_lock`
    // above until this one releases it below, whichever connections the work
    // in between happens to use.
    let result = run_migration_locked(pool, registry).await;

    // Releasing is best-effort: the lock is session-scoped, so dropping the
    // connection frees it anyway.
    let _ = sqlx::query("SELECT pg_advisory_unlock($1)")
        .bind(MIGRATION_LOCK_KEY)
        .execute(&mut *lock)
        .await;

    result
}

/// An arbitrary but stable key — any other advisory lock in this database has
/// to avoid it.
const MIGRATION_LOCK_KEY: i64 = 0x0F47_5EA4_C401;

async fn run_migration_locked(
    pool: &Pool<Postgres>,
    registry: &SchemaRegistry,
) -> Result<(), OperationOutcomeError> {
    execute_ddl(
        pool,
        &base_migration_sql(registry),
        "Failed to run PG search base migration",
    )
    .await?;

    // Sorted, not in registry order: `SchemaRegistry` is backed by a HashMap,
    // so every process iterates it differently, and each table's DDL takes an
    // ACCESS EXCLUSIVE lock. The advisory lock above already serializes this,
    // but a single order costs nothing and keeps the DDL from deadlocking if
    // it is ever run outside that lock.
    let mut schemas: Vec<_> = registry.iter().collect();
    schemas.sort_by(|a, b| a.table_name.cmp(&b.table_name));

    for schema in schemas {
        migrate_resource_type_table(pool, schema).await?;
    }

    tracing::info!(
        "PG search index tables created/verified successfully ({} resource type tables).",
        registry.len()
    );
    Ok(())
}

/// Runs one DDL script, tagging any failure with what it was doing.
async fn execute_ddl(
    pool: &Pool<Postgres>,
    sql: &str,
    context: &str,
) -> Result<(), OperationOutcomeError> {
    sqlx::raw_sql(sql)
        .execute(pool)
        .await
        .map_err(|e| wrap(context, &e))?;
    Ok(())
}

/// The tables that do not depend on which search parameters exist: the anchor,
/// and one shared table per value type for the repeating parameters.
fn base_migration_sql(registry: &SchemaRegistry) -> String {
    let resource_table = registry.resource_table_name();
    let mut sql = String::new();

    let _ = write!(
        sql,
        "CREATE TABLE IF NOT EXISTS {resource_table} (
    tenant        TEXT NOT NULL,
    project       TEXT NOT NULL,
    resource_type TEXT NOT NULL,
    resource_id   TEXT NOT NULL,
    version_id    TEXT NOT NULL,
    PRIMARY KEY (tenant, project, resource_type, resource_id)
);

-- `:contains` is an unanchored LIKE, which no B-tree can answer. Trigrams
-- can; `btree_gin` lets the tenant and project lead that same index.
CREATE EXTENSION IF NOT EXISTS pg_trgm;
CREATE EXTENSION IF NOT EXISTS btree_gin;
"
    );

    // Resource-level parameters live here rather than on every resource type
    // table, so a search that names no type can still read them.
    let anchor = registry.anchor();
    for column in &anchor.columns {
        let _ = writeln!(
            sql,
            "ALTER TABLE {resource_table} ADD COLUMN IF NOT EXISTS {} {};",
            quote_ident(&column.name),
            column.column_type.sql_type()
        );
    }
    for column in &anchor.columns {
        sql.push_str(&index_sql(&resource_table, column, ANCHOR_SCOPE));
    }

    for table in SharedTable::ALL {
        sql.push_str(&shared_table_sql(registry, table));
    }

    sql
}

/// One shared table per value type, holding the repeating parameters.
///
/// Every row carries the parameter's canonical URL, because one table holds
/// values for many parameters. The value columns are scalars, so the same
/// B-tree that serves a column on a resource type table serves these.
fn shared_table_sql(registry: &SchemaRegistry, table: SharedTable) -> String {
    let name = registry.shared_table_name(table);

    // Declared from the same list the batch inserts bind through, so the two
    // cannot drift apart.
    let value_columns = table
        .value_columns()
        .iter()
        .map(|column| {
            let null = if column.nullable { "" } else { " NOT NULL" };
            format!("    {} {}{null}", column.name, column.sql_type)
        })
        .collect::<Vec<_>>()
        .join(",\n");

    let mut sql = format!(
        "CREATE TABLE IF NOT EXISTS {name} (
    tenant        TEXT NOT NULL,
    project       TEXT NOT NULL,
    resource_type TEXT NOT NULL,
    resource_id   TEXT NOT NULL,
    param_url     TEXT NOT NULL,
{value_columns}
);

-- Correlating a row back to its resource is what every clause does after it
-- has matched, so the key leads with the identity columns.
CREATE INDEX IF NOT EXISTS idx_{name}_resource
    ON {name} (tenant, project, resource_type, resource_id, param_url);
"
    );

    // The lookup index: identity down to the parameter, then the value the
    // clause compares.
    let value_key = match table {
        // Lowered, because a FHIR string search is case-insensitive and so
        // compares `LOWER(value)`. An index on the bare column cannot serve a
        // predicate over an expression of it.
        SharedTable::String => Some("LOWER(value) text_pattern_ops".to_string()),
        SharedTable::Uri => Some("value".to_string()),
        SharedTable::Token => Some("code".to_string()),
        SharedTable::Date => Some("start_ms, end_ms".to_string()),
        SharedTable::Number => Some("value".to_string()),
        SharedTable::Quantity => Some("start_value, end_value".to_string()),
        SharedTable::Reference => Some("target_id".to_string()),
    };

    if let Some(value_key) = value_key {
        let _ = write!(
            sql,
            "CREATE INDEX IF NOT EXISTS idx_{name}_value
    ON {name} (tenant, project, resource_type, param_url, {value_key});
"
        );
    }

    if matches!(table, SharedTable::String) {
        let _ = write!(
            sql,
            "CREATE INDEX IF NOT EXISTS idx_{name}_contains
    ON {name} USING GIN (tenant, project, resource_type, param_url, LOWER(value) gin_trgm_ops);
"
        );
    }

    sql
}

/// Creates or extends one resource type's table of singular parameters.
///
/// A release that classifies a new parameter as singular adds a column here;
/// existing rows keep NULL for it until the resource is next indexed, which is
/// the same as the parameter simply having no value.
async fn migrate_resource_type_table(
    pool: &Pool<Postgres>,
    schema: &ResourceTypeSchema,
) -> Result<(), OperationOutcomeError> {
    let table = &schema.table_name;

    let mut sql = format!(
        "CREATE TABLE IF NOT EXISTS {table} (
    tenant      TEXT NOT NULL,
    project     TEXT NOT NULL,
    resource_id TEXT NOT NULL,
    version_id  TEXT NOT NULL,
    PRIMARY KEY (tenant, project, resource_id)
);
"
    );

    for column in &schema.columns {
        let _ = writeln!(
            sql,
            "ALTER TABLE {table} ADD COLUMN IF NOT EXISTS {} {};",
            quote_ident(&column.name),
            column.column_type.sql_type()
        );
    }

    for column in &schema.columns {
        // `resource_type` is a constant on this table, so the tenant and
        // project are the whole scope.
        sql.push_str(&index_sql(table, column, TENANT_SCOPE));
    }

    execute_ddl(
        pool,
        &sql,
        &format!("Failed to migrate PG search table '{table}'"),
    )
    .await?;

    Ok(())
}

/// Every query is scoped to one tenant and project, so those lead every
/// index. Without them a predicate like `kind_code = 'resource'` scans the
/// matching rows of *every* tenant and discards all but one's.
const TENANT_SCOPE: &[&str] = &["tenant", "project"];

/// The anchor holds all resource types in one table, and a type-scoped search
/// — which is nearly all of them — constrains `resource_type` as a constant
/// beside the tenant and project. A search that names no type still gets the
/// tenant and project as the index's leading columns.
const ANCHOR_SCOPE: &[&str] = &["tenant", "project", "resource_type"];

/// The index for one column, or nothing when the column is only ever read
/// beside an indexed sibling on the same row.
///
/// `scope` is the equality-constrained key the index leads with.
fn index_sql(table: &str, column: &ColumnDef, scope: &[&str]) -> String {
    let name = truncate_identifier(&format!("idx_{table}_{}", column.name));
    let ident = quote_ident(&column.name);
    let lead = scope.join(", ");

    match column.index {
        IndexKind::None => String::new(),
        IndexKind::BTree => {
            format!("CREATE INDEX IF NOT EXISTS {name} ON {table} ({lead}, {ident});\n")
        }
        // The B-tree answers `LIKE 'abc%'` (anchored) and the trigram index
        // `LIKE '%abc%'` (unanchored). `btree_gin` is what lets the tenant and
        // project sit in front of the trigram column.
        IndexKind::LoweredPrefix => format!(
            "CREATE INDEX IF NOT EXISTS {name} \
             ON {table} ({lead}, LOWER({ident}) text_pattern_ops);\n\
             CREATE INDEX IF NOT EXISTS {name}_ct \
             ON {table} USING GIN ({lead}, LOWER({ident}) gin_trgm_ops);\n"
        ),
    }
}

/// PostgreSQL's identifier limit (`NAMEDATALEN - 1`). Past it names are
/// silently truncated, which would make two long parameter names collide.
const MAX_IDENTIFIER_LEN: usize = 63;

fn truncate_identifier(name: &str) -> String {
    if name.len() <= MAX_IDENTIFIER_LEN {
        return name.to_string();
    }

    name.chars().take(MAX_IDENTIFIER_LEN).collect()
}

fn quote_ident(name: &str) -> String {
    format!("\"{}\"", name.replace('"', "\"\""))
}

fn wrap(context: &str, error: &sqlx::Error) -> OperationOutcomeError {
    OperationOutcomeError::fatal(IssueType::exception(), format!("{context}: {error}"))
}

/// The `CREATE TABLE` for one shared table, so the indexing module can assert
/// its inserts name the same columns. An empty parameter set is enough: the
/// shared tables' shape does not depend on the registered parameters.
#[cfg(test)]
pub(crate) fn shared_table_sql_for_test(
    version: haste_repository::types::SupportedFHIRVersions,
    table: SharedTable,
) -> String {
    shared_table_sql(&super::schema::generate_schemas(version, &[]), table)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pg_search::schema::ColumnType;

    fn column(name: &str, index: IndexKind) -> ColumnDef {
        ColumnDef {
            name: name.to_string(),
            column_type: ColumnType::Text,
            index,
        }
    }

    /// Every search is scoped to one tenant and project, so an index that does
    /// not lead with them makes Postgres scan the matching rows of every
    /// tenant and throw all but one's away.
    #[test]
    fn every_index_leads_with_the_tenant_scope() {
        for (scope, lead) in [
            (TENANT_SCOPE, "(tenant, project, "),
            (ANCHOR_SCOPE, "(tenant, project, resource_type, "),
        ] {
            for index in [IndexKind::BTree, IndexKind::LoweredPrefix] {
                let sql = index_sql("t", &column("code", index), scope);
                for statement in sql.lines().filter(|line| line.contains("CREATE INDEX")) {
                    assert!(
                        statement.contains(lead),
                        "index does not lead with {lead}: {statement}"
                    );
                }
            }
        }
    }
}
