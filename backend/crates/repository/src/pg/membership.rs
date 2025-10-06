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

fn read_membership<'a, 'c, Connection: Acquire<'c, Database = Postgres> + Send + 'a>(
    connection: Connection,
    tenant: &'a TenantId,
    project: &'a ProjectId,
    user_id: &'a str,
) -> impl Future<Output = Result<Membership, OperationOutcomeError>> + Send + 'a {
    async move {
        let mut conn = connection.acquire().await.map_err(StoreError::SQLXError)?;
        let membership = sqlx::query_as!(
            Membership,
            r#"
                SELECT tenant, project, user_id, role as "role: MembershipRole"
                FROM memberships
                WHERE tenant = $1 AND user_id = $2 AND project = $3
            "#,
            tenant.as_ref(),
            project.as_ref(),
            user_id
        )
        .fetch_one(&mut *conn)
        .await
        .map_err(StoreError::SQLXError)?;

        Ok(membership)
    }
}

fn update_membership<'a, 'c, Connection: Acquire<'c, Database = Postgres> + Send + 'a>(
    connection: Connection,
    tenant: &'a TenantId,
    project: &'a ProjectId,
    model: UpdateMembership,
) -> impl Future<Output = Result<Membership, OperationOutcomeError>> + Send + 'a {
    async move {
        let mut conn = connection.acquire().await.map_err(StoreError::SQLXError)?;
        let mut query_builder = QueryBuilder::new(
            r#"
                UPDATE memberships SET 
            "#,
        );

        let mut seperator = query_builder.separated(", ");

        seperator
            .push(" tenant = ")
            .push_bind_unseparated(tenant.as_ref())
            .push(" project = ")
            .push_bind_unseparated(project.as_ref())
            .push(" role = ")
            .push_bind_unseparated(model.role);

        query_builder.push(r#" RETURNING id, provider_id, email, role, method"#);

        let query = query_builder.build_query_as();

        let user = query
            .fetch_one(&mut *conn)
            .await
            .map_err(StoreError::SQLXError)?;

        Ok(user)
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
        match self {
            PGConnection::PgPool(pool) => {
                let res = read_membership(pool, tenant, project, id).await?;
                Ok(res)
            }
            PGConnection::PgTransaction(tx) => {
                let mut tx = tx.lock().await;
                let res = read_membership(&mut *tx, tenant, project, id).await?;
                Ok(res)
            }
        }
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
