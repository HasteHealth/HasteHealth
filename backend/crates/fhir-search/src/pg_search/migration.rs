use std::fmt::Write as _;

use haste_fhir_model::r4::generated::terminology::IssueType;
use haste_fhir_operation_error::OperationOutcomeError;
use sqlx::{Pool, Postgres};

use super::schema::{ColumnDef, IndexKind, ResourceTypeSchema, SchemaRegistry, SharedTable};

/// Creates the shared tables, then a table per schema in `registry`.
/// Idempotent.
///
/// A later release's new parameters arrive as `ADD COLUMN IF NOT EXISTS`, so
/// adding one never needs a reindex — existing rows keep NULL until the
/// resource is next indexed.
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
    // two servers starting together both see the table missing and one fails on
    // `pg_type_typname_nsp_index` — near certain across ~145 tables. The
    // advisory lock serializes the whole migration instead.
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

    // The DDL runs on the pool, not on `lock`; exclusion still holds, since any
    // other server blocks above until this one releases below.
    let result = run_migration_locked(pool, registry).await;

    // Best-effort: the lock is session-scoped, so dropping the connection frees
    // it anyway.
    let _ = sqlx::query("SELECT pg_advisory_unlock($1)")
        .bind(MIGRATION_LOCK_KEY)
        .execute(&mut *lock)
        .await;

    result
}

/// Arbitrary but stable; any other advisory lock in this database must avoid
/// it.
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

    // Sorted, not in registry order: the HashMap iterates differently per
    // process and each table's DDL takes an ACCESS EXCLUSIVE lock. The advisory
    // lock already serializes this, but one order costs nothing and keeps the
    // DDL deadlock-free if it is ever run outside that lock.
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

/// Runs one DDL script, tagging a failure with what it was doing.
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

/// The tables that do not depend on which parameters exist: the anchor, and one
/// shared table per value type.
fn base_migration_sql(registry: &SchemaRegistry) -> String {
    let resource_table = registry.resource_table_name();
    let mut sql = String::new();

    let _ = write!(
        sql,
        "-- One row per indexed resource, holding its identity. Every other table
-- refers to it by `res_key` alone, allocated here and never reused.
--
-- `sequence` is the repository's write sequence for the indexed version, so a
-- replayed older version cannot replace a newer one (see the upsert in
-- indexing).
CREATE TABLE IF NOT EXISTS {resource_table} (
    res_key       BIGINT GENERATED ALWAYS AS IDENTITY UNIQUE,
    tenant        TEXT NOT NULL,
    project       TEXT NOT NULL,
    resource_type TEXT NOT NULL,
    resource_id   TEXT NOT NULL,
    version_id    TEXT NOT NULL,
    sequence      BIGINT NOT NULL,
    PRIMARY KEY (tenant, project, resource_type, resource_id)
);

-- `:contains` is an unanchored LIKE, which no B-tree can answer; trigrams can,
-- and `btree_gin` lets the scoping key lead that same index.
CREATE EXTENSION IF NOT EXISTS pg_trgm;
CREATE EXTENSION IF NOT EXISTS btree_gin;
"
    );

    // Resource-level parameters live here, so a search naming no type can still
    // read them.
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
/// A row is keyed by `res_key` and the parameter's identity: sixteen bytes
/// instead of the text they hash. The value columns are scalars, so the same
/// B-trees that serve a resource type table's columns serve these.
fn shared_table_sql(registry: &SchemaRegistry, table: SharedTable) -> String {
    let name = registry.shared_table_name(table);

    // The same list the batch inserts bind through, so the two cannot drift.
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
    res_key        BIGINT NOT NULL,
    param_identity BIGINT NOT NULL,
{value_columns}
);

-- Every clause correlates a matched row back to its resource, and clearing a
-- resource's rows looks them up the same way.
CREATE INDEX IF NOT EXISTS idx_{name}_resource
    ON {name} (res_key, param_identity);
"
    );

    // The lookup index: the identity, which already confines it to one
    // project, then the value the clause compares.
    let value_key = match table {
        // A string search is case-insensitive, so it compares `LOWER(value)`,
        // which an index on the bare column cannot serve.
        SharedTable::String => Some("LOWER(value) text_pattern_ops".to_string()),
        SharedTable::Uri | SharedTable::Number => Some("value".to_string()),
        SharedTable::Token => Some("code".to_string()),
        SharedTable::Date => Some("start_ms, end_ms".to_string()),
        SharedTable::Quantity => Some("start_value, end_value".to_string()),
        SharedTable::Reference => Some("target_id".to_string()),
    };

    if let Some(value_key) = value_key {
        let _ = write!(
            sql,
            "CREATE INDEX IF NOT EXISTS idx_{name}_value
    ON {name} (param_identity, {value_key});
"
        );
    }

    // Every parameter shares these tables, so a value's frequency over the
    // whole table says little about its frequency for one parameter. Treating
    // the predicates as independent multiplies the selectivities and badly
    // underestimates a value common under one parameter (a LOINC code under
    // `combo-code`, say) — and that estimate decides whether the search drives
    // from the value index or the anchor's sort index. These statistics keep
    // the pair's frequencies together.
    let stats_key = match table {
        SharedTable::String => Some("(LOWER(value))"),
        SharedTable::Uri => Some("value"),
        SharedTable::Token => Some("code"),
        SharedTable::Reference => Some("target_id"),
        SharedTable::Number | SharedTable::Date | SharedTable::Quantity => None,
    };

    if let Some(stats_key) = stats_key {
        let _ = write!(
            sql,
            "CREATE STATISTICS IF NOT EXISTS {name}_identity_stats (mcv, ndistinct, dependencies)
    ON param_identity, {stats_key} FROM {name};
ALTER STATISTICS {name}_identity_stats SET STATISTICS 1000;
"
        );
    }

    if matches!(table, SharedTable::String) {
        let _ = write!(
            sql,
            "CREATE INDEX IF NOT EXISTS idx_{name}_contains
    ON {name} USING GIN (param_identity, LOWER(value) gin_trgm_ops);
"
        );
    }

    sql
}

/// Creates or extends one resource type's table of singular parameters. A newly
/// singular parameter adds a column; existing rows keep NULL until the resource
/// is next indexed, which reads the same as having no value.
async fn migrate_resource_type_table(
    pool: &Pool<Postgres>,
    schema: &ResourceTypeSchema,
) -> Result<(), OperationOutcomeError> {
    let table = &schema.table_name;

    // `scope` hashes the tenant and project, so an index led by it stays within
    // one project without every row carrying their text.
    let mut sql = format!(
        "CREATE TABLE IF NOT EXISTS {table} (
    res_key BIGINT PRIMARY KEY,
    scope   BIGINT NOT NULL
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
        // `resource_type` is constant here, so the scope is the whole of it.
        sql.push_str(&index_sql(table, column, TYPE_TABLE_SCOPE));
    }

    execute_ddl(
        pool,
        &sql,
        &format!("Failed to migrate PG search table '{table}'"),
    )
    .await?;

    Ok(())
}

/// Every query is scoped to one tenant and project, so their `scope` hash leads
/// every index here. Without it, `kind_code = 'resource'` scans every tenant's
/// matching rows and discards all but one's.
const TYPE_TABLE_SCOPE: &[&str] = &["scope"];

/// The anchor holds every resource type, and a type-scoped search — nearly all
/// of them — pins `resource_type` beside the tenant and project. A search
/// naming no type still leads with those two.
const ANCHOR_SCOPE: &[&str] = &["tenant", "project", "resource_type"];

/// The index for one column, or nothing when it is only read beside an indexed
/// sibling. `scope` is the equality-constrained key the index leads with.
fn index_sql(table: &str, column: &ColumnDef, scope: &[&str]) -> String {
    let name = truncate_identifier(&format!("idx_{table}_{}", column.name));
    let ident = quote_ident(&column.name);
    let lead = scope.join(", ");

    match column.index {
        IndexKind::None => String::new(),
        IndexKind::BTree => {
            format!("CREATE INDEX IF NOT EXISTS {name} ON {table} ({lead}, {ident});\n")
        }
        IndexKind::BTreeDescending => format!(
            "CREATE INDEX IF NOT EXISTS {name} ON {table} ({lead}, {ident} DESC NULLS LAST);\n"
        ),
        // The B-tree answers anchored `LIKE 'abc%'`, the trigram index
        // unanchored `LIKE '%abc%'`; `btree_gin` puts the scope in front of the
        // trigram column.
        IndexKind::LoweredPrefix => format!(
            "CREATE INDEX IF NOT EXISTS {name} \
             ON {table} ({lead}, LOWER({ident}) text_pattern_ops);\n\
             CREATE INDEX IF NOT EXISTS {name}_ct \
             ON {table} USING GIN ({lead}, LOWER({ident}) gin_trgm_ops);\n"
        ),
    }
}

/// PostgreSQL's identifier limit (`NAMEDATALEN - 1`). Past it names truncate
/// silently, which would collide two long parameter names.
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

/// The `CREATE TABLE` for one shared table, for the indexing module to assert
/// its inserts name the same columns. An empty parameter set is enough, since
/// their shape does not depend on the registered parameters.
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

    /// An index not led by the tenant scope makes Postgres scan every tenant's
    /// matching rows and throw all but one's away.
    #[test]
    fn every_index_leads_with_the_tenant_scope() {
        for (scope, lead) in [
            (TYPE_TABLE_SCOPE, "(scope, "),
            (ANCHOR_SCOPE, "(tenant, project, resource_type, "),
        ] {
            for index in [
                IndexKind::BTree,
                IndexKind::BTreeDescending,
                IndexKind::LoweredPrefix,
            ] {
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
