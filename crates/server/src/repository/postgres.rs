use chrono::Utc;
use fhir_client::request::FHIRRequest;
use fhir_model::r4::{
    sqlx::FHIRJson,
    types::{Patient, Resource},
};
use sqlx::{
    Decode, Executor, FromRow, PgPool, Pool, Postgres, Row, ValueRef, error::BoxDynError,
    types::Json,
};
use sqlx_postgres::{PgPoolOptions, PgRow};

use crate::{
    SupportedFHIRVersions,
    repository::{FHIRMethod, FHIRRepository, InsertResourceRow},
};

pub struct PostgresSQL(sqlx::PgPool);
impl PostgresSQL {
    pub async fn new(pool: sqlx::PgPool) -> Result<Self, sqlx::Error> {
        Ok(PostgresSQL(pool))
    }
}

struct ReturnV {
    resource: FHIRJson<Resource>,
}

impl FHIRRepository for PostgresSQL {
    async fn insert(&self, row: &InsertResourceRow) -> Result<Resource, crate::ServerErrors> {
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
                &row.resource as &FHIRJson<Resource>,
                row.deleted,
                row.request_method,
                row.author_type,
                &row.fhir_method as &FHIRMethod,
            ).fetch_one(&self.0).await?;

        Ok(result.resource.0)
    }

    fn read_by_version_id(
        &self,
        tenant_id: super::TenantId,
        project_id: super::ProjectId,
        version_id: Vec<super::VersionId>,
    ) -> impl Future<Output = Result<Vec<fhir_model::r4::types::Resource>, crate::ServerErrors>> + Send
    {
        async { todo!() }
    }

    fn read_latest(
        &self,
        tenant_id: super::TenantId,
        project_id: super::ProjectId,
        resource_id: super::ResourceId,
    ) -> impl Future<Output = Result<Option<fhir_model::r4::types::Resource>, crate::ServerErrors>> + Send
    {
        async { todo!() }
    }

    fn history(
        &self,
        tenant_id: super::TenantId,
        project_id: super::ProjectId,
        resource_id: super::ResourceId,
    ) -> impl Future<Output = Result<Vec<fhir_model::r4::types::Resource>, crate::ServerErrors>> + Send
    {
        async { todo!() }
    }

    fn get_sequence(
        &self,
        tenant_id: super::TenantId,
        project_id: super::ProjectId,
        sequence_id: u64,
        count: Option<u64>,
    ) -> impl Future<Output = Result<Vec<fhir_model::r4::types::Resource>, crate::ServerErrors>> + Send
    {
        async { todo!() }
    }
}
