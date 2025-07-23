use fhir_model::r4::types::Resource;
use sqlx::{Decode, Executor, FromRow, PgPool, Pool, Postgres, Row, ValueRef, error::BoxDynError};
use sqlx_postgres::{PgPoolOptions, PgRow};

use crate::repository::FHIRRepository;

pub struct PostgresSQL(sqlx::PgPool);
impl PostgresSQL {
    pub async fn new(pool: sqlx::PgPool) -> Result<Self, sqlx::Error> {
        Ok(PostgresSQL(pool))
    }
}

impl FHIRRepository for PostgresSQL {
    fn insert(
        &self,
        tenant_id: super::TenantId,
        project_id: super::ProjectId,
        user_id: super::UserId,
        resource: fhir_model::r4::types::Resource,
    ) -> impl Future<Output = Result<fhir_model::r4::types::Resource, crate::ServerErrors>> + Send
    {
        async { todo!() }
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
