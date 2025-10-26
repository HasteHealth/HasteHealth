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
                INSERT INTO memberships(tenant, project, user_id, role) VALUES (
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
) -> impl Future<Output = Result<Option<Membership>, OperationOutcomeError>> + Send + 'a {
    async move {
        let mut conn = connection.acquire().await.map_err(StoreError::SQLXError)?;
        let membership = sqlx::query_as!(
            Membership,
            r#"
                SELECT tenant as "tenant: TenantId", project as "project: ProjectId", user_id, role as "role: MembershipRole"
                FROM memberships
                WHERE tenant = $1 AND project = $2 AND user_id = $3
            "#,
            tenant.as_ref(),
            project.as_ref(),
            user_id
        )
        .fetch_optional(&mut *conn)
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

        let mut set_statements = query_builder.separated(", ");

        set_statements
            .push(" role = ")
            .push_bind_unseparated(model.role);

        query_builder.push(" WHERE ");

        let mut where_statements = query_builder.separated(" AND ");
        where_statements
            .push(" tenant = ")
            .push_bind_unseparated(tenant.as_ref())
            .push(" project = ")
            .push_bind_unseparated(project.as_ref())
            .push(" user_id = ")
            .push_bind_unseparated(&model.user_id);

        query_builder.push(r#" RETURNING id, provider_id, email, role, method"#);

        let query = query_builder.build_query_as();

        let membership = query
            .fetch_one(&mut *conn)
            .await
            .map_err(StoreError::SQLXError)?;

        Ok(membership)
    }
}

fn delete_membership<'a, 'c, Connection: Acquire<'c, Database = Postgres> + Send + 'a>(
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
                DELETE FROM memberships
                WHERE tenant = $1 AND project = $2 AND user_id = $3
                RETURNING user_id, tenant as "tenant: TenantId", project as "project: ProjectId", role as "role: MembershipRole"
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

fn search_memberships<'a, 'c, Connection: Acquire<'c, Database = Postgres> + Send + 'a>(
    connection: Connection,
    tenant: &'a TenantId,
    project: &'a ProjectId,
    clauses: &'a MembershipSearchClaims,
) -> impl Future<Output = Result<Vec<Membership>, OperationOutcomeError>> + Send + 'a {
    async move {
        let mut conn = connection.acquire().await.map_err(StoreError::SQLXError)?;

        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
            r#"SELECT user_id, tenant, project, role as "role: MembershipRole" FROM memberships WHERE  "#,
        );

        let mut seperator = query_builder.separated(" AND ");
        seperator
            .push(" tenant = ")
            .push_bind_unseparated(tenant.as_ref())
            .push(" project = ")
            .push_bind_unseparated(project.as_ref());

        if let Some(user_id) = clauses.user_id.as_ref() {
            seperator
                .push(" user_id = ")
                .push_bind_unseparated(user_id.as_ref());
        }

        if let Some(role) = clauses.role.as_ref() {
            seperator.push(" role = ").push_bind_unseparated(role);
        }

        let query = query_builder.build_query_as();

        let memberships: Vec<Membership> = query
            .fetch_all(&mut *conn)
            .await
            .map_err(StoreError::from)?;

        Ok(memberships)
    }
}

impl<Key: AsRef<str> + Send + Sync>
    ProjectAuthAdmin<CreateMembership, Membership, MembershipSearchClaims, UpdateMembership, Key>
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
        id: &Key,
    ) -> Result<Option<Membership>, OperationOutcomeError> {
        match self {
            PGConnection::PgPool(pool) => {
                let res = read_membership(pool, tenant, project, id.as_ref()).await?;
                Ok(res)
            }
            PGConnection::PgTransaction(tx) => {
                let mut tx = tx.lock().await;
                let res = read_membership(&mut *tx, tenant, project, id.as_ref()).await?;
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
        match self {
            PGConnection::PgPool(pool) => {
                let res = update_membership(pool, tenant, project, model).await?;
                Ok(res)
            }
            PGConnection::PgTransaction(tx) => {
                let mut tx = tx.lock().await;
                let res = update_membership(&mut *tx, tenant, project, model).await?;
                Ok(res)
            }
        }
    }

    async fn delete(
        &self,
        tenant: &crate::types::TenantId,
        project: &crate::types::ProjectId,
        id: &Key,
    ) -> Result<Membership, OperationOutcomeError> {
        match self {
            PGConnection::PgPool(pool) => {
                let res = delete_membership(pool, tenant, project, id.as_ref()).await?;
                Ok(res)
            }
            PGConnection::PgTransaction(tx) => {
                let mut tx = tx.lock().await;
                let res = delete_membership(&mut *tx, tenant, project, id.as_ref()).await?;
                Ok(res)
            }
        }
    }

    async fn search(
        &self,
        tenant: &crate::types::TenantId,
        project: &crate::types::ProjectId,
        clauses: &MembershipSearchClaims,
    ) -> Result<Vec<Membership>, OperationOutcomeError> {
        match self {
            PGConnection::PgPool(pool) => {
                let res = search_memberships(pool, tenant, project, clauses).await?;
                Ok(res)
            }
            PGConnection::PgTransaction(tx) => {
                let mut tx = tx.lock().await;
                let res = search_memberships(&mut *tx, tenant, project, clauses).await?;
                Ok(res)
            }
        }
    }
}
