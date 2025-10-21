use crate::{
    admin::ProjectAuthAdmin,
    pg::{PGConnection, StoreError},
    types::{
        ProjectId, TenantId,
        scope::{CreateScope, Scope, ScopeKey, ScopeSearchClaims, UpdateScope},
        scopes::Scopes,
    },
};
use oxidized_fhir_operation_error::OperationOutcomeError;
use sqlx::{Acquire, Postgres, QueryBuilder};

fn create_scope<'a, 'c, Connection: Acquire<'c, Database = Postgres> + Send + 'a>(
    connection: Connection,
    tenant: &'a TenantId,
    project: &'a ProjectId,
    scope: CreateScope,
) -> impl Future<Output = Result<Scope, OperationOutcomeError>> + Send + 'a {
    async move {
        let mut conn = connection.acquire().await.map_err(StoreError::SQLXError)?;
        let scope = sqlx::query_as!(
            Scope,
            r#"INSERT INTO authorization_scopes(tenant, project, client, user_, scope) VALUES ($1, $2, $3, $4, $5) ON CONFLICT (tenant, project, client, user_) DO UPDATE set scope = $5  RETURNING  client, user_, scope"#,
            tenant.as_ref(),
            project.as_ref(),
            &scope.client.as_ref(),
            &scope.user_.as_ref(),
            &scope.scope as &Scopes,
        ).fetch_one(&mut *conn).await.map_err(StoreError::SQLXError)?;

        Ok(scope)
    }
}

fn update_scope<'a, 'c, Connection: Acquire<'c, Database = Postgres> + Send + 'a>(
    connection: Connection,
    tenant: &'a TenantId,
    project: &'a ProjectId,
    model: UpdateScope,
) -> impl Future<Output = Result<Scope, OperationOutcomeError>> + Send + 'a {
    async move {
        let mut conn = connection.acquire().await.map_err(StoreError::SQLXError)?;
        let mut query_builder = QueryBuilder::new(
            r#"
                UPDATE authorization_scopes SET 
            "#,
        );

        let mut set_statements = query_builder.separated(", ");

        set_statements
            .push(" scope = ")
            .push_bind_unseparated(model.scope);

        query_builder.push(" WHERE ");

        let mut where_statements = query_builder.separated(" AND ");
        where_statements
            .push(" tenant = ")
            .push_bind_unseparated(tenant.as_ref())
            .push(" project = ")
            .push_bind_unseparated(project.as_ref())
            .push(" client = ")
            .push_bind_unseparated(model.client.as_ref())
            .push(" user = ")
            .push_bind_unseparated(model.user_.as_ref());

        query_builder.push(r#" RETURNING client, user_ , scope"#);

        let query = query_builder.build_query_as();

        let scope = query
            .fetch_one(&mut *conn)
            .await
            .map_err(StoreError::SQLXError)?;

        Ok(scope)
    }
}

fn read_scope<'a, 'c, Connection: Acquire<'c, Database = Postgres> + Send + 'a>(
    connection: Connection,
    tenant: &'a TenantId,
    project: &'a ProjectId,
    id: &'a ScopeKey,
) -> impl Future<Output = Result<Option<Scope>, OperationOutcomeError>> + Send + 'a {
    async move {
        let mut conn = connection.acquire().await.map_err(StoreError::SQLXError)?;
        let scope = sqlx::query_as!(
            Scope,
            r#"
                SELECT user_, client, scope
                FROM authorization_scopes
                WHERE tenant = $1 AND project = $2 AND client = $3 and user_ = $4
            "#,
            tenant.as_ref(),
            project.as_ref(),
            String::from(id.0.clone()),
            String::from(id.1.clone()),
        )
        .fetch_optional(&mut *conn)
        .await
        .map_err(StoreError::SQLXError)?;

        Ok(scope)
    }
}

fn delete_scope<'a, 'c, Connection: Acquire<'c, Database = Postgres> + Send + 'a>(
    connection: Connection,
    tenant: &'a TenantId,
    project: &'a ProjectId,
    key: &'a ScopeKey,
) -> impl Future<Output = Result<Scope, OperationOutcomeError>> + Send + 'a {
    async move {
        let mut conn = connection.acquire().await.map_err(StoreError::SQLXError)?;
        let scope = sqlx::query_as!(
            Scope,
            r#"
                DELETE FROM authorization_scopes
                WHERE tenant = $1 AND project = $2 AND client = $3 AND user_ = $4
                RETURNING user_, client, scope
            "#,
            tenant.as_ref(),
            project.as_ref(),
            key.0.as_ref(),
            key.1.as_ref(),
        )
        .fetch_one(&mut *conn)
        .await
        .map_err(StoreError::SQLXError)?;

        Ok(scope)
    }
}

fn search_scopes<'a, 'c, Connection: Acquire<'c, Database = Postgres> + Send + 'a>(
    connection: Connection,
    tenant: &'a TenantId,
    project: &'a ProjectId,
    clauses: &'a ScopeSearchClaims,
) -> impl Future<Output = Result<Vec<Scope>, OperationOutcomeError>> + Send + 'a {
    async move {
        let mut conn = connection.acquire().await.map_err(StoreError::SQLXError)?;

        let mut query_builder: QueryBuilder<Postgres> =
            QueryBuilder::new(r#"SELECT user_, client, scope FROM authorization_scopes WHERE  "#);

        let mut seperator = query_builder.separated(" AND ");
        seperator
            .push(" tenant = ")
            .push_bind_unseparated(tenant.as_ref())
            .push(" project = ")
            .push_bind_unseparated(project.as_ref());

        if let Some(user_id) = clauses.user_.as_ref() {
            seperator
                .push(" user_ = ")
                .push_bind_unseparated(user_id.as_ref());
        }

        if let Some(client) = clauses.client.as_ref() {
            seperator
                .push(" client = ")
                .push_bind_unseparated(client.as_ref());
        }

        let query = query_builder.build_query_as();

        let scopes: Vec<Scope> = query
            .fetch_all(&mut *conn)
            .await
            .map_err(StoreError::from)?;

        Ok(scopes)
    }
}

impl ProjectAuthAdmin<CreateScope, Scope, ScopeSearchClaims, UpdateScope, ScopeKey>
    for PGConnection
{
    async fn create(
        &self,
        tenant: &crate::types::TenantId,
        project: &crate::types::ProjectId,
        new_scope: CreateScope,
    ) -> Result<Scope, OperationOutcomeError> {
        match self {
            PGConnection::PgPool(pool) => {
                let res = create_scope(pool, tenant, project, new_scope).await?;
                Ok(res)
            }
            PGConnection::PgTransaction(tx) => {
                let mut tx = tx.lock().await;
                let res = create_scope(&mut *tx, tenant, project, new_scope).await?;
                Ok(res)
            }
        }
    }

    async fn read(
        &self,
        tenant: &crate::types::TenantId,
        project: &crate::types::ProjectId,
        key: &ScopeKey,
    ) -> Result<Option<Scope>, OperationOutcomeError> {
        match self {
            PGConnection::PgPool(pool) => {
                let res = read_scope(pool, tenant, project, key).await?;
                Ok(res)
            }
            PGConnection::PgTransaction(tx) => {
                let mut tx = tx.lock().await;
                let res = read_scope(&mut *tx, tenant, project, key).await?;
                Ok(res)
            }
        }
    }

    async fn update(
        &self,
        tenant: &crate::types::TenantId,
        project: &crate::types::ProjectId,
        model: UpdateScope,
    ) -> Result<Scope, OperationOutcomeError> {
        match self {
            PGConnection::PgPool(pool) => {
                let res = update_scope(pool, tenant, project, model).await?;
                Ok(res)
            }
            PGConnection::PgTransaction(tx) => {
                let mut tx = tx.lock().await;
                let res = update_scope(&mut *tx, tenant, project, model).await?;
                Ok(res)
            }
        }
    }

    async fn delete(
        &self,
        tenant: &crate::types::TenantId,
        project: &crate::types::ProjectId,
        key: &ScopeKey,
    ) -> Result<Scope, OperationOutcomeError> {
        match self {
            PGConnection::PgPool(pool) => {
                let res = delete_scope(pool, tenant, project, key).await?;
                Ok(res)
            }
            PGConnection::PgTransaction(tx) => {
                let mut tx = tx.lock().await;
                let res = delete_scope(&mut *tx, tenant, project, key).await?;
                Ok(res)
            }
        }
    }

    async fn search(
        &self,
        tenant: &crate::types::TenantId,
        project: &crate::types::ProjectId,
        clauses: &ScopeSearchClaims,
    ) -> Result<Vec<Scope>, OperationOutcomeError> {
        match self {
            PGConnection::PgPool(pool) => {
                let res = search_scopes(pool, tenant, project, clauses).await?;
                Ok(res)
            }
            PGConnection::PgTransaction(tx) => {
                let mut tx = tx.lock().await;
                let res = search_scopes(&mut *tx, tenant, project, clauses).await?;
                Ok(res)
            }
        }
    }
}
