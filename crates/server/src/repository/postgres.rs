use oxidized_fhir_model::r4::{
    sqlx::{FHIRJson, FHIRJsonRef},
    types::{Resource, ResourceType},
};
use oxidized_fhir_operation_error::OperationOutcomeError;
use oxidized_fhir_operation_error::derive::OperationOutcomeError;
use sqlx::{Execute, Executor, Postgres, QueryBuilder, Row, query_builder::Separated};

use crate::{
    SupportedFHIRVersions,
    repository::{
        FHIRMethod, FHIRRepository, HistoryRequest, InsertResourceRow, ProjectId, ResourceId,
        TenantId, VersionId, utilities,
    },
};

pub struct FHIRPostgresRepository(sqlx::PgPool);
impl FHIRPostgresRepository {
    pub fn new(pool: sqlx::PgPool) -> Self {
        FHIRPostgresRepository(pool)
    }
}

#[derive(sqlx::FromRow, Debug)]
struct ReturnV {
    resource: FHIRJson<Resource>,
}

#[derive(OperationOutcomeError, Debug)]
pub enum StoreError {
    #[error(code = "invalid", diagnostic = "SQL Error occured.")]
    FailedInsert(#[from] sqlx::Error),
}

impl FHIRRepository for FHIRPostgresRepository {
    async fn insert<'a>(
        &self,
        row: &InsertResourceRow<'a>,
    ) -> Result<Resource, OperationOutcomeError> {
        let result = sqlx::query_as!(
                ReturnV,
                r#"INSERT INTO resources (tenant, project, author_id, fhir_version, resource, deleted, request_method, author_type, fhir_method) 
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) 
                RETURNING resource as "resource: FHIRJson<Resource>""#,
                row.tenant,
                row.project,
                row.author_id,
                // Useless cast so that macro has access to the type information.
                // Otherwise it will not compile on type check.
                &row.fhir_version as &SupportedFHIRVersions,
                &FHIRJsonRef(row.resource) as &FHIRJsonRef<'_ , Resource>,
                row.deleted,
                row.request_method,
                row.author_type,
                &row.fhir_method as &FHIRMethod,
            ).fetch_one(&self.0).await.map_err(StoreError::from)?;

        Ok(result.resource.0)
    }

    async fn read_by_version_ids(
        &self,
        tenant_id: &TenantId,
        project_id: &ProjectId,
        version_ids: Vec<VersionId<'_>>,
    ) -> Result<Vec<Resource>, OperationOutcomeError> {
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
        let response: Vec<ReturnV> = query.fetch_all(&self.0).await.map_err(StoreError::from)?;
        Ok(response.into_iter().map(|r| r.resource.0).collect())
    }

    async fn read_latest(
        &self,
        tenant_id: &TenantId,
        project_id: &ProjectId,
        resource_type: &ResourceType,
        resource_id: &ResourceId,
    ) -> Result<Option<oxidized_fhir_model::r4::types::Resource>, OperationOutcomeError> {
        let response = sqlx::query!(
            r#"SELECT resource as "resource: FHIRJson<Resource>" FROM resources WHERE tenant = $1 AND project = $2 AND id = $3 AND resource_type = $4 ORDER BY sequence DESC"#,
            tenant_id.as_ref(),
            project_id.as_ref(),
            resource_id.as_ref(),
            resource_type.as_str(),
        ).fetch_optional(&self.0).await.map_err(StoreError::from)?;

        Ok(response.map(|r| r.resource.0))
    }

    async fn history(
        &self,
        tenant_id: &TenantId,
        project_id: &ProjectId,
        history_request: HistoryRequest<'_>,
    ) -> Result<Vec<oxidized_fhir_model::r4::types::Resource>, OperationOutcomeError> {
        match history_request {
            HistoryRequest::Instance(history_instance_request) => {
                let response = sqlx::query_as!(ReturnV,
                    r#"SELECT resource as "resource: FHIRJson<Resource>" FROM resources WHERE tenant = $1 AND project = $2 AND id = $3 AND resource_type = $4 ORDER BY sequence DESC LIMIT 100"#,
                        tenant_id.as_ref()  as &str,
                        project_id.as_ref() as &str,
                        history_instance_request.id.as_ref() as &str,
                        history_instance_request.resource_type.as_str() as &str
                    ).fetch_all(&self.0).await.map_err(StoreError::from)?;

                Ok(response.into_iter().map(|r| r.resource.0).collect())
            }
            HistoryRequest::Type(history_type_request) => {
                let response = sqlx::query_as!(ReturnV,
                    r#"SELECT resource as "resource: FHIRJson<Resource>" FROM resources WHERE tenant = $1 AND project = $2 AND resource_type = $3 ORDER BY sequence DESC LIMIT 100"#,
                        tenant_id.as_ref()  as &str,
                        project_id.as_ref() as &str,                
                        history_type_request.resource_type.as_str() as &str
                    ).fetch_all(&self.0).await.map_err(StoreError::from)?;

                Ok(response.into_iter().map(|r| r.resource.0).collect())
            }
            HistoryRequest::System(request) => {
                let response = sqlx::query_as!(ReturnV,
                    r#"SELECT resource as "resource: FHIRJson<Resource>" FROM resources WHERE tenant = $1 AND project = $2 ORDER BY sequence DESC LIMIT 100"#,
                        tenant_id.as_ref()  as &str,
                        project_id.as_ref() as &str,                
                    ).fetch_all(&self.0).await.map_err(StoreError::from)?;

                Ok(response.into_iter().map(|r| r.resource.0).collect())
            },
        }
    }

    async fn get_sequence(
        &self,
        tenant_id: &TenantId,
        project_id: &ProjectId,
        sequence_id: u64,
        count: Option<u64>,
    ) -> Result<Vec<oxidized_fhir_model::r4::types::Resource>, OperationOutcomeError> {
        todo!()
    }
}
