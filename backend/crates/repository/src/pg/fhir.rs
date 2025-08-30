use crate::{
    fhir::{FHIRRepository, HistoryRequest, ResourcePollingValue},
    pg::{PGConnection, StoreError},
    types::{
        Author, FHIRMethod, ProjectId, ResourceId, SupportedFHIRVersions, TenantId, VersionIdRef,
    },
    utilities,
};
use oxidized_fhir_model::r4::{
    sqlx::{FHIRJson, FHIRJsonRef},
    types::{Resource, ResourceType},
};
use oxidized_fhir_operation_error::OperationOutcomeError;
use sqlx::{Acquire, Postgres, QueryBuilder};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(sqlx::FromRow, Debug)]
struct ReturnV {
    resource: FHIRJson<Resource>,
}

impl FHIRRepository for PGConnection {
    async fn create(
        &self,
        tenant: &TenantId,
        project: &ProjectId,
        author: &Author,
        fhir_version: &SupportedFHIRVersions,
        resource: &mut Resource,
    ) -> Result<Resource, OperationOutcomeError> {
        match &self {
            PGConnection::PgPool(pool) => {
                let res = create(pool, tenant, project, author, fhir_version, resource).await?;
                Ok(res)
            }
            PGConnection::PgTransaction(tx) => {
                let mut tx = tx.lock().await;
                let res = create(&mut *tx, tenant, project, author, fhir_version, resource).await?;
                Ok(res)
            }
        }
    }

    async fn update(
        &self,
        tenant: &TenantId,
        project: &ProjectId,
        author: &Author,
        fhir_version: &SupportedFHIRVersions,
        resource: &mut Resource,
        id: &str,
    ) -> Result<Resource, OperationOutcomeError> {
        match self {
            PGConnection::PgPool(pool) => {
                let res = update(pool, tenant, project, author, fhir_version, resource, id).await?;
                Ok(res)
            }
            PGConnection::PgTransaction(tx) => {
                let mut conn = tx.lock().await;
                // Handle PgConnection connection
                let res = update(
                    &mut *conn,
                    tenant,
                    project,
                    author,
                    fhir_version,
                    resource,
                    id,
                )
                .await?;
                Ok(res)
            }
        }
    }

    async fn read_by_version_ids(
        &self,
        tenant_id: &TenantId,
        project_id: &ProjectId,
        version_ids: Vec<VersionIdRef<'_>>,
    ) -> Result<Vec<Resource>, OperationOutcomeError> {
        if version_ids.is_empty() {
            return Ok(vec![]);
        }

        match self {
            PGConnection::PgPool(pool) => {
                let res = read_by_version_ids(pool, tenant_id, project_id, version_ids).await?;
                Ok(res)
            }
            PGConnection::PgTransaction(tx) => {
                let mut conn = tx.lock().await;
                // Handle PgConnection connection
                let res =
                    read_by_version_ids(&mut *conn, tenant_id, project_id, version_ids).await?;
                Ok(res)
            }
        }
    }

    async fn read_latest(
        &self,
        tenant_id: &TenantId,
        project_id: &ProjectId,
        resource_type: &ResourceType,
        resource_id: &ResourceId,
    ) -> Result<Option<oxidized_fhir_model::r4::types::Resource>, OperationOutcomeError> {
        match self {
            PGConnection::PgPool(pool) => {
                let res =
                    read_latest(pool, tenant_id, project_id, resource_type, resource_id).await?;
                Ok(res)
            }
            PGConnection::PgTransaction(tx) => {
                let mut conn = tx.lock().await;
                // Handle PgConnection connection
                let res = read_latest(
                    &mut *conn,
                    tenant_id,
                    project_id,
                    resource_type,
                    resource_id,
                )
                .await?;
                Ok(res)
            }
        }
    }

    async fn history(
        &self,
        tenant_id: &TenantId,
        project_id: &ProjectId,
        request: HistoryRequest<'_>,
    ) -> Result<Vec<Resource>, OperationOutcomeError> {
        match self {
            PGConnection::PgPool(pool) => {
                let res = history(pool, tenant_id, project_id, request).await?;
                Ok(res)
            }
            PGConnection::PgTransaction(tx) => {
                let mut conn = tx.lock().await;
                // Handle PgConnection connection
                let res = history(&mut *conn, tenant_id, project_id, request).await?;
                Ok(res)
            }
        }
    }

    async fn get_sequence(
        &self,
        tenant_id: &TenantId,
        sequence_id: u64,
        count: Option<u64>,
    ) -> Result<Vec<ResourcePollingValue>, OperationOutcomeError> {
        match self {
            PGConnection::PgPool(pool) => {
                let res = get_sequence(pool, tenant_id, sequence_id, count).await?;
                Ok(res)
            }
            PGConnection::PgTransaction(tx) => {
                let mut conn = tx.lock().await;
                // Handle PgConnection connection
                let res = get_sequence(&mut *conn, tenant_id, sequence_id, count).await?;
                Ok(res)
            }
        }
    }

    async fn transaction<'a>(&'a self) -> Result<Self, OperationOutcomeError> {
        match self {
            PGConnection::PgPool(pool) => {
                let tx = pool.begin().await.map_err(StoreError::from)?;
                Ok(PGConnection::PgTransaction(Arc::new(Mutex::new(tx))))
            }
            PGConnection::PgTransaction(tx) => {
                let tx = tx.clone();
                Ok(PGConnection::PgTransaction(tx))
            } // Transaction doesn't live long enough so cannot create.
        }
    }

    async fn commit(self) -> Result<(), OperationOutcomeError> {
        match self {
            PGConnection::PgPool(_pool) => Err(StoreError::NotTransaction.into()),
            PGConnection::PgTransaction(tx) => {
                let conn = Mutex::into_inner(
                    Arc::try_unwrap(tx).map_err(|_e| StoreError::FailedCommitTransaction)?,
                );

                // Handle PgConnection connection
                let res = conn.commit().await.map_err(StoreError::from)?;
                Ok(res)
            }
        }
    }
    async fn rollback(self) -> Result<(), OperationOutcomeError> {
        match self {
            PGConnection::PgPool(_pool) => Err(StoreError::NotTransaction.into()),
            PGConnection::PgTransaction(tx) => {
                let conn = Mutex::into_inner(
                    Arc::try_unwrap(tx).map_err(|_e| StoreError::FailedCommitTransaction)?,
                );

                // Handle PgConnection connection
                let res = conn.rollback().await.map_err(StoreError::from)?;
                Ok(res)
            }
        }
    }
}

fn create<'a, 'c, Connection: Acquire<'c, Database = Postgres> + Send + 'a>(
    connection: Connection,
    tenant: &'a TenantId,
    project: &'a ProjectId,
    author: &'a Author,
    fhir_version: &'a SupportedFHIRVersions,
    resource: &'a mut Resource,
) -> impl Future<Output = Result<Resource, OperationOutcomeError>> + Send + 'a {
    async move {
        utilities::set_resource_id(resource, None)?;
        utilities::set_version_id(resource)?;
        let mut conn = connection.acquire().await.map_err(StoreError::SQLXError)?;
        let result = sqlx::query_as!(
                ReturnV,
                r#"INSERT INTO resources (tenant, project, author_id, fhir_version, resource, deleted, request_method, author_type, fhir_method) 
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) 
                RETURNING resource as "resource: FHIRJson<Resource>""#,
                tenant.as_ref() as &str,
                project.as_ref() as &str,
                author.id,
                // Useless cast so that macro has access to the type information.
                // Otherwise it will not compile on type check.
                fhir_version as &SupportedFHIRVersions,
                &FHIRJsonRef(resource) as &FHIRJsonRef<'_ , Resource>,
                false, // deleted
                "POST",
                author.kind,
                &FHIRMethod::Create as &FHIRMethod,
            ).fetch_one(&mut *conn).await.map_err(StoreError::from)?;
        Ok(result.resource.0)
    }
}

fn update<'a, 'c, Connection: Acquire<'c, Database = Postgres> + Send + 'a>(
    connection: Connection,
    tenant: &'a TenantId,
    project: &'a ProjectId,
    author: &'a Author,
    fhir_version: &'a SupportedFHIRVersions,
    resource: &'a mut Resource,
    id: &'a str,
) -> impl Future<Output = Result<Resource, OperationOutcomeError>> + Send + 'a {
    async move {
        utilities::set_resource_id(resource, Some(id.to_string()))?;
        utilities::set_version_id(resource)?;

        let mut conn = connection.acquire().await.map_err(StoreError::SQLXError)?;

        let query = sqlx::query_as!(
            ReturnV,
            r#"INSERT INTO resources (tenant, project, author_id, fhir_version, resource, deleted, request_method, author_type, fhir_method) 
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) 
                RETURNING resource as "resource: FHIRJson<Resource>""#,
            tenant.as_ref() as &str,
            project.as_ref() as &str,
            author.id,
            // Useless cast so that macro has access to the type information.
            // Otherwise it will not compile on type check.
            fhir_version as &SupportedFHIRVersions,
            &FHIRJsonRef(resource) as &FHIRJsonRef<'_, Resource>,
            false, // deleted
            "PUT",
            author.kind,
            &FHIRMethod::Update as &FHIRMethod,
        );

        let result = query
            .fetch_one(&mut *conn)
            .await
            .map_err(StoreError::from)?;

        Ok(result.resource.0)
    }
}

fn read_by_version_ids<'a, 'c, Connection: Acquire<'c, Database = Postgres> + Send + 'a>(
    connection: Connection,
    tenant_id: &'a TenantId,
    project_id: &'a ProjectId,
    version_ids: Vec<VersionIdRef<'a>>,
) -> impl Future<Output = Result<Vec<Resource>, OperationOutcomeError>> + Send + 'a {
    async move {
        let mut conn = connection.acquire().await.map_err(StoreError::SQLXError)?;

        let mut query_builder: QueryBuilder<Postgres> =
            QueryBuilder::new(r#"SELECT resource FROM resources WHERE tenant = "#);

        query_builder
            .push_bind(tenant_id.as_ref())
            .push(" AND project =")
            .push_bind(project_id.as_ref());

        query_builder.push(" AND version_id in (");

        let mut separated = query_builder.separated(", ");
        for version_id in version_ids.iter() {
            separated.push_bind(version_id.as_ref());
        }
        separated.push_unseparated(")");

        // To preserve sort order.
        query_builder.push(" ORDER BY  array_position(array[");
        let mut order_separator = query_builder.separated(", ");
        for version_id in version_ids.iter() {
            order_separator.push_bind(version_id.as_ref());
        }
        query_builder.push("], version_id)");

        let query = query_builder.build_query_as();
        let response: Vec<ReturnV> = query
            .fetch_all(&mut *conn)
            .await
            .map_err(StoreError::from)?;

        Ok(response.into_iter().map(|r| r.resource.0).collect())
    }
}

fn read_latest<'a, 'c, Connection: Acquire<'c, Database = Postgres> + Send + 'a>(
    connection: Connection,
    tenant_id: &'a TenantId,
    project_id: &'a ProjectId,
    resource_type: &'a ResourceType,
    resource_id: &'a ResourceId,
) -> impl Future<
    Output = Result<Option<oxidized_fhir_model::r4::types::Resource>, OperationOutcomeError>,
> + Send
+ 'a {
    async move {
        let mut conn = connection.acquire().await.map_err(StoreError::SQLXError)?;
        let response = sqlx::query!(
            r#"SELECT resource as "resource: FHIRJson<Resource>" FROM resources WHERE tenant = $1 AND project = $2 AND id = $3 AND resource_type = $4 ORDER BY sequence DESC"#,
            tenant_id.as_ref(),
            project_id.as_ref(),
            resource_id.as_ref(),
            resource_type.as_str(),
        ).fetch_optional(&mut *conn).await.map_err(StoreError::from)?;

        Ok(response.map(|r| r.resource.0))
    }
}

fn history<'a, 'c, Connection: Acquire<'c, Database = Postgres> + Send + 'a>(
    connection: Connection,
    tenant_id: &'a TenantId,
    project_id: &'a ProjectId,
    history_request: HistoryRequest<'a>,
) -> impl Future<Output = Result<Vec<Resource>, OperationOutcomeError>> + Send + 'a {
    async move {
        let mut conn = connection.acquire().await.map_err(StoreError::from)?;
        match history_request {
            HistoryRequest::Instance(history_instance_request) => {
                let response = sqlx::query_as!(ReturnV,
                    r#"SELECT resource as "resource: FHIRJson<Resource>" FROM resources WHERE tenant = $1 AND project = $2 AND id = $3 AND resource_type = $4 ORDER BY sequence DESC LIMIT 100"#,
                        tenant_id.as_ref()  as &str,
                        project_id.as_ref() as &str,
                        history_instance_request.id.as_ref() as &str,
                        history_instance_request.resource_type.as_str() as &str
                    ).fetch_all(&mut *conn).await.map_err(StoreError::from)?;

                Ok(response.into_iter().map(|r| r.resource.0).collect())
            }
            HistoryRequest::Type(history_type_request) => {
                let response = sqlx::query_as!(ReturnV,
                    r#"SELECT resource as "resource: FHIRJson<Resource>" FROM resources WHERE tenant = $1 AND project = $2 AND resource_type = $3 ORDER BY sequence DESC LIMIT 100"#,
                        tenant_id.as_ref()  as &str,
                        project_id.as_ref() as &str,
                        history_type_request.resource_type.as_str() as &str
                    ).fetch_all(&mut *conn).await.map_err(StoreError::from)?;

                Ok(response.into_iter().map(|r| r.resource.0).collect())
            }
            HistoryRequest::System(_request) => {
                let response = sqlx::query_as!(ReturnV,
                    r#"SELECT resource as "resource: FHIRJson<Resource>" FROM resources WHERE tenant = $1 AND project = $2 ORDER BY sequence DESC LIMIT 100"#,
                        tenant_id.as_ref()  as &str,
                        project_id.as_ref() as &str
                    ).fetch_all(&mut *conn).await.map_err(StoreError::from)?;

                Ok(response.into_iter().map(|r| r.resource.0).collect())
            }
        }
    }
}

fn get_sequence<'a, 'c, Connection: Acquire<'c, Database = Postgres> + Send + 'a>(
    connection: Connection,
    tenant_id: &'a TenantId,
    cur_sequence: u64,
    count: Option<u64>,
) -> impl Future<Output = Result<Vec<ResourcePollingValue>, OperationOutcomeError>> + Send + 'a {
    async move {
        let mut conn = connection.acquire().await.map_err(StoreError::from)?;
        let result = sqlx::query_as!(
            ResourcePollingValue,
            r#"SELECT  id as "id: ResourceId", 
                       tenant as "tenant: TenantId", 
                       project as "project: ProjectId", 
                       version_id, 
                       resource_type as "resource_type: ResourceType", 
                       fhir_method as "fhir_method: FHIRMethod", 
                       sequence, 
                       resource as "resource: FHIRJson<Resource>" 
            FROM resources WHERE tenant = $1 AND sequence > $2 ORDER BY sequence LIMIT $3 "#,
            tenant_id.as_ref() as &str,
            cur_sequence as i64,
            count.unwrap_or(100) as i64
        )
        .fetch_all(&mut *conn)
        .await
        .map_err(StoreError::from)?;

        Ok(result)
    }
}
