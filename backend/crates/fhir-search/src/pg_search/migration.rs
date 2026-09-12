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
    sqlx::raw_sql(BASE_MIGRATION_SQL)
        .execute(pool)
        .await
        .map_err(|e| wrap("Failed to run PG search base migration", &e))?;

    for schema in registry.iter() {
        migrate_resource_type_table(pool, schema).await?;
    }

    tracing::info!(
        "PG search index tables created/verified successfully ({} resource type tables).",
        registry.len()
    );
    Ok(())
}

async fn migrate_resource_type_table(
    pool: &Pool<Postgres>,
    schema: &ResourceTypeSchema,
) -> Result<(), OperationOutcomeError> {
    let table = &schema.table_name;

    sqlx::raw_sql(&create_table_sql(schema))
        .execute(pool)
        .await
        .map_err(|e| wrap(&format!("Failed to create table '{table}'"), &e))?;

    // A table created by an earlier release may be missing columns for search
    // parameters added since; add them rather than recreating the table.
    let add_columns = add_columns_sql(schema);
    if !add_columns.is_empty() {
        sqlx::raw_sql(&add_columns)
            .execute(pool)
            .await
            .map_err(|e| wrap(&format!("Failed to add columns to '{table}'"), &e))?;
    }

    let indexes = create_indexes_sql(schema);
    if !indexes.is_empty() {
        sqlx::raw_sql(&indexes)
            .execute(pool)
            .await
            .map_err(|e| wrap(&format!("Failed to create indexes on '{table}'"), &e))?;
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

    sql.push_str(
        ",\n    PRIMARY KEY (tenant, project, resource_id)\
         ,\n    FOREIGN KEY (tenant, project, resource_type, resource_id)\n        \
         REFERENCES search_resource (tenant, project, resource_type, resource_id)\n        \
         ON DELETE CASCADE\n);",
    );

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

-- String values (name, address, etc.)
CREATE TABLE IF NOT EXISTS search_dynamic_string (
    tenant        TEXT NOT NULL,
    project       TEXT NOT NULL,
    resource_type TEXT NOT NULL,
    resource_id   TEXT NOT NULL,
    param_url     TEXT NOT NULL,
    value         TEXT NOT NULL,
    FOREIGN KEY (tenant, project, resource_type, resource_id)
        REFERENCES search_resource (tenant, project, resource_type, resource_id)
        ON DELETE CASCADE
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
    code          TEXT,
    FOREIGN KEY (tenant, project, resource_type, resource_id)
        REFERENCES search_resource (tenant, project, resource_type, resource_id)
        ON DELETE CASCADE
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
    end_ms        BIGINT NOT NULL,
    FOREIGN KEY (tenant, project, resource_type, resource_id)
        REFERENCES search_resource (tenant, project, resource_type, resource_id)
        ON DELETE CASCADE
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
    value         DOUBLE PRECISION NOT NULL,
    FOREIGN KEY (tenant, project, resource_type, resource_id)
        REFERENCES search_resource (tenant, project, resource_type, resource_id)
        ON DELETE CASCADE
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
    value         TEXT NOT NULL,
    FOREIGN KEY (tenant, project, resource_type, resource_id)
        REFERENCES search_resource (tenant, project, resource_type, resource_id)
        ON DELETE CASCADE
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
    target_uri           TEXT,
    FOREIGN KEY (tenant, project, resource_type, resource_id)
        REFERENCES search_resource (tenant, project, resource_type, resource_id)
        ON DELETE CASCADE
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
    end_code      TEXT,
    FOREIGN KEY (tenant, project, resource_type, resource_id)
        REFERENCES search_resource (tenant, project, resource_type, resource_id)
        ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_search_dynamic_quantity_value
    ON search_dynamic_quantity (tenant, project, resource_type, param_url, start_value, end_value);
"#;

/// Renders the DDL a schema would produce, for tests and manual inspection.
#[cfg(test)]
pub fn preview_ddl(schema: &ResourceTypeSchema) -> String {
    format!(
        "{}\n\n{}",
        create_table_sql(schema),
        create_indexes_sql(schema)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pg_search::schema::{ColumnType, ParamColumns};
    use std::collections::HashMap;

    fn test_schema() -> ResourceTypeSchema {
        let mut parameters = HashMap::new();
        parameters.insert(
            "name".to_string(),
            ParamColumns::String {
                value: "name".to_string(),
            },
        );
        parameters.insert(
            "birthdate".to_string(),
            ParamColumns::Date {
                start: "birthdate_start".to_string(),
                end: "birthdate_end".to_string(),
            },
        );

        ResourceTypeSchema {
            resource_type: "Patient".to_string(),
            table_name: "search_patient".to_string(),
            parameters,
            columns: vec![
                ColumnDef {
                    name: "birthdate_end".to_string(),
                    column_type: ColumnType::BigIntArray,
                    indexed: true,
                },
                ColumnDef {
                    name: "birthdate_start".to_string(),
                    column_type: ColumnType::BigIntArray,
                    indexed: true,
                },
                ColumnDef {
                    name: "name".to_string(),
                    column_type: ColumnType::TextArray,
                    indexed: true,
                },
            ],
        }
    }

    #[test]
    fn create_table_includes_every_column_and_key() {
        let sql = create_table_sql(&test_schema());

        assert!(sql.starts_with("CREATE TABLE IF NOT EXISTS search_patient ("));
        assert!(sql.contains("\"name\" TEXT[]"));
        assert!(sql.contains("\"birthdate_start\" BIGINT[]"));
        assert!(sql.contains("\"birthdate_end\" BIGINT[]"));
        assert!(sql.contains("PRIMARY KEY (tenant, project, resource_id)"));
        assert!(sql.contains("resource_type TEXT NOT NULL DEFAULT 'Patient'"));
        assert!(sql.contains("CHECK (resource_type = 'Patient')"));
        assert!(sql.contains("REFERENCES search_resource"));
    }

    #[test]
    fn add_columns_is_idempotent_per_column() {
        let sql = add_columns_sql(&test_schema());
        assert_eq!(sql.matches("ADD COLUMN IF NOT EXISTS").count(), 3);
        assert!(
            sql.contains("ALTER TABLE search_patient ADD COLUMN IF NOT EXISTS \"name\" TEXT[];")
        );
    }

    #[test]
    fn indexes_cover_indexed_columns_only() {
        let mut schema = test_schema();
        schema.columns.push(ColumnDef {
            name: "identifier_system".to_string(),
            column_type: ColumnType::TextArray,
            indexed: false,
        });

        let sql = create_indexes_sql(&schema);
        assert!(sql.contains("idx_search_patient_lookup ON search_patient (tenant, project)"));
        assert!(sql.contains("USING GIN (\"name\")"));
        assert!(!sql.contains("identifier_system"));
    }

    #[test]
    fn long_index_names_are_truncated_and_stay_distinct() {
        let long_a = format!("idx_search_x_{}", "a".repeat(80));
        let long_b = format!("idx_search_x_{}", "b".repeat(80));

        let a = truncate_identifier(&long_a);
        let b = truncate_identifier(&long_b);

        assert!(a.len() <= MAX_IDENTIFIER_LEN);
        assert!(b.len() <= MAX_IDENTIFIER_LEN);
        assert_ne!(a, b);
        assert_eq!(truncate_identifier("idx_short"), "idx_short");
    }
}

/// Runs the real migration against a live PostgreSQL instance.
///
/// Ignored by default since it needs a database; run with
/// `PG_SEARCH_TEST_URL=postgres://... cargo test -p haste-fhir-search
/// --lib migration::live -- --ignored --nocapture`.
#[cfg(test)]
mod live {
    use super::*;
    use crate::memory::R4_SEARCH_PARAMETERS_INDEX;
    use crate::pg_search::schema::generate_schemas;

    #[tokio::test]
    #[ignore = "requires a live PostgreSQL instance"]
    async fn migration_is_idempotent() {
        let Ok(url) = std::env::var("PG_SEARCH_TEST_URL") else {
            panic!("set PG_SEARCH_TEST_URL");
        };

        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(4)
            .connect(&url)
            .await
            .expect("connect");

        let registry = generate_schemas(&R4_SEARCH_PARAMETERS_INDEX.all_parameters());
        println!("generated {} resource type tables", registry.len());

        run_migration(&pool, &registry).await.expect("first run");
        // Re-running must be a no-op, not an error.
        run_migration(&pool, &registry).await.expect("second run");

        let tables: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM information_schema.tables \
             WHERE table_schema = 'public' AND table_name LIKE 'search\\_%'",
        )
        .fetch_one(&pool)
        .await
        .expect("count tables");

        println!("search_* tables in database: {tables}");
        assert!(tables as usize >= registry.len());
    }
}
