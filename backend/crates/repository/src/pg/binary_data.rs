//! Lookup of extension values on `Binary.data` across stored versions.

use std::collections::HashSet;

use haste_fhir_operation_error::OperationOutcomeError;
use haste_jwt::{ProjectId, TenantId};
use sqlx::PgExecutor;

use crate::pg::{PGConnection, StoreError};

impl PGConnection {
    /// Which of `values` occur as the `valueString` of an extension with
    /// `url` on `Binary.data` in a stored version of the project. Every
    /// version counts, deleted ones included.
    ///
    /// The partial GIN index on Binary rows' `_data`
    /// (`resources_binary_data_idx`) serves this, so it costs one containment
    /// probe per value rather than a scan of the project's Binary rows.
    ///
    /// # Errors
    ///
    /// Fails when the database does.
    pub async fn binary_data_extension_values(
        &self,
        tenant: &TenantId,
        project: &ProjectId,
        url: &str,
        values: &[&str],
    ) -> Result<HashSet<String>, OperationOutcomeError> {
        if values.is_empty() {
            return Ok(HashSet::new());
        }

        let found = match self {
            PGConnection::Pool(pool, _) => {
                binary_data_extension_values(pool, tenant, project, url, values).await?
            }
            PGConnection::Transaction(tx, _, pending) => {
                pending.flush(tx).await?;
                let mut conn = tx.lock().await;
                binary_data_extension_values(&mut **conn, tenant, project, url, values).await?
            }
        };
        Ok(found.into_iter().collect())
    }
}

/// `resource_type = 'Binary'` and `resource -> '_data'` are spelled out so the
/// planner can match the partial index; FHIR JSON keeps a primitive's
/// extensions under the underscored element name.
async fn binary_data_extension_values<'e, E>(
    executor: E,
    tenant: &TenantId,
    project: &ProjectId,
    url: &str,
    values: &[&str],
) -> Result<Vec<String>, StoreError>
where
    E: PgExecutor<'e>,
{
    sqlx::query_scalar::<_, String>(
        r"
            SELECT wanted.value
            FROM unnest($3::text[]) AS wanted(value)
            WHERE EXISTS (
                SELECT 1
                FROM resources
                WHERE tenant = $1 AND project = $2 AND resource_type = 'Binary'
                  AND (resource -> '_data') @> jsonb_build_object(
                        'extension',
                        jsonb_build_array(
                            jsonb_build_object('url', $4::text, 'valueString', wanted.value)
                        )
                      )
            )
        ",
    )
    .bind(tenant.as_ref())
    .bind(project.as_ref())
    .bind(values)
    .bind(url)
    .fetch_all(executor)
    .await
    .map_err(StoreError::from)
}
