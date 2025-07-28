use fhir_model::r4::{
    sqlx::{FHIRJson, FHIRJsonRef},
    types::Resource,
};
use fhir_operation_error::OperationOutcomeError;
use fhir_operation_error::derive::OperationOutcomeError;
use sqlx::{Executor, Row};

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

struct ReturnV {
    resource: FHIRJson<Resource>,
}

#[derive(OperationOutcomeError, Debug)]
pub enum StoreError {
    #[error(code = "invalid", diagnostic = "Could not insert resource.")]
    FailedInsert(#[from] sqlx::Error),
}

impl FHIRRepository for FHIRPostgresRepository {
    async fn insert<'a>(
        &self,
        row: &mut InsertResourceRow<'a>,
    ) -> Result<Resource, OperationOutcomeError> {
        utilities::set_resource_id(&mut row.resource)?;
        utilities::set_version_id(&mut row.resource)?;

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
        version_id: Vec<VersionId>,
    ) -> Result<Vec<Resource>, OperationOutcomeError> {
        todo!();
    }

    async fn read_latest(
        &self,
        tenant_id: &TenantId,
        project_id: &ProjectId,
        resource_id: &ResourceId,
    ) -> Result<fhir_model::r4::types::Resource, OperationOutcomeError> {
        let response = sqlx::query!(
            r#"SELECT resource as "resource: FHIRJson<Resource>" FROM resources WHERE tenant = $1 AND project = $2 AND id = $3 ORDER BY sequence DESC"#,
            tenant_id.as_ref(),
            project_id.as_ref(),
            resource_id.as_ref(),
        ).fetch_one(&self.0).await.map_err(StoreError::from)?;
        panic!();
    }

    async fn history(
        &self,
        tenant_id: &TenantId,
        project_id: &ProjectId,
        resource_id: &ResourceId,
    ) -> Result<Vec<fhir_model::r4::types::Resource>, OperationOutcomeError> {
        todo!();
    }

    async fn get_sequence(
        &self,
        tenant_id: &TenantId,
        project_id: &ProjectId,
        sequence_id: u64,
        count: Option<u64>,
    ) -> Result<Vec<fhir_model::r4::types::Resource>, OperationOutcomeError> {
        todo!()
    }
}
