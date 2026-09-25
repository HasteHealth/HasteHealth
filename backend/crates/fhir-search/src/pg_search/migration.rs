use haste_fhir_model::r4::generated::terminology::IssueType;
use haste_fhir_operation_error::OperationOutcomeError;
use sqlx::{Pool, Postgres};

use super::schema::{
    ColumnDef, IndexKind, ResourceTypeSchema, SHARED_TABLES, SchemaRegistry, SharedTable,
    resource_table_name, shared_table_name, shared_value_columns, sql_type,
};

/// Creates the anchor, the shared tables and one table per resource type.
/// Idempotent: new parameters are added with `ADD COLUMN IF NOT EXISTS`, and
/// existing rows read NULL until reindexed.
///
/// # Errors
///
/// Returns an error if the migration lock cannot be taken or any DDL fails.
pub async fn run_migration(
    pool: &Pool<Postgres>,
    registry: &SchemaRegistry,
) -> Result<(), OperationOutcomeError> {
    // Concurrent `CREATE TABLE IF NOT EXISTS` can fail on
    // `pg_type_typname_nsp_index` when two servers start together, so the
    // whole migration runs under an advisory lock.
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

    // DDL runs on the pool; other servers are still blocked on the lock above.
    let result = run_migration_locked(pool, registry).await;

    // Best-effort: the lock is session-scoped and dies with the connection.
    let _ = sqlx::query("SELECT pg_advisory_unlock($1)")
        .bind(MIGRATION_LOCK_KEY)
        .execute(&mut *lock)
        .await;

    result
}

/// Arbitrary but stable. No other advisory lock in this database may use it.
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

    // A fixed order keeps the per-table ACCESS EXCLUSIVE locks deadlock-free
    // even without the advisory lock.
    let mut schemas: Vec<_> = registry.schemas.values().collect();
    schemas.sort_by(|a, b| a.table_name.cmp(&b.table_name));

    for schema in schemas {
        execute_ddl(
            pool,
            &resource_type_table_sql(schema),
            &format!("Failed to migrate PG search table '{}'", schema.table_name),
        )
        .await?;
    }

    tracing::info!(
        "PG search index tables created/verified successfully ({} resource type tables).",
        registry.schemas.len()
    );
    Ok(())
}

async fn execute_ddl(
    pool: &Pool<Postgres>,
    sql: &str,
    context: &str,
) -> Result<(), OperationOutcomeError> {
    sqlx::raw_sql(sql)
        .execute(pool)
        .await
        .map(|_| ())
        .map_err(|e| wrap(context, &e))
}

/// The anchor and the shared tables: everything independent of which
/// parameters exist.
fn base_migration_sql(registry: &SchemaRegistry) -> String {
    let resource_table = resource_table_name(&registry.version);

    let create_anchor = format!(
        "-- One row per indexed resource. Other tables reference it by `res_key`,
-- which is never reused.
--
-- `sequence` is the repository write sequence of the indexed version; the
-- upsert uses it to reject replays of older versions.
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

-- `:contains` is an unanchored LIKE: trigrams serve it, and `btree_gin` lets
-- the scope columns lead that index.
CREATE EXTENSION IF NOT EXISTS pg_trgm;
CREATE EXTENSION IF NOT EXISTS btree_gin;
"
    );

    // Resource-level parameters are anchor columns, so untyped searches can
    // read them.
    let anchor_columns = add_columns_sql(&resource_table, &registry.anchor.columns);
    let anchor_indexes = indexes_sql(&resource_table, &registry.anchor.columns, ANCHOR_SCOPE);
    let shared_tables: String = SHARED_TABLES
        .into_iter()
        .map(|table| shared_table_sql(registry, table))
        .collect();

    create_anchor + &anchor_columns + &anchor_indexes + &shared_tables
}

/// One shared table. Rows are keyed by `(res_key, param_identity)`, sixteen
/// bytes instead of the text they hash, and value columns are scalar so the
/// same B-trees work as on the type tables.
fn shared_table_sql(registry: &SchemaRegistry, table: SharedTable) -> String {
    let name = shared_table_name(&registry.version, table);

    let value_columns = shared_value_columns(table)
        .iter()
        .map(|column| {
            let null = if column.nullable { "" } else { " NOT NULL" };
            format!("    {} {}{null}", column.name, column.sql_type)
        })
        .collect::<Vec<_>>()
        .join(",\n");

    let create_table = format!(
        "CREATE TABLE IF NOT EXISTS {name} (
    res_key        BIGINT NOT NULL,
    param_identity BIGINT NOT NULL,
{value_columns}
);

-- Correlates rows back to their resource, for clauses and for clearing.
CREATE INDEX IF NOT EXISTS idx_{name}_resource
    ON {name} (res_key, param_identity);
"
    );

    // Lookup index: the identity (already one project), then the compared
    // value. String searches compare `LOWER(value)`.
    let value_key = match table {
        SharedTable::String => "LOWER(value) text_pattern_ops",
        SharedTable::Uri | SharedTable::Number => "value",
        SharedTable::Token => "code",
        SharedTable::Date => "start_ms, end_ms",
        SharedTable::Quantity => "start_value, end_value",
        SharedTable::Reference => "target_id",
    };
    let value_index = format!(
        "CREATE INDEX IF NOT EXISTS idx_{name}_value
    ON {name} (param_identity, {value_key});
"
    );

    // All parameters share the table, so a value's overall frequency says
    // little about its frequency under one parameter. Without these stats the
    // planner multiplies selectivities, badly underestimates values common to
    // one parameter (e.g. a LOINC code under `combo-code`), and picks the wrong
    // driving index.
    let stats_key = match table {
        SharedTable::String => Some("(LOWER(value))"),
        SharedTable::Uri => Some("value"),
        SharedTable::Token => Some("code"),
        SharedTable::Reference => Some("target_id"),
        SharedTable::Number | SharedTable::Date | SharedTable::Quantity => None,
    };
    let statistics = stats_key.map_or_else(String::new, |stats_key| {
        format!(
            "CREATE STATISTICS IF NOT EXISTS {name}_identity_stats (mcv, ndistinct, dependencies)
    ON param_identity, {stats_key} FROM {name};
ALTER STATISTICS {name}_identity_stats SET STATISTICS 1000;
"
        )
    });

    let contains_index = if matches!(table, SharedTable::String) {
        format!(
            "CREATE INDEX IF NOT EXISTS idx_{name}_contains
    ON {name} USING GIN (param_identity, LOWER(value) gin_trgm_ops);
"
        )
    } else {
        String::new()
    };

    create_table + &value_index + &statistics + &contains_index
}

/// Creates or extends one resource type table. `scope` hashes tenant and
/// project so indexes stay within one project without storing their text.
fn resource_type_table_sql(schema: &ResourceTypeSchema) -> String {
    let table = &schema.table_name;

    format!(
        "CREATE TABLE IF NOT EXISTS {table} (
    res_key BIGINT PRIMARY KEY,
    scope   BIGINT NOT NULL
);
"
    ) + &add_columns_sql(table, &schema.columns)
        + &indexes_sql(table, &schema.columns, TYPE_TABLE_SCOPE)
}

fn add_columns_sql(table: &str, columns: &[ColumnDef]) -> String {
    columns
        .iter()
        .map(|column| {
            format!(
                "ALTER TABLE {table} ADD COLUMN IF NOT EXISTS {} {};\n",
                quote_ident(&column.name),
                sql_type(column.column_type)
            )
        })
        .collect()
}

fn indexes_sql(table: &str, columns: &[ColumnDef], scope: &[&str]) -> String {
    columns
        .iter()
        .map(|column| index_sql(table, column, scope))
        .collect()
}

/// Leads every type table index, so a query never scans other tenants' rows.
const TYPE_TABLE_SCOPE: &[&str] = &["scope"];

/// Leads every anchor index. Typed searches (nearly all) pin `resource_type`
/// too; untyped ones still use the first two.
const ANCHOR_SCOPE: &[&str] = &["tenant", "project", "resource_type"];

/// The index for one column, led by the equality-constrained `scope` columns.
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
        // B-tree for anchored `LIKE 'abc%'`, trigram GIN for `LIKE '%abc%'`.
        IndexKind::LoweredPrefix => format!(
            "CREATE INDEX IF NOT EXISTS {name} \
             ON {table} ({lead}, LOWER({ident}) text_pattern_ops);\n\
             CREATE INDEX IF NOT EXISTS {name}_ct \
             ON {table} USING GIN ({lead}, LOWER({ident}) gin_trgm_ops);\n"
        ),
    }
}

/// Postgres truncates identifiers past `NAMEDATALEN - 1` silently.
const MAX_IDENTIFIER_LEN: usize = 63;

fn truncate_identifier(name: &str) -> String {
    name.chars().take(MAX_IDENTIFIER_LEN).collect()
}

fn quote_ident(name: &str) -> String {
    format!("\"{}\"", name.replace('"', "\"\""))
}

fn wrap(context: &str, error: &sqlx::Error) -> OperationOutcomeError {
    OperationOutcomeError::fatal(IssueType::exception(), format!("{context}: {error}"))
}

/// A shared table's `CREATE TABLE`, so indexing tests can check their inserts
/// against it. Its shape doesn't depend on the registered parameters.
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

    /// An index not led by the scope scans every tenant's rows.
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
