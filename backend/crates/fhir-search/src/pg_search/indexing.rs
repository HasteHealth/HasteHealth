use std::sync::Arc;

use haste_fhir_model::r4::generated::terminology::IssueType;
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhirpath::FPEngine;
use haste_repository::types::FHIRMethod;
use sqlx::{Pool, Postgres};

use super::{
    PgSearchError, resource_to_search_index,
    schema::{ParamColumns, ResourceTypeSchema, SchemaRegistry},
};
use crate::{
    IndexFailure, IndexOutcome, IndexResource, SearchParameterResolve,
    indexing_conversion::InsertableIndex,
};

pub async fn index_resources(
    pool: &Pool<Postgres>,
    parameter_resolver: &impl SearchParameterResolve,
    schema_registry: &SchemaRegistry,
    fp_engine: Arc<FPEngine>,
    resources: Vec<IndexResource>,
) -> Result<IndexOutcome, OperationOutcomeError> {
    let total = resources.len();
    tracing::trace!("PG search: indexing {} resources", total);

    let mut succeeded = 0usize;
    let mut failed = Vec::new();

    // Process in a single transaction for atomicity.
    let mut tx = pool.begin().await.map_err(|e| {
        OperationOutcomeError::fatal(
            IssueType::exception(),
            format!("Failed to begin PG search transaction: {e}"),
        )
    })?;

    for resource in resources {
        match index_single_resource(
            &mut tx,
            parameter_resolver,
            schema_registry,
            fp_engine.clone(),
            &resource,
        )
        .await
        {
            Ok(()) => succeeded += 1,
            Err(error) => failed.push(IndexFailure { resource, error }),
        }
    }

    tx.commit().await.map_err(|e| {
        OperationOutcomeError::fatal(
            IssueType::exception(),
            format!("Failed to commit PG search transaction: {e}"),
        )
    })?;

    if !failed.is_empty() {
        tracing::error!(
            "PG search: {} failed item(s) out of {}.",
            failed.len(),
            total
        );
    }

    Ok(IndexOutcome { succeeded, failed })
}

async fn index_single_resource(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    parameter_resolver: &impl SearchParameterResolve,
    schema_registry: &SchemaRegistry,
    fp_engine: Arc<FPEngine>,
    resource: &IndexResource,
) -> Result<(), OperationOutcomeError> {
    let tenant = resource.tenant.as_ref();
    let project = resource.project.as_ref();
    let resource_type = resource.resource_type.as_ref();
    let resource_id = resource.id.as_ref();

    let schema = schema_registry.get(resource_type);

    match &resource.fhir_method {
        FHIRMethod::Create | FHIRMethod::Update => {
            // Deleting the anchor row cascades to both the dynamic EAV tables
            // and the per-resource-type table, so this clears every trace of
            // the previous version before re-inserting.
            delete_resource_index(tx.as_mut(), tenant, project, resource_type, resource_id).await?;

            let params = parameter_resolver
                .by_resource_type(&resource.tenant, &resource.project, &resource.resource_type)
                .await?;

            let index = resource_to_search_index(fp_engine, &params, &resource.resource).await?;

            // Insert anchor row. The per-resource-type row's foreign key
            // points at this, so it has to land first.
            sqlx::query(
                "INSERT INTO search_resource (tenant, project, resource_type, resource_id, version_id) \
                 VALUES ($1, $2, $3, $4, $5)",
            )
            .bind(tenant)
            .bind(project)
            .bind(resource_type)
            .bind(resource_id)
            .bind(resource.version_id.as_ref())
            .execute(tx.as_mut())
            .await
            .map_err(|e| {
                OperationOutcomeError::fatal(
                    IssueType::exception(),
                    format!("Failed to insert search_resource: {e}"),
                )
            })?;

            // System-level parameters go into the per-resource-type table, one
            // row per resource with an array per column.
            if let Some(schema) = schema {
                insert_system_columns(
                    tx.as_mut(),
                    schema,
                    tenant,
                    project,
                    resource_id,
                    resource.version_id.as_ref(),
                    &index.system_entries,
                )
                .await?;
            }

            // Project-level parameters go into the shared EAV tables.
            for (param_url, insertable) in &index.dynamic_entries {
                insert_dynamic_values(
                    tx.as_mut(),
                    tenant,
                    project,
                    resource_type,
                    resource_id,
                    param_url,
                    insertable,
                )
                .await?;
            }

            // Reverse-reference lookups (`_revinclude`, chained search) scan
            // references across resource types, which the per-type columns
            // can't serve. Mirroring system-level references into the shared
            // reference table keeps one place to scan.
            insert_system_reference_mirror(
                tx.as_mut(),
                &params,
                tenant,
                project,
                resource_type,
                resource_id,
                &index.system_entries,
            )
            .await?;

            Ok(())
        }

        FHIRMethod::Delete => {
            delete_resource_index(tx.as_mut(), tenant, project, resource_type, resource_id).await
        }

        method @ FHIRMethod::Read => Err(OperationOutcomeError::from(
            PgSearchError::UnsupportedFHIRMethod((*method).clone()),
        )),
    }
}

async fn delete_resource_index(
    conn: &mut sqlx::PgConnection,
    tenant: &str,
    project: &str,
    resource_type: &str,
    resource_id: &str,
) -> Result<(), OperationOutcomeError> {
    sqlx::query(
        "DELETE FROM search_resource \
         WHERE tenant = $1 AND project = $2 AND resource_type = $3 AND resource_id = $4",
    )
    .bind(tenant)
    .bind(project)
    .bind(resource_type)
    .bind(resource_id)
    .execute(conn)
    .await
    .map_err(|e| {
        OperationOutcomeError::fatal(
            IssueType::exception(),
            format!("Failed to delete search_resource: {e}"),
        )
    })?;
    Ok(())
}

/// One column's worth of values, ready to bind as a PostgreSQL array.
///
/// `Text` carries `Option<String>` because a token's system (and a reference's
/// type) is genuinely absent for some values, and the NULL has to stay in
/// place so the parallel arrays keep lining up.
enum ColumnValues {
    Text(Vec<Option<String>>),
    BigInt(Vec<i64>),
    Double(Vec<f64>),
}

/// Writes the resource's system-level values as a single row on the
/// per-resource-type table.
///
/// Every value column is an array, and multi-part types (token, date,
/// reference, quantity) are written as *parallel* arrays: position `i` of each
/// column belongs to the same logical value, which is what lets queries
/// recombine them with `unnest(a, b)`.
async fn insert_system_columns(
    conn: &mut sqlx::PgConnection,
    schema: &ResourceTypeSchema,
    tenant: &str,
    project: &str,
    resource_id: &str,
    version_id: &str,
    entries: &[(String, InsertableIndex)],
) -> Result<(), OperationOutcomeError> {
    let mut columns: Vec<&str> = Vec::new();
    let mut values: Vec<ColumnValues> = Vec::new();

    for (code, insertable) in entries {
        // A parameter with no generated column (an unmapped type, or one that
        // lost a column-name collision) simply has nowhere to go here.
        let Some(param_columns) = schema.columns_for(code) else {
            continue;
        };

        collect_column_values(param_columns, insertable, &mut columns, &mut values);
    }

    // Keep the row even with no indexed values: the join in every
    // resource-type-scoped query is an inner join, so a missing row would hide
    // the resource from searches that filter on nothing but resource type.
    let mut column_list = String::from("tenant, project, resource_id, version_id");
    let mut placeholders = String::from("$1, $2, $3, $4");

    for (offset, column) in columns.iter().enumerate() {
        column_list.push_str(&format!(", \"{column}\""));
        placeholders.push_str(&format!(", ${}", offset + 5));
    }

    let sql = format!(
        "INSERT INTO {} ({column_list}) VALUES ({placeholders})",
        schema.table_name
    );

    let mut query = sqlx::query(&sql)
        .bind(tenant)
        .bind(project)
        .bind(resource_id)
        .bind(version_id);

    for value in &values {
        query = match value {
            ColumnValues::Text(v) => query.bind(v),
            ColumnValues::BigInt(v) => query.bind(v),
            ColumnValues::Double(v) => query.bind(v),
        };
    }

    query.execute(conn).await.map_err(|e| {
        OperationOutcomeError::fatal(
            IssueType::exception(),
            format!(
                "Failed to insert system search columns into {}: {e}",
                schema.table_name
            ),
        )
    })?;

    Ok(())
}

/// Appends the column names and parallel array values for one parameter.
///
/// A mismatch between the parameter's declared type and the evaluated value's
/// type (which would mean the schema and the conversion disagree) writes
/// nothing rather than a half-filled set of parallel arrays.
fn collect_column_values<'a>(
    param_columns: &'a ParamColumns,
    insertable: &InsertableIndex,
    columns: &mut Vec<&'a str>,
    values: &mut Vec<ColumnValues>,
) {
    match (param_columns, insertable) {
        (ParamColumns::String { value: column }, InsertableIndex::String(strings)) => {
            columns.push(column);
            values.push(ColumnValues::Text(
                strings.iter().map(|s| Some(s.clone())).collect(),
            ));
        }

        (ParamColumns::Uri { value: column }, InsertableIndex::URI(uris)) => {
            columns.push(column);
            values.push(ColumnValues::Text(
                uris.iter().map(|u| Some(u.clone())).collect(),
            ));
        }

        (ParamColumns::Number { value: column }, InsertableIndex::Number(numbers)) => {
            columns.push(column);
            values.push(ColumnValues::Double(numbers.clone()));
        }

        (ParamColumns::Token { system, code }, InsertableIndex::Token(tokens)) => {
            columns.push(system);
            values.push(ColumnValues::Text(
                tokens
                    .iter()
                    .map(|t| t.system().map(str::to_string))
                    .collect(),
            ));
            columns.push(code);
            values.push(ColumnValues::Text(
                tokens
                    .iter()
                    .map(|t| t.code().map(str::to_string))
                    .collect(),
            ));
        }

        (ParamColumns::Date { start, end }, InsertableIndex::Date(ranges)) => {
            columns.push(start);
            values.push(ColumnValues::BigInt(
                ranges.iter().map(|d| d.start).collect(),
            ));
            columns.push(end);
            values.push(ColumnValues::BigInt(ranges.iter().map(|d| d.end).collect()));
        }

        (
            ParamColumns::Reference {
                target_type,
                target_id,
            },
            InsertableIndex::Reference(references),
        ) => {
            columns.push(target_type);
            values.push(ColumnValues::Text(
                references
                    .iter()
                    .map(|r| r.resource_type().map(str::to_string))
                    .collect(),
            ));
            columns.push(target_id);
            values.push(ColumnValues::Text(
                references
                    .iter()
                    .map(|r| r.id().map(str::to_string))
                    .collect(),
            ));
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
            columns.push(start);
            values.push(ColumnValues::Double(
                quantities
                    .iter()
                    .map(super::super::indexing_conversion::QuantityRange::start_value)
                    .collect(),
            ));
            columns.push(end);
            values.push(ColumnValues::Double(
                quantities
                    .iter()
                    .map(super::super::indexing_conversion::QuantityRange::end_value)
                    .collect(),
            ));
            columns.push(system);
            values.push(ColumnValues::Text(
                quantities
                    .iter()
                    .map(|q| q.start_system().map(str::to_string))
                    .collect(),
            ));
            columns.push(code);
            values.push(ColumnValues::Text(
                quantities
                    .iter()
                    .map(|q| q.start_code().map(str::to_string))
                    .collect(),
            ));
        }

        _ => {}
    }
}

/// Mirrors system-level references into `search_dynamic_reference`.
///
/// The per-resource-type columns can only be read once the query knows which
/// table to join, so a reverse lookup ("which resources point at this one?")
/// has no table to start from. The shared reference table is that starting
/// point, which is why every reference is written to both places.
async fn insert_system_reference_mirror(
    conn: &mut sqlx::PgConnection,
    parameters: &[crate::ResolvedParameter],
    tenant: &str,
    project: &str,
    resource_type: &str,
    resource_id: &str,
    entries: &[(String, InsertableIndex)],
) -> Result<(), OperationOutcomeError> {
    for (code, insertable) in entries {
        let InsertableIndex::Reference(references) = insertable else {
            continue;
        };

        if references.is_empty() {
            continue;
        }

        // The mirror is keyed by URL like every other row in this table, so
        // the code has to be resolved back to the parameter it came from.
        let Some(param_url) = parameters
            .iter()
            .find(|p| p.search_parameter.code.value.as_deref() == Some(code.as_str()))
            .and_then(|p| p.search_parameter.url.value.as_deref())
        else {
            continue;
        };

        for reference in references {
            insert_reference_row(
                conn,
                tenant,
                project,
                resource_type,
                resource_id,
                param_url,
                reference,
            )
            .await?;
        }
    }

    Ok(())
}

/// Writes project-level values into the shared `search_dynamic_*` EAV tables.
async fn insert_dynamic_values(
    conn: &mut sqlx::PgConnection,
    tenant: &str,
    project: &str,
    resource_type: &str,
    resource_id: &str,
    param_url: &str,
    insertable: &InsertableIndex,
) -> Result<(), OperationOutcomeError> {
    match insertable {
        InsertableIndex::String(values) => {
            for value in values {
                sqlx::query(
                    "INSERT INTO search_dynamic_string (tenant, project, resource_type, resource_id, param_url, value) \
                     VALUES ($1, $2, $3, $4, $5, $6)",
                )
                .bind(tenant)
                .bind(project)
                .bind(resource_type)
                .bind(resource_id)
                .bind(param_url)
                .bind(value)
                .execute(&mut *conn)
                .await
                .map_err(wrap_sqlx)?;
            }
        }

        InsertableIndex::Token(tokens) => {
            for token in tokens {
                sqlx::query(
                    "INSERT INTO search_dynamic_token (tenant, project, resource_type, resource_id, param_url, system, code) \
                     VALUES ($1, $2, $3, $4, $5, $6, $7)",
                )
                .bind(tenant)
                .bind(project)
                .bind(resource_type)
                .bind(resource_id)
                .bind(param_url)
                .bind(token.system())
                .bind(token.code())
                .execute(&mut *conn)
                .await
                .map_err(wrap_sqlx)?;
            }
        }

        InsertableIndex::Date(ranges) => {
            for range in ranges {
                sqlx::query(
                    "INSERT INTO search_dynamic_date (tenant, project, resource_type, resource_id, param_url, start_ms, end_ms) \
                     VALUES ($1, $2, $3, $4, $5, $6, $7)",
                )
                .bind(tenant)
                .bind(project)
                .bind(resource_type)
                .bind(resource_id)
                .bind(param_url)
                .bind(range.start)
                .bind(range.end)
                .execute(&mut *conn)
                .await
                .map_err(wrap_sqlx)?;
            }
        }

        InsertableIndex::Number(numbers) => {
            for number in numbers {
                sqlx::query(
                    "INSERT INTO search_dynamic_number (tenant, project, resource_type, resource_id, param_url, value) \
                     VALUES ($1, $2, $3, $4, $5, $6)",
                )
                .bind(tenant)
                .bind(project)
                .bind(resource_type)
                .bind(resource_id)
                .bind(param_url)
                .bind(number)
                .execute(&mut *conn)
                .await
                .map_err(wrap_sqlx)?;
            }
        }

        InsertableIndex::URI(uris) => {
            for uri in uris {
                sqlx::query(
                    "INSERT INTO search_dynamic_uri (tenant, project, resource_type, resource_id, param_url, value) \
                     VALUES ($1, $2, $3, $4, $5, $6)",
                )
                .bind(tenant)
                .bind(project)
                .bind(resource_type)
                .bind(resource_id)
                .bind(param_url)
                .bind(uri)
                .execute(&mut *conn)
                .await
                .map_err(wrap_sqlx)?;
            }
        }

        InsertableIndex::Reference(references) => {
            for reference in references {
                insert_reference_row(
                    conn,
                    tenant,
                    project,
                    resource_type,
                    resource_id,
                    param_url,
                    reference,
                )
                .await?;
            }
        }

        InsertableIndex::Quantity(quantities) => {
            for quantity in quantities {
                sqlx::query(
                    "INSERT INTO search_dynamic_quantity (tenant, project, resource_type, resource_id, param_url, start_value, start_system, start_code, end_value, end_system, end_code) \
                     VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
                )
                .bind(tenant)
                .bind(project)
                .bind(resource_type)
                .bind(resource_id)
                .bind(param_url)
                .bind(quantity.start_value())
                .bind(quantity.start_system())
                .bind(quantity.start_code())
                .bind(quantity.end_value())
                .bind(quantity.end_system())
                .bind(quantity.end_code())
                .execute(&mut *conn)
                .await
                .map_err(wrap_sqlx)?;
            }
        }

        InsertableIndex::DynamicParameters(dynamic_entries) => {
            for entry in dynamic_entries {
                insert_dynamic_parameter(conn, tenant, project, resource_type, resource_id, entry)
                    .await?;
            }
        }

        // Composite and Special have no PG representation yet; Meta is
        // internal bookkeeping that never reaches the index tables.
        InsertableIndex::Composite(_) | InsertableIndex::Special(_) | InsertableIndex::Meta(_) => {}
    }

    Ok(())
}

async fn insert_reference_row(
    conn: &mut sqlx::PgConnection,
    tenant: &str,
    project: &str,
    resource_type: &str,
    resource_id: &str,
    param_url: &str,
    reference: &crate::indexing_conversion::ReferenceIndex,
) -> Result<(), OperationOutcomeError> {
    sqlx::query(
        "INSERT INTO search_dynamic_reference (tenant, project, resource_type, resource_id, param_url, target_resource_type, target_id, target_uri) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
    )
    .bind(tenant)
    .bind(project)
    .bind(resource_type)
    .bind(resource_id)
    .bind(param_url)
    .bind(reference.resource_type())
    .bind(reference.id())
    .bind(reference.uri())
    .execute(conn)
    .await
    .map_err(wrap_sqlx)?;
    Ok(())
}

/// Writes one `DynamicParameterEntry`, which carries its own URL and a value
/// slot per type, into the matching EAV tables.
async fn insert_dynamic_parameter(
    conn: &mut sqlx::PgConnection,
    tenant: &str,
    project: &str,
    resource_type: &str,
    resource_id: &str,
    entry: &crate::indexing_conversion::DynamicParameterEntry,
) -> Result<(), OperationOutcomeError> {
    let param_url = entry.url.as_str();

    if let Some(strings) = &entry.value.string {
        for value in strings {
            sqlx::query(
                "INSERT INTO search_dynamic_string (tenant, project, resource_type, resource_id, param_url, value) \
                 VALUES ($1, $2, $3, $4, $5, $6)",
            )
            .bind(tenant)
            .bind(project)
            .bind(resource_type)
            .bind(resource_id)
            .bind(param_url)
            .bind(value)
            .execute(&mut *conn)
            .await
            .map_err(wrap_sqlx)?;
        }
    }

    if let Some(numbers) = &entry.value.number {
        for number in numbers {
            sqlx::query(
                "INSERT INTO search_dynamic_number (tenant, project, resource_type, resource_id, param_url, value) \
                 VALUES ($1, $2, $3, $4, $5, $6)",
            )
            .bind(tenant)
            .bind(project)
            .bind(resource_type)
            .bind(resource_id)
            .bind(param_url)
            .bind(number)
            .execute(&mut *conn)
            .await
            .map_err(wrap_sqlx)?;
        }
    }

    if let Some(uris) = &entry.value.uri {
        for uri in uris {
            sqlx::query(
                "INSERT INTO search_dynamic_uri (tenant, project, resource_type, resource_id, param_url, value) \
                 VALUES ($1, $2, $3, $4, $5, $6)",
            )
            .bind(tenant)
            .bind(project)
            .bind(resource_type)
            .bind(resource_id)
            .bind(param_url)
            .bind(uri)
            .execute(&mut *conn)
            .await
            .map_err(wrap_sqlx)?;
        }
    }

    if let Some(tokens) = &entry.value.token {
        for token in tokens {
            sqlx::query(
                "INSERT INTO search_dynamic_token (tenant, project, resource_type, resource_id, param_url, system, code) \
                 VALUES ($1, $2, $3, $4, $5, $6, $7)",
            )
            .bind(tenant)
            .bind(project)
            .bind(resource_type)
            .bind(resource_id)
            .bind(param_url)
            .bind(token.system())
            .bind(token.code())
            .execute(&mut *conn)
            .await
            .map_err(wrap_sqlx)?;
        }
    }

    if let Some(dates) = &entry.value.date {
        for date in dates {
            sqlx::query(
                "INSERT INTO search_dynamic_date (tenant, project, resource_type, resource_id, param_url, start_ms, end_ms) \
                 VALUES ($1, $2, $3, $4, $5, $6, $7)",
            )
            .bind(tenant)
            .bind(project)
            .bind(resource_type)
            .bind(resource_id)
            .bind(param_url)
            .bind(date.start)
            .bind(date.end)
            .execute(&mut *conn)
            .await
            .map_err(wrap_sqlx)?;
        }
    }

    if let Some(references) = &entry.value.reference {
        for reference in references {
            insert_reference_row(
                conn,
                tenant,
                project,
                resource_type,
                resource_id,
                param_url,
                reference,
            )
            .await?;
        }
    }

    if let Some(quantities) = &entry.value.quantity {
        for quantity in quantities {
            sqlx::query(
                "INSERT INTO search_dynamic_quantity (tenant, project, resource_type, resource_id, param_url, start_value, start_system, start_code, end_value, end_system, end_code) \
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
            )
            .bind(tenant)
            .bind(project)
            .bind(resource_type)
            .bind(resource_id)
            .bind(param_url)
            .bind(quantity.start_value())
            .bind(quantity.start_system())
            .bind(quantity.start_code())
            .bind(quantity.end_value())
            .bind(quantity.end_system())
            .bind(quantity.end_code())
            .execute(&mut *conn)
            .await
            .map_err(wrap_sqlx)?;
        }
    }

    Ok(())
}

fn wrap_sqlx(e: sqlx::Error) -> OperationOutcomeError {
    OperationOutcomeError::fatal(
        IssueType::exception(),
        format!("PG search indexing failed: {e}"),
    )
}
