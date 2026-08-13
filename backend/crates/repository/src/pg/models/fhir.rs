use crate::{
    fhir::{CachePolicy, FHIRRepository, ResourceHistoryValue},
    pg::{
        PGConnection, StoreError,
        pending::{self, PendingRows},
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
    sqlx::FHIRJson,
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_jwt::{ProjectId, ResourceId, TenantId, VersionId, claims::UserTokenClaims};
use moka::future::Cache;
use sqlx::{PgExecutor, Postgres, QueryBuilder, query_builder::Separated};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(sqlx::FromRow, Debug)]
struct ReturnVersionedResource {
    resource: FHIRJson<Resource>,
    version_id: VersionId,
}

#[derive(sqlx::FromRow, Debug)]
struct HistoryValue {
    pub resource: FHIRJson<Resource>,
    pub request_method: String,
}

async fn read_version_ids_from_cache<'a>(
    cache: &Cache<VersionId, Resource>,
    version_ids: &'a [&VersionId],
) -> (Vec<Resource>, Vec<&'a VersionId>) {
    let mut remaining_version_ids = vec![];
    let mut cached_resources = vec![];
    for version_id in version_ids {
        if let Some(resource) = cache.get(*version_id).await {
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
            read_version_ids_from_cache(self.cache(), version_ids).await;

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
                            .insert(v.version_id.clone(), v.resource.0.clone())
                            .await;
                    }
                }

                Ok(cached_result
                    .into_iter()
                    .chain(res.into_iter().map(|r| r.resource.0))
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
                            .insert(v.version_id.clone(), v.resource.0.clone())
                            .await;
                    }
                }

                Ok(cached_result
                    .into_iter()
                    .chain(res.into_iter().map(|r| r.resource.0))
                    .collect::<Vec<_>>())
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
    let mut query_builder: QueryBuilder<sqlx::Postgres> =
        QueryBuilder::new("SELECT resource, version_id FROM resources WHERE tenant = ");

    query_builder
        .push_bind(tenant_id.as_ref())
        .push(" AND project =")
        .push_bind(project_id.as_ref());

    query_builder.push(" AND version_id in (");

    let mut separated = query_builder.separated(", ");
    for version_id in version_ids {
        separated.push_bind(version_id.as_ref());
    }
    separated.push_unseparated(")");

    query_builder.push(" ORDER BY array_position(array[");
    let mut order_separator = query_builder.separated(", ");
    for version_id in version_ids {
        order_separator.push_bind(version_id.as_ref());
    }
    query_builder.push("], version_id)");

    let query = query_builder.build_query_as::<ReturnVersionedResource>();

    let response: Vec<ReturnVersionedResource> =
        query.fetch_all(executor).await.map_err(StoreError::from)?;

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
    let response = sqlx::query_as::<_, (FHIRJson<Resource>, bool)>(
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
    .fetch_optional(executor)
    .await
    .map_err(StoreError::from)?;

    match response {
        Some((_, true)) | None => Ok(None),
        Some((json, _)) => Ok(Some(json.0)),
    }
}

fn process_history_parameters<'a>(
    parameters: &'a ParsedParameters,
    clauses: &mut Separated<'_, 'a, Postgres, &str>,
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

                        clauses.push(" created_at >= ").push_bind_unseparated(
                            chrono::DateTime::try_from(date_time).map_err(|e| {
                                OperationOutcomeError::fatal(
                                    IssueType::invalid(),
                                    format!("Invalid _since parameter datetime: {e:?}"),
                                )
                            })?,
                        );
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
        QueryBuilder::new(r"SELECT resource, request_method FROM resources WHERE ");

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

    let query = query_builder.build_query_as::<HistoryValue>();

    let result: Vec<HistoryValue> = query.fetch_all(executor).await.map_err(StoreError::from)?;

    Ok(result
        .into_iter()
        .map(|r| ResourceHistoryValue {
            resource: r.resource.0,
            request_method: r.request_method,
        })
        .collect::<Vec<_>>())
}
