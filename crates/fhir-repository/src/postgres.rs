use std::{marker::PhantomData, pin::Pin, sync::Arc};

use crate::{
    Author, FHIRMethod, FHIRRepository, FHIRTransaction, HistoryRequest, ProjectId, ResourceId,
    ResourcePollingValue, SupportedFHIRVersions, TenantId, VersionId, utilities,
};
use oxidized_fhir_model::r4::{
    sqlx::{FHIRJson, FHIRJsonRef},
    types::{Patient, Resource, ResourceType},
};
use oxidized_fhir_operation_error::OperationOutcomeError;
use oxidized_fhir_operation_error::derive::OperationOutcomeError;
use sqlx::{Acquire, Postgres, QueryBuilder};
use sqlx::{Connection, Transaction};

#[derive(sqlx::FromRow, Debug)]
struct ReturnV {
    resource: FHIRJson<Resource>,
}

#[derive(OperationOutcomeError, Debug)]
pub enum StoreError {
    #[error(code = "invalid", diagnostic = "SQL Error occured.")]
    SQLXError(#[from] sqlx::Error),
}

async fn create<'a, Acquirer: Acquire<'a, Database = Postgres> + Send + Sync>(
    acquirer: Acquirer,
    tenant: &TenantId,
    project: &ProjectId,
    author: &Author,
    fhir_version: &SupportedFHIRVersions,
    resource: &mut Resource,
) -> Result<Resource, OperationOutcomeError> {
    utilities::set_resource_id(resource, None)?;
    utilities::set_version_id(resource)?;

    let mut conn = acquirer.acquire().await.map_err(StoreError::SQLXError)?;
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

async fn update<'a, Acquirer: Acquire<'a, Database = Postgres> + Send + Sync>(
    acquirer: Acquirer,
    tenant: &TenantId,
    project: &ProjectId,
    author: &Author,
    fhir_version: &SupportedFHIRVersions,
    resource: &mut Resource,
    id: &str,
) -> Result<Resource, OperationOutcomeError> {
    utilities::set_resource_id(resource, Some(id.to_string()))?;
    utilities::set_version_id(resource)?;

    let mut conn = acquirer.acquire().await.map_err(StoreError::SQLXError)?;

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
                "PUT",
                author.kind,
                &FHIRMethod::Update as &FHIRMethod,
            ).fetch_one(&mut *conn).await.map_err(StoreError::from)?;

    Ok(result.resource.0)
}

async fn read_by_version_ids<'a, Acquirer: Acquire<'a, Database = Postgres> + Send + Sync>(
    acquirer: Acquirer,
    tenant_id: &TenantId,
    project_id: &ProjectId,
    version_ids: Vec<VersionId<'_>>,
) -> Result<Vec<Resource>, OperationOutcomeError> {
    let mut conn = acquirer.acquire().await.map_err(StoreError::SQLXError)?;
    let mut query_builder: QueryBuilder<Postgres> =
        QueryBuilder::new(r#"SELECT resource FROM resources WHERE tenant = "#);
    query_builder.push_bind(tenant_id.as_ref());
    query_builder.push(" AND project =");
    query_builder.push_bind(project_id.as_ref());
    query_builder.push(" AND version_id in (");

    let mut separated = query_builder.separated(", ");
    for version_id in version_ids.iter() {
        separated.push_bind(version_id.as_ref());
    }
    separated.push_unseparated(")");

    let query = query_builder.build_query_as();
    let response: Vec<ReturnV> = query
        .fetch_all(&mut *conn)
        .await
        .map_err(StoreError::from)?;
    Ok(response.into_iter().map(|r| r.resource.0).collect())
}

async fn read_latest<'a, Acquirer: Acquire<'a, Database = Postgres> + Send + Sync>(
    acquirer: Acquirer,
    tenant_id: &TenantId,
    project_id: &ProjectId,
    resource_type: &ResourceType,
    resource_id: &ResourceId,
) -> Result<Option<oxidized_fhir_model::r4::types::Resource>, OperationOutcomeError> {
    let mut conn = acquirer.acquire().await.map_err(StoreError::SQLXError)?;
    let response = sqlx::query!(
            r#"SELECT resource as "resource: FHIRJson<Resource>" FROM resources WHERE tenant = $1 AND project = $2 AND id = $3 AND resource_type = $4 ORDER BY sequence DESC"#,
            tenant_id.as_ref(),
            project_id.as_ref(),
            resource_id.as_ref(),
            resource_type.as_str(),
        ).fetch_optional(&mut *conn).await.map_err(StoreError::from)?;

    Ok(response.map(|r| r.resource.0))
}

async fn history<'a, Acquirer: Acquire<'a, Database = Postgres> + Send>(
    acquirer: Acquirer,
    tenant_id: &TenantId,
    project_id: &ProjectId,
    history_request: HistoryRequest<'_>,
) -> Result<Vec<oxidized_fhir_model::r4::types::Resource>, sqlx::Error> {
    let mut conn = acquirer.acquire().await?;
    match history_request {
        HistoryRequest::Instance(history_instance_request) => {
            let response = sqlx::query_as!(ReturnV,
                    r#"SELECT resource as "resource: FHIRJson<Resource>" FROM resources WHERE tenant = $1 AND project = $2 AND id = $3 AND resource_type = $4 ORDER BY sequence DESC LIMIT 100"#,
                        tenant_id.as_ref()  as &str,
                        project_id.as_ref() as &str,
                        history_instance_request.id.as_ref() as &str,
                        history_instance_request.resource_type.as_str() as &str
                    ).fetch_all(&mut *conn).await?;

            Ok(response.into_iter().map(|r| r.resource.0).collect())
        }
        HistoryRequest::Type(history_type_request) => {
            let response = sqlx::query_as!(ReturnV,
                    r#"SELECT resource as "resource: FHIRJson<Resource>" FROM resources WHERE tenant = $1 AND project = $2 AND resource_type = $3 ORDER BY sequence DESC LIMIT 100"#,
                        tenant_id.as_ref()  as &str,
                        project_id.as_ref() as &str,
                        history_type_request.resource_type.as_str() as &str
                    ).fetch_all(&mut *conn).await?;

            Ok(response.into_iter().map(|r| r.resource.0).collect())
        }
        HistoryRequest::System(_request) => {
            let response = sqlx::query_as!(ReturnV,
                    r#"SELECT resource as "resource: FHIRJson<Resource>" FROM resources WHERE tenant = $1 AND project = $2 ORDER BY sequence DESC LIMIT 100"#,
                        tenant_id.as_ref()  as &str,
                        project_id.as_ref() as &str
                    ).fetch_all(&mut *conn).await?;

            Ok(response.into_iter().map(|r| r.resource.0).collect())
        }
    }
}

fn get_sequence<'a, A>(
    acquirer: A,
    tenant_id: &TenantId,
    _project_id: &ProjectId,
    cur_sequence: u64,
    count: Option<u64>,
) -> impl Future<Output = Result<Vec<ResourcePollingValue>, sqlx::Error>> + Send
where
    A: Acquire<'a, Database = Postgres> + Send,
{
    async move {
        let mut conn = acquirer.acquire().await?;
        let result = sqlx::query_as!(
            ResourcePollingValue,
            r#"SELECT  id, tenant, project, version_id, resource_type, fhir_method as "fhir_method: FHIRMethod", sequence, resource as "resource: FHIRJson<Resource>" FROM resources WHERE tenant = $1 AND sequence > $2 ORDER BY sequence LIMIT $3 "#,
            tenant_id.as_ref() as &str,
            cur_sequence as i64,
            count.unwrap_or(100) as i64
        )
        .fetch_all(&mut *conn)
        .await?;

        Ok(result)
    }
}

// async fn test(conn: &mut sqlx::PgConnection) -> Result<(), OperationOutcomeError> {
//     let k: Result<(), sqlx::Error> = conn
//         .transaction(|transaction| {
//             Box::pin(async move {
//                 let k = get_sequence(
//                     transaction,
//                     &TenantId::new("tenant".to_string()),
//                     &ProjectId::new("project".to_string()),
//                     0,
//                     Some(10),
//                 )
//                 .await;

//                 Ok(())
//             })
//         })
//         .await;

//     Ok(())
// }

pub struct FHIRPostgresRepositoryPool(sqlx::Pool<Postgres>);

impl FHIRPostgresRepositoryPool {
    pub fn new(pool: sqlx::Pool<Postgres>) -> Self {
        FHIRPostgresRepositoryPool(pool)
    }
}

impl FHIRRepository for FHIRPostgresRepositoryPool {
    async fn create(
        &self,
        tenant: &TenantId,
        project: &ProjectId,
        author: &Author,
        fhir_version: &SupportedFHIRVersions,
        resource: &mut Resource,
    ) -> Result<Resource, OperationOutcomeError> {
        let res = create(&self.0, tenant, project, author, fhir_version, resource).await?;
        Ok(res)
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
        let res = update(&self.0, tenant, project, author, fhir_version, resource, id).await?;

        Ok(res)
    }

    async fn read_by_version_ids(
        &self,
        tenant_id: &TenantId,
        project_id: &ProjectId,
        version_ids: Vec<VersionId<'_>>,
    ) -> Result<Vec<Resource>, OperationOutcomeError> {
        let res = read_by_version_ids(&self.0, tenant_id, project_id, version_ids).await?;

        Ok(res)
    }

    async fn read_latest(
        &self,
        tenant_id: &TenantId,
        project_id: &ProjectId,
        resource_type: &ResourceType,
        resource_id: &ResourceId,
    ) -> Result<Option<oxidized_fhir_model::r4::types::Resource>, OperationOutcomeError> {
        let res = read_latest(&self.0, tenant_id, project_id, resource_type, resource_id).await?;

        Ok(res)
    }

    async fn history(
        &self,
        tenant_id: &TenantId,
        project_id: &ProjectId,
        request: HistoryRequest<'_>,
    ) -> Result<Vec<Resource>, OperationOutcomeError> {
        let res = history(&self.0, tenant_id, project_id, request)
            .await
            .map_err(StoreError::from)?;

        Ok(res)
    }

    async fn get_sequence(
        &self,
        tenant_id: &TenantId,
        project_id: &ProjectId,
        sequence_id: u64,
        count: Option<u64>,
    ) -> Result<Vec<ResourcePollingValue>, OperationOutcomeError> {
        let result = get_sequence(&self.0, tenant_id, project_id, sequence_id, count)
            .await
            .map_err(StoreError::from)?;
        Ok(result)
    }
}

async fn tester(transaction: &mut Transaction<'_, Postgres>) -> Result<(), OperationOutcomeError> {
    let mut patient = Resource::Patient(Patient::default());
    SQLImplementation::create(
        transaction,
        &TenantId::new("tenant".to_string()),
        &ProjectId::new("project".to_string()),
        &Author {
            id: "author_id".to_string(),
            kind: "author_kind".to_string(),
        },
        &SupportedFHIRVersions::R4,
        &mut patient,
    )
    .await
    .unwrap();
    Ok(())
}

async fn my_test<Connection, T: FHIRTransaction<Connection>>(
    connection: Connection,
    imple: T,
) -> Result<(), OperationOutcomeError> {
    let mut patient = Resource::Patient(Patient::default());
    T::create(
        connection,
        &TenantId::new("tenant".to_string()),
        &ProjectId::new("project".to_string()),
        &Author {
            id: "author_id".to_string(),
            kind: "author_kind".to_string(),
        },
        &SupportedFHIRVersions::R4,
        &mut patient,
    )
    .await?;
    Ok(())
}

pub struct SQLImplementation<'a> {
    _marker: &'a PhantomData<()>,
}
impl<'a, Connection> FHIRTransaction<Connection> for SQLImplementation<'a>
where
    Connection: Acquire<'a, Database = Postgres> + Send + Sync,
{
    async fn create(
        k: Connection,
        tenant: &TenantId,
        project: &ProjectId,
        author: &Author,
        fhir_version: &SupportedFHIRVersions,
        resource: &mut Resource,
    ) -> Result<Resource, OperationOutcomeError> {
        let res = create(k, tenant, project, author, fhir_version, resource).await?;

        // let z = self.0;
        Ok(res)
    }

    async fn update(
        k: Connection,
        tenant: &TenantId,
        project: &ProjectId,
        author: &Author,
        fhir_version: &SupportedFHIRVersions,
        resource: &mut Resource,
        id: &str,
    ) -> Result<Resource, OperationOutcomeError> {
        let res = update(k, tenant, project, author, fhir_version, resource, id).await?;

        Ok(res)
    }

    async fn read_by_version_ids(
        k: Connection,
        tenant_id: &TenantId,
        project_id: &ProjectId,
        version_ids: Vec<VersionId<'_>>,
    ) -> Result<Vec<Resource>, OperationOutcomeError> {
        let res = read_by_version_ids(k, tenant_id, project_id, version_ids).await?;

        Ok(res)
    }

    async fn read_latest(
        k: Connection,
        tenant_id: &TenantId,
        project_id: &ProjectId,
        resource_type: &ResourceType,
        resource_id: &ResourceId,
    ) -> Result<Option<oxidized_fhir_model::r4::types::Resource>, OperationOutcomeError> {
        let res = read_latest(k, tenant_id, project_id, resource_type, resource_id).await?;

        Ok(res)
    }

    async fn history(
        k: Connection,
        tenant_id: &TenantId,
        project_id: &ProjectId,
        request: HistoryRequest<'_>,
    ) -> Result<Vec<Resource>, OperationOutcomeError> {
        let res = history(k, tenant_id, project_id, request)
            .await
            .map_err(StoreError::from)?;

        Ok(res)
    }

    async fn get_sequence(
        k: Connection,
        tenant_id: &TenantId,
        project_id: &ProjectId,
        sequence_id: u64,
        count: Option<u64>,
    ) -> Result<Vec<ResourcePollingValue>, OperationOutcomeError> {
        let result = get_sequence(k, tenant_id, project_id, sequence_id, count)
            .await
            .map_err(StoreError::from)?;
        Ok(result)
    }
}
