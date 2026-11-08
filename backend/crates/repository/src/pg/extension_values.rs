//! Lookup of primitive extension values across stored resource versions.

use std::collections::HashSet;

use haste_fhir_model::r4::generated::resources::ResourceType;
use haste_fhir_operation_error::OperationOutcomeError;
use haste_jwt::{ProjectId, TenantId};
use sqlx::PgExecutor;

use crate::pg::{PGConnection, StoreError};

impl PGConnection {
    /// Which of `values` occur as the `valueString` of an extension with
    /// `url` on the top-level primitive element `element` of a stored version
    /// of `resource_type` in the project. Every version counts, deleted ones
    /// included.
    ///
    /// # Errors
    ///
    /// Fails when the database does.
    pub async fn primitive_extension_values(
        &self,
        tenant: &TenantId,
        project: &ProjectId,
        resource_type: &ResourceType,
        element: &str,
        url: &str,
        values: &[&str],
    ) -> Result<HashSet<String>, OperationOutcomeError> {
        if values.is_empty() {
            return Ok(HashSet::new());
        }
        // FHIR JSON keeps a primitive's extensions under `_<element>`.
        let extension_element = format!("_{element}");

        let found = match self {
            PGConnection::Pool(pool, _) => {
                primitive_extension_values(
                    pool,
                    tenant,
                    project,
                    resource_type,
                    &extension_element,
                    url,
                    values,
                )
                .await?
            }
            PGConnection::Transaction(tx, _, pending) => {
                pending.flush(tx).await?;
                let mut conn = tx.lock().await;
                primitive_extension_values(
                    &mut **conn,
                    tenant,
                    project,
                    resource_type,
                    &extension_element,
                    url,
                    values,
                )
                .await?
            }
        };
        Ok(found.into_iter().collect())
    }
}

async fn primitive_extension_values<'e, E>(
    executor: E,
    tenant: &TenantId,
    project: &ProjectId,
    resource_type: &ResourceType,
    extension_element: &str,
    url: &str,
    values: &[&str],
) -> Result<Vec<String>, StoreError>
where
    E: PgExecutor<'e>,
{
    sqlx::query_scalar::<_, String>(
        r"
            SELECT DISTINCT extension->>'valueString'
            FROM resources,
                 jsonb_array_elements(COALESCE(resource->($4::text)->'extension', '[]'::jsonb)) AS extension
            WHERE tenant = $1 AND project = $2 AND resource_type = $3
              AND extension->>'url' = $5
              AND extension->>'valueString' = ANY($6::text[])
        ",
    )
    .bind(tenant.as_ref())
    .bind(project.as_ref())
    .bind(resource_type.as_ref())
    .bind(extension_element)
    .bind(url)
    .bind(values)
    .fetch_all(executor)
    .await
    .map_err(StoreError::from)
}
