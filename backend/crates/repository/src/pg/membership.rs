use crate::{
    admin::ProjectAuthAdmin,
    pg::{PGConnection, StoreError},
    types::{
        ProjectId, TenantId,
        membership::{
            CreateMembership, Membership, MembershipRole, MembershipSearchClaims, UpdateMembership,
        },
    },
};
use oxidized_fhir_operation_error::OperationOutcomeError;
use sqlx::{Acquire, Postgres, QueryBuilder};

fn create_membership<'a, 'c, Connection: Acquire<'c, Database = Postgres> + Send + 'a>(
    connection: Connection,
    tenant: &'a TenantId,
    project: &'a ProjectId,
    membership: CreateMembership,
) -> impl Future<Output = Result<Membership, OperationOutcomeError>> + Send + 'a {
    async move {
        let mut conn = connection.acquire().await.map_err(StoreError::SQLXError)?;
        let mut query_builder = QueryBuilder::new(
            r#"
                INSERT INTO users(tenant, project, user_id, role) VALUES (
            "#,
        );

        let mut seperator = query_builder.separated(", ");

        seperator
            .push_bind(tenant.as_ref())
            .push_bind(project.as_ref())
            .push_bind(&membership.user_id)
            .push_bind(membership.role as MembershipRole);

        query_builder.push(r#") RETURNING tenant, project, user_id, role"#);

        let query = query_builder.build_query_as();

        let membership = query
            .fetch_one(&mut *conn)
            .await
            .map_err(StoreError::SQLXError)?;

        Ok(membership)
    }
}

impl ProjectAuthAdmin<CreateMembership, Membership, MembershipSearchClaims, UpdateMembership>
    for PGConnection
{
    async fn create(
        &self,
        tenant: &crate::types::TenantId,
        project: &crate::types::ProjectId,
        new_membership: CreateMembership,
    ) -> Result<Membership, OperationOutcomeError> {
        match self {
            PGConnection::PgPool(pool) => {
                let res = create_membership(pool, tenant, project, new_membership).await?;
                Ok(res)
            }
            PGConnection::PgTransaction(tx) => {
                let mut tx = tx.lock().await;
                let res = create_membership(&mut *tx, tenant, project, new_membership).await?;
                Ok(res)
            }
        }
    }

    async fn read(
        &self,
        tenant: &crate::types::TenantId,
        project: &crate::types::ProjectId,
        id: &str,
    ) -> Result<Membership, OperationOutcomeError> {
        todo!()
    }

    async fn update(
        &self,
        tenant: &crate::types::TenantId,
        project: &crate::types::ProjectId,
        model: UpdateMembership,
    ) -> Result<Membership, OperationOutcomeError> {
        todo!()
    }

    async fn delete(
        &self,
        tenant: &crate::types::TenantId,
        project: &crate::types::ProjectId,
        id: &str,
    ) -> Result<Membership, OperationOutcomeError> {
        todo!()
    }

    async fn search(
        &self,
        tenant: &crate::types::TenantId,
        project: &crate::types::ProjectId,
        clauses: &MembershipSearchClaims,
    ) -> Result<Vec<Membership>, OperationOutcomeError> {
        todo!()
    }
}
