use crate::{
    fhir::{CachePolicy, FHIRRepository, ResourceHistoryValue},
    pg::{
        PGConnection, StoreError, VersionCache, decode_resource,
        pending::{self, PendingRows},
        row_resource_type,
        transaction::{commit_transaction, create_transaction},
    },
    types::{FHIRMethod, SupportedFHIRVersions},
    utilities,
};
use haste_fhir_client::{
    request::HistoryRequest,
    url::{ParsedParameter, ParsedParameters},
};
use haste_fhir_model::r4::{
    datetime::parse_datetime,
    generated::{
        resources::{Resource, ResourceType},
        terminology::IssueType,
    },
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_jwt::{ProjectId, ResourceId, TenantId, VersionId, claims::UserTokenClaims};
use sqlx::{
    PgExecutor, Postgres, QueryBuilder, Row, postgres::PgRow, query_builder::Separated,
    types::time::OffsetDateTime,
};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

#[derive(Debug)]
struct ReturnVersionedResource {
    resource: Resource,
    version_id: VersionId,
}

async fn read_version_ids_from_cache<'a>(
    cache: &VersionCache,
    tenant_id: &TenantId,
    project_id: &ProjectId,
    version_ids: &'a [&VersionId],
) -> (Vec<Resource>, Vec<&'a VersionId>) {
    let mut remaining_version_ids = vec![];
    let mut cached_resources = vec![];
    for version_id in version_ids {
        if let Some(resource) = cache.get(tenant_id, project_id, version_id).await {
            cached_resources.push(resource);
        } else {
            remaining_version_ids.push(*version_id);
        }
    }

    (cached_resources, remaining_version_ids)
}

impl FHIRRepository for PGConnection {
    async fn create(
        &self,
        tenant: &TenantId,
        project: &ProjectId,
        author: &UserTokenClaims,
        fhir_version: &SupportedFHIRVersions,
        mut resource: Resource,
    ) -> Result<Resource, OperationOutcomeError> {
        utilities::set_resource_id(&mut resource, None)?;
        utilities::set_resource_meta(&mut resource, &author.resource_type, &author.sub)?;
        write_resource(
            self,
            tenant,
            project,
            author,
            fhir_version,
            resource,
            false,
            "POST",
            FHIRMethod::Create,
        )
        .await
    }

    async fn delete(
        &self,
        tenant: &TenantId,
        project: &ProjectId,
        author: &UserTokenClaims,
        fhir_version: &SupportedFHIRVersions,
        mut resource: Resource,
        id: &str,
    ) -> Result<Resource, OperationOutcomeError> {
        utilities::set_resource_id(&mut resource, Some(id.to_string()))?;
        utilities::set_resource_meta(&mut resource, &author.resource_type, &author.sub)?;
        write_resource(
            self,
            tenant,
            project,
            author,
            fhir_version,
            resource,
            true,
            "DELETE",
            FHIRMethod::Delete,
        )
        .await
    }

    async fn update(
        &self,
        tenant: &TenantId,
        project: &ProjectId,
        author: &UserTokenClaims,
        fhir_version: &SupportedFHIRVersions,
        mut resource: Resource,
        id: &str,
    ) -> Result<Resource, OperationOutcomeError> {
        utilities::set_resource_id(&mut resource, Some(id.to_string()))?;
        utilities::set_resource_meta(&mut resource, &author.resource_type, &author.sub)?;
        write_resource(
            self,
            tenant,
            project,
            author,
            fhir_version,
            resource,
            false,
            "PUT",
            FHIRMethod::Update,
        )
        .await
    }

    async fn read_by_version_ids(
        &self,
        tenant_id: &TenantId,
        project_id: &ProjectId,
        version_ids: &[&VersionId],
        cache_policy: CachePolicy,
    ) -> Result<Vec<Resource>, OperationOutcomeError> {
        if version_ids.is_empty() {
            return Ok(Vec::new());
        }

        let (cached_result, remaining_version_ids) =
            read_version_ids_from_cache(self.cache(), tenant_id, project_id, version_ids).await;

        if remaining_version_ids.is_empty() {
            return Ok(cached_result);
        }

        match self {
            PGConnection::Pool(pool, cache) => {
                let res = read_by_version_ids(pool, tenant_id, project_id, &remaining_version_ids)
                    .await?;

                if cache_policy == CachePolicy::Cache {
                    for v in &res {
                        cache
                            .insert(
                                tenant_id,
                                project_id,
                                v.version_id.clone(),
                                v.resource.clone(),
                            )
                            .await;
                    }
                }

                Ok(cached_result
                    .into_iter()
                    .chain(res.into_iter().map(|r| r.resource))
                    .collect::<Vec<_>>())
            }
            PGConnection::Transaction(tx, cache, pending) => {
                pending.flush(tx).await?;
                let mut conn = tx.lock().await;
                // Handle PgConnection connection
                let res =
                    read_by_version_ids(&mut **conn, tenant_id, project_id, &remaining_version_ids)
                        .await?;

                if cache_policy == CachePolicy::Cache {
                    for v in &res {
                        cache
                            .insert(
                                tenant_id,
                                project_id,
                                v.version_id.clone(),
                                v.resource.clone(),
                            )
                            .await;
                    }
                }

                Ok(cached_result
                    .into_iter()
                    .chain(res.into_iter().map(|r| r.resource))
                    .collect::<Vec<_>>())
            }
        }
    }

    async fn read_version(
        &self,
        tenant_id: &TenantId,
        project_id: &ProjectId,
        resource_type: &ResourceType,
        resource_id: &ResourceId,
        version_id: &VersionId,
    ) -> Result<Option<Resource>, OperationOutcomeError> {
        match self {
            PGConnection::Pool(pool, _) => {
                read_version(
                    pool,
                    tenant_id,
                    project_id,
                    resource_type,
                    resource_id,
                    version_id,
                )
                .await
            }
            PGConnection::Transaction(tx, _, pending) => {
                pending.flush(tx).await?;
                let mut conn = tx.lock().await;
                read_version(
                    &mut **conn,
                    tenant_id,
                    project_id,
                    resource_type,
                    resource_id,
                    version_id,
                )
                .await
            }
        }
    }

    async fn read_latest(
        &self,
        tenant_id: &TenantId,
        project_id: &ProjectId,
        resource_type: &ResourceType,
        resource_id: &ResourceId,
    ) -> Result<Option<Resource>, OperationOutcomeError> {
        match self {
            PGConnection::Pool(pool, _) => {
                let res =
                    read_latest(pool, tenant_id, project_id, resource_type, resource_id).await?;
                Ok(res)
            }
            PGConnection::Transaction(tx, _, pending) => {
                pending.flush(tx).await?;
                let mut conn = tx.lock().await;
                // Handle PgConnection connection
                read_latest(
                    &mut **conn,
                    tenant_id,
                    project_id,
                    resource_type,
                    resource_id,
                )
                .await
            }
        }
    }

    async fn read_latest_many(
        &self,
        tenant_id: &TenantId,
        project_id: &ProjectId,
        keys: &[(ResourceType, ResourceId)],
    ) -> Result<Vec<Resource>, OperationOutcomeError> {
        if keys.is_empty() {
            return Ok(Vec::new());
        }

        match self {
            PGConnection::Pool(pool, _) => {
                read_latest_many(pool, tenant_id, project_id, keys).await
            }
            PGConnection::Transaction(tx, _, pending) => {
                pending.flush(tx).await?;
                let mut conn = tx.lock().await;
                read_latest_many(&mut **conn, tenant_id, project_id, keys).await
            }
        }
    }

    async fn history(
        &self,
        tenant_id: &TenantId,
        project_id: &ProjectId,
        request: &HistoryRequest,
    ) -> Result<Vec<ResourceHistoryValue>, OperationOutcomeError> {
        match self {
            PGConnection::Pool(pool, _) => history(pool, tenant_id, project_id, request).await,
            PGConnection::Transaction(tx, _, pending) => {
                pending.flush(tx).await?;
                let mut conn = tx.lock().await;
                // Handle PgConnection connection
                history(&mut **conn, tenant_id, project_id, request).await
            }
        }
    }

    fn in_transaction(&self) -> bool {
        matches!(self, PGConnection::Transaction(_tx, _, _))
    }

    async fn transaction(&self, is_updating_sequence: bool) -> Result<Self, OperationOutcomeError> {
        let tx = create_transaction(self, is_updating_sequence).await?;
        let pending = match self {
            PGConnection::Transaction(_, _, pending) => pending.clone(),
            PGConnection::Pool(_, _) => PendingRows::new(),
        };
        Ok(PGConnection::Transaction(tx, self.cache().clone(), pending))
    }

    async fn commit(self) -> Result<(), OperationOutcomeError> {
        match self {
            PGConnection::Pool(_pool, _) => Err(StoreError::NotTransaction.into()),
            PGConnection::Transaction(tx, _, pending) => {
                pending.flush(&tx).await?;
                commit_transaction(tx).await
            }
        }
    }

    async fn rollback(self) -> Result<(), OperationOutcomeError> {
        match self {
            PGConnection::Pool(_pool, _) => Err(StoreError::NotTransaction.into()),
            PGConnection::Transaction(tx, _, _pending) => {
                let conn = Mutex::into_inner(
                    Arc::try_unwrap(tx).map_err(|_e| StoreError::FailedCommitTransaction)?,
                );

                // Handle PgConnection connection
                conn.rollback().await.map_err(StoreError::from)?;
                Ok(())
            }
        }
    }
}

/// Inserts (Pool) or buffers (Transaction) a `resources` row for
/// create/update/delete and hands the resource back to the caller. The
/// `resource` argument must already be mutated (`id`/`meta` set) by the
/// caller. A `Pool` write is never buffered, so it's inserted straight from
/// a borrow of `resource` with no clone; a `Transaction` write is queued via
/// `PendingRows::push`, which needs its own owned copy since it must outlive
/// this call until flush/commit.
#[allow(clippy::too_many_arguments)]
async fn write_resource(
    conn: &PGConnection,
    tenant: &TenantId,
    project: &ProjectId,
    author: &UserTokenClaims,
    fhir_version: &SupportedFHIRVersions,
    resource: Resource,
    deleted: bool,
    request_method: &'static str,
    fhir_method: FHIRMethod,
) -> Result<Resource, OperationOutcomeError> {
    match conn {
        PGConnection::Pool(_pool, _) => {
            let tx = create_transaction(conn, true).await?;
            {
                let mut c = tx.lock().await;
                pending::execute(
                    &mut **c,
                    tenant,
                    project,
                    author,
                    fhir_version,
                    &resource,
                    deleted,
                    request_method,
                    fhir_method,
                )
                .await?;
            }
            commit_transaction(tx).await?;
        }
        PGConnection::Transaction(_tx, _, pending) => {
            pending
                .push(
                    tenant,
                    project,
                    author,
                    fhir_version,
                    resource.clone(),
                    deleted,
                    request_method,
                    fhir_method,
                )
                .await;
        }
    }

    Ok(resource)
}

async fn read_by_version_ids<'a, 'e, E>(
    executor: E,
    tenant_id: &'a TenantId,
    project_id: &'a ProjectId,
    version_ids: &'a Vec<&'a VersionId>,
) -> Result<Vec<ReturnVersionedResource>, OperationOutcomeError>
where
    E: PgExecutor<'e>,
{
    let bound_version_ids: Vec<&str> = version_ids
        .iter()
        .map(std::convert::AsRef::as_ref)
        .collect();

    // Rows are decoded in `try_map` as they stream in, overlapping with the
    // server sending the rest of the result.
    let mut response = sqlx::query(
        r"
            SELECT resource, resource_type, version_id
            FROM resources
            WHERE tenant = $1 AND project = $2 AND version_id = ANY($3::text[])
        ",
    )
    .bind(tenant_id.as_ref())
    .bind(project_id.as_ref())
    .bind(&bound_version_ids)
    .try_map(|row: PgRow| {
        let resource_type = row_resource_type(&row)?;
        Ok(ReturnVersionedResource {
            resource: decode_resource(&row, &resource_type)?,
            version_id: row.try_get("version_id")?,
        })
    })
    .fetch_all(executor)
    .await
    .map_err(|e| match e {
        sqlx::Error::Decode(e) => StoreError::DeserializeError(e.to_string()),
        e => StoreError::from(e),
    })?;

    // Create a lookup table to remember the original order of the requested version IDs.
    let mut requested_order: HashMap<&VersionId, usize> = HashMap::with_capacity(version_ids.len());
    for (index, version_id) in version_ids.iter().enumerate() {
        requested_order.insert(*version_id, index);
    }

    response.sort_by_key(|r| {
        requested_order
            .get(&r.version_id)
            .copied()
            .unwrap_or(usize::MAX)
    });

    Ok(response)
}

/// The version `version_id` of `resource_type/resource_id`. The primary key
/// finds the row by its version id; the type and id leave it out unless it
/// is a version of that resource.
async fn read_version<'a, 'e, E>(
    executor: E,
    tenant_id: &'a TenantId,
    project_id: &'a ProjectId,
    resource_type: &'a ResourceType,
    resource_id: &'a ResourceId,
    version_id: &'a VersionId,
) -> Result<Option<Resource>, OperationOutcomeError>
where
    E: PgExecutor<'e>,
{
    let response = sqlx::query(
        r"
            SELECT resource
            FROM resources
            WHERE tenant = $1 AND project = $2 AND version_id = $3
                AND resource_type = $4 AND id = $5
        ",
    )
    .bind(tenant_id.as_ref())
    .bind(project_id.as_ref())
    .bind(version_id.as_ref())
    .bind(resource_type.as_ref())
    .bind(resource_id.as_ref())
    .try_map(|row: PgRow| decode_resource(&row, resource_type))
    .fetch_optional(executor)
    .await
    .map_err(StoreError::from)?;

    Ok(response)
}

async fn read_latest<'a, 'e, E>(
    executor: E,
    tenant_id: &'a TenantId,
    project_id: &'a ProjectId,
    resource_type: &'a ResourceType,
    resource_id: &'a ResourceId,
) -> Result<Option<Resource>, OperationOutcomeError>
where
    E: PgExecutor<'e>,
{
    let response = sqlx::query(
        r"
            SELECT resource, deleted
            FROM resources
            WHERE tenant = $1 AND project = $2 AND id = $3 AND resource_type = $4
            ORDER BY sequence DESC
            LIMIT 1
        ",
    )
    .bind(tenant_id.as_ref())
    .bind(project_id.as_ref())
    .bind(resource_id.as_ref())
    .bind(resource_type.as_ref())
    .try_map(|row: PgRow| {
        if row.try_get("deleted")? {
            return Ok(None);
        }
        decode_resource(&row, resource_type).map(Some)
    })
    .fetch_optional(executor)
    .await
    .map_err(StoreError::from)?;

    Ok(response.flatten())
}

/// The lookup of [`read_latest`] for every key, in one statement. The lateral
/// subquery is one probe of `resources_id_idx` per key stopping at the newest
/// version, so the cost is the same as the single reads without their round
/// trips.
async fn read_latest_many<'a, 'e, E>(
    executor: E,
    tenant_id: &'a TenantId,
    project_id: &'a ProjectId,
    keys: &'a [(ResourceType, ResourceId)],
) -> Result<Vec<Resource>, OperationOutcomeError>
where
    E: PgExecutor<'e>,
{
    let types: Vec<&str> = keys.iter().map(|(type_, _)| type_.as_ref()).collect();
    let ids: Vec<&str> = keys.iter().map(|(_, id)| id.as_ref()).collect();

    let rows = sqlx::query(
        r"
            SELECT r.resource, r.deleted, k.resource_type
            FROM unnest($3::text[], $4::text[]) AS k(resource_type, id)
            CROSS JOIN LATERAL (
                SELECT resource, deleted
                FROM resources
                WHERE tenant = $1 AND project = $2
                  AND resource_type = k.resource_type AND id = k.id
                ORDER BY sequence DESC
                LIMIT 1
            ) r
        ",
    )
    .bind(tenant_id.as_ref())
    .bind(project_id.as_ref())
    .bind(&types)
    .bind(&ids)
    .try_map(|row: PgRow| {
        if row.try_get("deleted")? {
            return Ok(None);
        }
        decode_resource(&row, &row_resource_type(&row)?).map(Some)
    })
    .fetch_all(executor)
    .await
    .map_err(StoreError::from)?;

    Ok(rows.into_iter().flatten().collect())
}

fn process_history_parameters(
    parameters: &ParsedParameters,
    clauses: &mut Separated<'_, Postgres, &str>,
) -> Result<(), OperationOutcomeError> {
    for parameter in parameters.parameters() {
        match parameter {
            ParsedParameter::Result(result_param) => {
                if result_param.name.as_str() == "_since" {
                    if let Some(value) = result_param.value.first() {
                        let date_time = parse_datetime(value.as_str()).map_err(|e| {
                            OperationOutcomeError::fatal(
                                IssueType::invalid(),
                                format!("Invalid _since parameter datetime: {e:?}"),
                            )
                        })?;

                        let since = chrono::DateTime::try_from(date_time).map_err(|e| {
                            OperationOutcomeError::fatal(
                                IssueType::invalid(),
                                format!("Invalid _since parameter datetime: {e:?}"),
                            )
                        })?;
                        // Bound as a `time` value: sqlx's `chrono` feature stays off so
                        // its query macros map timestamps to one crate only.
                        let since = OffsetDateTime::from_unix_timestamp_nanos(
                            i128::from(since.timestamp()) * 1_000_000_000
                                + i128::from(since.timestamp_subsec_nanos()),
                        )
                        .map_err(|e| {
                            OperationOutcomeError::fatal(
                                IssueType::invalid(),
                                format!("Invalid _since parameter datetime: {e}"),
                            )
                        })?;

                        clauses.push(" created_at >= ").push_bind_unseparated(since);
                    }
                } else {
                    // Ignore offset and count parameter as these parameters are held separately and not used in the where clause.
                }
            }
            ParsedParameter::Resource(_) => {
                return Err(OperationOutcomeError::fatal(
                    IssueType::not_supported(),
                    format!(
                        "Parameter '{}' is not supported for history requests.",
                        parameter.name()
                    ),
                ));
            }
        }
    }

    Ok(())
}

async fn history<'a, 'e, E>(
    executor: E,
    tenant: &'a TenantId,
    project: &'a ProjectId,
    history_request: &'a HistoryRequest,
) -> Result<Vec<ResourceHistoryValue>, OperationOutcomeError>
where
    E: PgExecutor<'e>,
{
    let mut query_builder: QueryBuilder<sqlx::Postgres> =
        QueryBuilder::new(r"SELECT resource, resource_type, request_method FROM resources WHERE ");

    let mut clauses = query_builder.separated(" AND ");
    clauses
        .push(" tenant = ")
        .push_bind_unseparated(tenant.as_ref())
        .push(" project = ")
        .push_bind_unseparated(project.as_ref());

    let history_parameters = match history_request {
        HistoryRequest::Instance(history_instance_request) => &history_instance_request.parameters,
        HistoryRequest::Type(history_type_request) => &history_type_request.parameters,
        HistoryRequest::System(system_request) => &system_request.parameters,
    };

    process_history_parameters(history_parameters, &mut clauses)?;

    match history_request {
        HistoryRequest::Instance(history_instance_request) => {
            clauses
                .push(" resource_type = ")
                .push_bind_unseparated(history_instance_request.resource_type.as_ref())
                .push(" id = ")
                .push_bind_unseparated(&history_instance_request.id);
        }
        HistoryRequest::Type(history_type_request) => {
            clauses
                .push(" resource_type = ")
                .push_bind_unseparated(history_type_request.resource_type.as_ref());
        }
        HistoryRequest::System(_request) => {}
    }

    let limit = if let Some(ParsedParameter::Result(count_param)) = history_parameters.get("_count")
    {
        std::cmp::min(
            1000,
            count_param
                .value
                .first()
                .and_then(|v| v.parse::<i64>().ok())
                .unwrap_or(100),
        )
    } else {
        1000
    };

    if limit < 0 {
        return Err(OperationOutcomeError::fatal(
            IssueType::invalid(),
            "Invalid _count parameter value. Must be greater than or equal to 0.".to_string(),
        ));
    }

    query_builder
        .push(" ORDER BY sequence DESC LIMIT ")
        .push_bind(limit);

    if let Some(ParsedParameter::Result(offset_param)) = history_parameters.get("_offset") {
        let offset = offset_param
            .value
            .first()
            .and_then(|v| v.parse::<i64>().ok())
            .unwrap_or(0);

        if offset < 0 {
            return Err(OperationOutcomeError::fatal(
                IssueType::invalid(),
                "Invalid _offset parameter value. Must be greater than or equal to 0.".to_string(),
            ));
        }

        query_builder.push(" OFFSET ").push_bind(offset);
    }

    let result = query_builder
        .build()
        .try_map(|row: PgRow| {
            let resource_type = row_resource_type(&row)?;
            Ok(ResourceHistoryValue {
                resource: decode_resource(&row, &resource_type)?,
                request_method: row.try_get("request_method")?,
            })
        })
        .fetch_all(executor)
        .await
        .map_err(StoreError::from)?;

    Ok(result)
}
