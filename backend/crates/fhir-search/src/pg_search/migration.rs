use haste_fhir_model::r4::generated::terminology::IssueType;
use haste_fhir_operation_error::OperationOutcomeError;
use sqlx::{Pool, Postgres};

use super::schema::{ColumnDef, ResourceTypeSchema, SchemaRegistry};

/// Creates the shared tables, then a per-resource-type table for every schema
/// in `registry`. Idempotent — safe to re-run on an existing database.
///
/// New search parameters added by a later release show up as new columns on
/// the existing tables via `ADD COLUMN IF NOT EXISTS`, so an upgrade never
/// requires a reindex to *add* a parameter (existing rows keep NULL until the
/// resource is next indexed).
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
        BASE_MIGRATION_SQL,
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

async fn migrate_resource_type_table(
    pool: &Pool<Postgres>,
    schema: &ResourceTypeSchema,
) -> Result<(), OperationOutcomeError> {
    let table = &schema.table_name;

    execute_ddl(
        pool,
        &create_table_sql(schema),
        &format!("Failed to create table '{table}'"),
    )
    .await?;

    // A table created by an earlier release may be missing columns for search
    // parameters added since; add them rather than recreating the table.
    let add_columns = add_columns_sql(schema);
    if !add_columns.is_empty() {
        execute_ddl(
            pool,
            &add_columns,
            &format!("Failed to add columns to '{table}'"),
        )
        .await?;
    }

    let indexes = create_indexes_sql(schema);
    if !indexes.is_empty() {
        execute_ddl(
            pool,
            &indexes,
            &format!("Failed to create indexes on '{table}'"),
        )
        .await?;
    }

    Ok(())
}

/// The `CREATE TABLE IF NOT EXISTS` statement for one resource type.
///
/// `resource_type` is a constant on this table — it exists only so the
/// foreign key can match `search_resource`'s composite primary key — so it
/// defaults to the table's resource type and is pinned there by a CHECK. The
/// table's own primary key is just `(tenant, project, resource_id)`.
fn create_table_sql(schema: &ResourceTypeSchema) -> String {
    let mut sql = format!(
        "CREATE TABLE IF NOT EXISTS {table} (\n    \
         tenant        TEXT NOT NULL,\n    \
         project       TEXT NOT NULL,\n    \
         resource_id   TEXT NOT NULL,\n    \
         version_id    TEXT NOT NULL,\n    \
         resource_type TEXT NOT NULL DEFAULT '{resource_type}'\n        \
         CONSTRAINT {constraint} CHECK (resource_type = '{resource_type}')",
        table = schema.table_name,
        resource_type = schema.resource_type,
        constraint = truncate_identifier(&format!("chk_{}_resource_type", schema.table_name)),
    );

    for column in &schema.columns {
        sql.push_str(&format!(
            ",\n    {} {}",
            quote_ident(&column.name),
            column.column_type.sql_type()
        ));
    }

    sql.push_str(",\n    PRIMARY KEY (tenant, project, resource_id)\n);");

    sql
}

/// `ALTER TABLE ... ADD COLUMN IF NOT EXISTS` for every value column, so an
/// existing table picks up parameters added after it was created.
fn add_columns_sql(schema: &ResourceTypeSchema) -> String {
    let mut sql = String::new();
    for column in &schema.columns {
        sql.push_str(&format!(
            "ALTER TABLE {} ADD COLUMN IF NOT EXISTS {} {};\n",
            schema.table_name,
            quote_ident(&column.name),
            column.column_type.sql_type()
        ));
    }
    sql
}

/// GIN indexes for the selective value columns, plus the tenant/project
/// lookup index used by every query's join.
fn create_indexes_sql(schema: &ResourceTypeSchema) -> String {
    let table = &schema.table_name;
    let mut sql =
        format!("CREATE INDEX IF NOT EXISTS idx_{table}_lookup ON {table} (tenant, project);\n");

    for column in &schema.columns {
        if !column.indexed {
            continue;
        }
        sql.push_str(&index_sql(table, column));
    }

    sql
}

fn index_sql(table: &str, column: &ColumnDef) -> String {
    // Index names are capped at 63 bytes by PostgreSQL and silently truncated
    // past that, which would make two long parameter names collide.
    let index_name = truncate_identifier(&format!("idx_{table}_{}", column.name));
    format!(
        "CREATE INDEX IF NOT EXISTS {index_name} ON {table} USING GIN ({});\n",
        quote_ident(&column.name)
    )
}

/// PostgreSQL's identifier limit (`NAMEDATALEN - 1`).
const MAX_IDENTIFIER_LEN: usize = 63;

fn truncate_identifier(name: &str) -> String {
    if name.len() <= MAX_IDENTIFIER_LEN {
        return name.to_string();
    }

    // Keep the prefix readable but append a hash of the full name so two
    // truncated-to-identical names stay distinct.
    let hash = name.bytes().fold(0u64, |acc, b| {
        acc.wrapping_mul(31).wrapping_add(u64::from(b))
    });
    let suffix = format!("_{hash:x}");
    let keep = MAX_IDENTIFIER_LEN - suffix.len();
    format!("{}{suffix}", &name[..keep])
}

/// Double-quotes an identifier so a generated column name is never parsed as a
/// keyword. Column names come from `code_to_column_base`, which already
/// restricts them to `[a-z0-9_]`, so escaping embedded quotes is unnecessary —
/// but the quoting keeps codes like `_source` unambiguous.
fn quote_ident(name: &str) -> String {
    format!("\"{name}\"")
}

fn wrap(context: &str, error: &sqlx::Error) -> OperationOutcomeError {
    OperationOutcomeError::fatal(IssueType::exception(), format!("{context}: {error}"))
}

/// Shared tables: the resource anchor plus the EAV tables backing
/// project-level (dynamic) search parameters.
///
/// The `search_dynamic_*` names replace the earlier `search_*` EAV tables,
/// which held *all* parameters before system-level ones moved to dedicated
/// per-resource-type columns. The rename is done first, so a database created
/// by the earlier schema carries its rows forward instead of silently starting
/// over with empty tables.
static BASE_MIGRATION_SQL: &str = r#"
-- Migrate pre-hybrid EAV tables to their new names. `search_resource` is
-- unchanged, so a renamed table keeps its foreign key intact.
DO $$
DECLARE
    legacy TEXT;
BEGIN
    FOREACH legacy IN ARRAY ARRAY['string', 'token', 'date', 'number', 'uri', 'reference', 'quantity']
    LOOP
        IF to_regclass('search_' || legacy) IS NOT NULL
           AND to_regclass('search_dynamic_' || legacy) IS NULL THEN
            EXECUTE format('ALTER TABLE %I RENAME TO %I', 'search_' || legacy, 'search_dynamic_' || legacy);
        END IF;
    END LOOP;
END $$;

-- Core resource identity table (one row per indexed resource)
CREATE TABLE IF NOT EXISTS search_resource (
    tenant        TEXT NOT NULL,
    project       TEXT NOT NULL,
    resource_type TEXT NOT NULL,
    resource_id   TEXT NOT NULL,
    version_id    TEXT NOT NULL,
    PRIMARY KEY (tenant, project, resource_type, resource_id)
);

CREATE INDEX IF NOT EXISTS idx_search_resource_lookup
    ON search_resource (tenant, project, resource_type);

-- Every search table used to carry an ON DELETE CASCADE foreign key to
-- `search_resource`. With one table per resource type that grew to ~145
-- constraints on a single parent, and Postgres fires a referential-integrity
-- trigger for *every one of them* on *every* anchor row deleted — ~145,000
-- trigger invocations to delete a 1000-resource batch, which measured at
-- 800ms before any real work happened. Indexing now deletes the child rows
-- explicitly and targets only the tables a batch actually touches, so the
-- constraints are dropped here. This is a derived index rebuildable from the
-- repository, and `indexing.rs` is the only writer.
DO $$
DECLARE
    constraint_row record;
BEGIN
    FOR constraint_row IN
        SELECT conrelid::regclass AS child_table, conname
        FROM pg_constraint
        WHERE confrelid = 'search_resource'::regclass AND contype = 'f'
        -- Dropping a constraint takes an ACCESS EXCLUSIVE lock on its table.
        -- Two servers migrating at once would take ~145 of those in whatever
        -- order the catalog scan returned, and deadlock; a deterministic order
        -- makes them queue instead.
        ORDER BY conrelid::regclass::text, conname
    LOOP
        -- The other session may have dropped it in between: both took their
        -- snapshot of the catalog before either started.
        EXECUTE format(
            'ALTER TABLE %s DROP CONSTRAINT IF EXISTS %I',
            constraint_row.child_table,
            constraint_row.conname
        );
    END LOOP;
END $$;

-- String values (name, address, etc.)
CREATE TABLE IF NOT EXISTS search_dynamic_string (
    tenant        TEXT NOT NULL,
    project       TEXT NOT NULL,
    resource_type TEXT NOT NULL,
    resource_id   TEXT NOT NULL,
    param_url     TEXT NOT NULL,
    value         TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_search_dynamic_string_prefix
    ON search_dynamic_string (tenant, project, resource_type, param_url, value text_pattern_ops);

-- Token values (code, system|code pairs)
CREATE TABLE IF NOT EXISTS search_dynamic_token (
    tenant        TEXT NOT NULL,
    project       TEXT NOT NULL,
    resource_type TEXT NOT NULL,
    resource_id   TEXT NOT NULL,
    param_url     TEXT NOT NULL,
    system        TEXT,
    code          TEXT
);

CREATE INDEX IF NOT EXISTS idx_search_dynamic_token_code
    ON search_dynamic_token (tenant, project, resource_type, param_url, code);
CREATE INDEX IF NOT EXISTS idx_search_dynamic_token_system_code
    ON search_dynamic_token (tenant, project, resource_type, param_url, system, code);

-- Date values (ranges stored as milliseconds-since-epoch)
CREATE TABLE IF NOT EXISTS search_dynamic_date (
    tenant        TEXT NOT NULL,
    project       TEXT NOT NULL,
    resource_type TEXT NOT NULL,
    resource_id   TEXT NOT NULL,
    param_url     TEXT NOT NULL,
    start_ms      BIGINT NOT NULL,
    end_ms        BIGINT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_search_dynamic_date_range
    ON search_dynamic_date (tenant, project, resource_type, param_url, start_ms, end_ms);

-- Number values
CREATE TABLE IF NOT EXISTS search_dynamic_number (
    tenant        TEXT NOT NULL,
    project       TEXT NOT NULL,
    resource_type TEXT NOT NULL,
    resource_id   TEXT NOT NULL,
    param_url     TEXT NOT NULL,
    value         DOUBLE PRECISION NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_search_dynamic_number_value
    ON search_dynamic_number (tenant, project, resource_type, param_url, value);

-- URI values
CREATE TABLE IF NOT EXISTS search_dynamic_uri (
    tenant        TEXT NOT NULL,
    project       TEXT NOT NULL,
    resource_type TEXT NOT NULL,
    resource_id   TEXT NOT NULL,
    param_url     TEXT NOT NULL,
    value         TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_search_dynamic_uri_value
    ON search_dynamic_uri (tenant, project, resource_type, param_url, value);

-- Reference values. Also backs reverse-reference lookups for system-level
-- parameters, which is why every reference is written here in addition to the
-- per-resource-type columns.
CREATE TABLE IF NOT EXISTS search_dynamic_reference (
    tenant               TEXT NOT NULL,
    project              TEXT NOT NULL,
    resource_type        TEXT NOT NULL,
    resource_id          TEXT NOT NULL,
    param_url            TEXT NOT NULL,
    target_resource_type TEXT,
    target_id            TEXT,
    target_uri           TEXT
);

CREATE INDEX IF NOT EXISTS idx_search_dynamic_reference_target
    ON search_dynamic_reference (tenant, project, resource_type, param_url, target_resource_type, target_id);
CREATE INDEX IF NOT EXISTS idx_search_dynamic_reference_reverse
    ON search_dynamic_reference (tenant, project, target_resource_type, target_id);

-- Quantity values (ranges with unit info)
CREATE TABLE IF NOT EXISTS search_dynamic_quantity (
    tenant        TEXT NOT NULL,
    project       TEXT NOT NULL,
    resource_type TEXT NOT NULL,
    resource_id   TEXT NOT NULL,
    param_url     TEXT NOT NULL,
    start_value   DOUBLE PRECISION NOT NULL,
    start_system  TEXT,
    start_code    TEXT,
    end_value     DOUBLE PRECISION NOT NULL,
    end_system    TEXT,
    end_code      TEXT
);

CREATE INDEX IF NOT EXISTS idx_search_dynamic_quantity_value
    ON search_dynamic_quantity (tenant, project, resource_type, param_url, start_value, end_value);

-- Re-indexing a resource clears its old rows from every `search_dynamic_*`
-- table by (tenant, project, resource_type, resource_id). The value indexes
-- above all carry `param_url` in position 4, so none of them can serve that
-- lookup — without these the delete degrades to a sequential scan of the whole
-- table, on every single create and update.

CREATE INDEX IF NOT EXISTS idx_search_dynamic_string_resource
    ON search_dynamic_string (tenant, project, resource_type, resource_id);
CREATE INDEX IF NOT EXISTS idx_search_dynamic_token_resource
    ON search_dynamic_token (tenant, project, resource_type, resource_id);
CREATE INDEX IF NOT EXISTS idx_search_dynamic_date_resource
    ON search_dynamic_date (tenant, project, resource_type, resource_id);
CREATE INDEX IF NOT EXISTS idx_search_dynamic_number_resource
    ON search_dynamic_number (tenant, project, resource_type, resource_id);
CREATE INDEX IF NOT EXISTS idx_search_dynamic_uri_resource
    ON search_dynamic_uri (tenant, project, resource_type, resource_id);
CREATE INDEX IF NOT EXISTS idx_search_dynamic_reference_resource
    ON search_dynamic_reference (tenant, project, resource_type, resource_id);
CREATE INDEX IF NOT EXISTS idx_search_dynamic_quantity_resource
    ON search_dynamic_quantity (tenant, project, resource_type, resource_id);
"#;
