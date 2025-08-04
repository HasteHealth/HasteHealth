use oxidized_fhir_model::r4::{
    sqlx::{FHIRJson, FHIRJsonRef},
    types::Resource,
};
use oxidized_fhir_operation_error::OperationOutcomeError;
use oxidized_fhir_operation_error::derive::OperationOutcomeError;
use sqlx::{Executor, Postgres, QueryBuilder, Row};

use crate::{
    SupportedFHIRVersions,
    repository::{
        FHIRMethod, FHIRRepository, InsertResourceRow, ProjectId, ResourceId, TenantId, VersionId,
        utilities,
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
    #[error(
        code = "invalid",
        diagnostic = "An error occured while retrieving a resource."
    )]
    FailedInsert(#[from] sqlx::Error),
}

impl FHIRRepository for FHIRPostgresRepository {
    async fn insert<'a>(
        &self,
        row: &InsertResourceRow<'a>,
    ) -> Result<Resource, OperationOutcomeError> {
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
            "INSERT INTO resources (tenant, project, author_id, fhir_version, resource, deleted, request_method, author_type, fhir_method) VALUES (",
        );
        query_builder
            .push_bind(&row.tenant)
            .push_bind(&row.project)
            .push_bind(&row.author_id)
            .push_bind(&row.fhir_version)
            .push_bind(FHIRJsonRef(row.resource))
            .push_bind(row.deleted)
            .push_bind(&row.request_method)
            .push_bind(&row.author_type)
            .push_bind(&row.fhir_method as &FHIRMethod);

        query_builder.push(r#") RETURNING resource as "resource: FHIRJson<Resource>""#);

        let query = query_builder.build_query_as();

        let result: ReturnV = query.fetch_one(&self.0).await.map_err(StoreError::from)?;

        Ok(result.resource.0)
    }

    async fn read_by_version_ids(
        &self,
        tenant_id: &TenantId,
        project_id: &ProjectId,
        version_id: Vec<VersionId>,
    ) -> Result<Vec<Resource>, OperationOutcomeError> {
        todo!();
    }

    async fn read_latest(
        &self,
        tenant_id: &TenantId,
        project_id: &ProjectId,
        resource_id: &ResourceId,
    ) -> Result<Option<oxidized_fhir_model::r4::types::Resource>, OperationOutcomeError> {
        let response = sqlx::query!(
            r#"SELECT resource as "resource: FHIRJson<Resource>" FROM resources WHERE tenant = $1 AND project = $2 AND id = $3 ORDER BY sequence DESC"#,
            tenant_id.as_ref(),
            project_id.as_ref(),
            resource_id.as_ref(),
        ).fetch_optional(&self.0).await.map_err(StoreError::from)?;

        Ok(response.map(|r| r.resource.0))
    }

    async fn history(
        &self,
        tenant_id: &TenantId,
        project_id: &ProjectId,
        resource_id: &ResourceId,
    ) -> Result<Vec<oxidized_fhir_model::r4::types::Resource>, OperationOutcomeError> {
        todo!();
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
