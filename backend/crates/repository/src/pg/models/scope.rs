use crate::{
    admin::ProjectModelAdmin,
    pg::{PGConnection, StoreError},
    types::scope::{CreateScope, Scope, ScopeKey, ScopeSearchClaims, UpdateScope},
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_jwt::{ProjectId, TenantId};
use sqlx::{PgExecutor, QueryBuilder};

async fn create_scope<'a, 'e, E>(
    executor: E,
    tenant: &'a TenantId,
    project: &'a ProjectId,
    scope: CreateScope,
) -> Result<Scope, OperationOutcomeError>
where
    E: PgExecutor<'e>,
{
    let scope = sqlx::query_as::<_, Scope>(
        r"
            INSERT INTO authorization_scopes(tenant, project, client, user_, scope)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (tenant, project, client, user_)
            DO UPDATE SET scope = $5
            RETURNING client, user_, scope, created_at
        ",
    )
    .bind(tenant.as_ref())
    .bind(project.as_ref())
    .bind(scope.client.as_ref())
    .bind(scope.user_.as_ref())
    .bind(scope.scope)
    .fetch_one(executor)
    .await
    .map_err(StoreError::SQLXError)?;

    Ok(scope)
}

async fn update_scope<'a, 'e, E>(
    executor: E,
    tenant: &'a TenantId,
    project: &'a ProjectId,
    model: UpdateScope,
) -> Result<Scope, OperationOutcomeError>
where
    E: PgExecutor<'e>,
{
    let mut query_builder = QueryBuilder::new(
        r"
            UPDATE authorization_scopes SET
        ",
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
        .push(" user_ = ")
        .push_bind_unseparated(model.user_.as_ref());

    query_builder.push(r" RETURNING client, user_, scope, created_at");

    let query = query_builder.build_query_as::<Scope>();

    let scope = query
        .fetch_one(executor)
        .await
        .map_err(StoreError::SQLXError)?;

    Ok(scope)
}

async fn read_scope<'a, 'e, E>(
    executor: E,
    tenant: &'a TenantId,
    project: &'a ProjectId,
    id: &'a ScopeKey,
) -> Result<Option<Scope>, OperationOutcomeError>
where
    E: PgExecutor<'e>,
{
    let scope = sqlx::query_as::<_, Scope>(
        r"
            SELECT user_, client, scope, created_at
            FROM authorization_scopes
            WHERE tenant = $1 AND project = $2 AND client = $3 AND user_ = $4
        ",
    )
    .bind(tenant.as_ref())
    .bind(project.as_ref())
    .bind(String::from(id.0.clone()))
    .bind(String::from(id.1.clone()))
    .fetch_optional(executor)
    .await
    .map_err(StoreError::SQLXError)?;

    Ok(scope)
}

async fn delete_scope<'a, 'e, E>(
    executor: E,
    tenant: &'a TenantId,
    project: &'a ProjectId,
    key: &'a ScopeKey,
) -> Result<(), OperationOutcomeError>
where
    E: PgExecutor<'e>,
{
    sqlx::query(
        r"
            DELETE FROM authorization_scopes
            WHERE tenant = $1 AND project = $2 AND client = $3 AND user_ = $4
        ",
    )
    .bind(tenant.as_ref())
    .bind(project.as_ref())
    .bind(key.0.as_ref())
    .bind(key.1.as_ref())
    .execute(executor)
    .await
    .map_err(StoreError::SQLXError)?;

    Ok(())
}

async fn search_scopes<'a, 'e, E>(
    executor: E,
    tenant: &'a TenantId,
    project: &'a ProjectId,
    clauses: &'a ScopeSearchClaims,
) -> Result<Vec<Scope>, OperationOutcomeError>
where
    E: PgExecutor<'e>,
{
    let mut query_builder: QueryBuilder<sqlx::Postgres> = QueryBuilder::new(
        r"SELECT user_, client, scope, created_at FROM authorization_scopes WHERE ",
    );

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

    let query = query_builder.build_query_as::<Scope>();

    let scopes: Vec<Scope> = query.fetch_all(executor).await.map_err(StoreError::from)?;

    Ok(scopes)
}

impl ProjectModelAdmin<CreateScope, Scope, ScopeSearchClaims, UpdateScope, ScopeKey>
    for PGConnection
{
    async fn create(
        &self,
        tenant: &TenantId,
        project: &ProjectId,
        new_scope: CreateScope,
    ) -> Result<Scope, OperationOutcomeError> {
        match self {
            PGConnection::Pool(pool, _) => create_scope(pool, tenant, project, new_scope).await,
            PGConnection::Transaction(tx, _, _) => {
                let mut tx = tx.lock().await;
                create_scope(&mut **tx, tenant, project, new_scope).await
            }
        }
    }

    async fn read(
        &self,
        tenant: &TenantId,
        project: &ProjectId,
        key: &ScopeKey,
    ) -> Result<Option<Scope>, OperationOutcomeError> {
        match self {
            PGConnection::Pool(pool, _) => read_scope(pool, tenant, project, key).await,
            PGConnection::Transaction(tx, _, _) => {
                let mut tx = tx.lock().await;
                read_scope(&mut **tx, tenant, project, key).await
            }
        }
    }

    async fn update(
        &self,
        tenant: &TenantId,
        project: &ProjectId,
        model: UpdateScope,
    ) -> Result<Scope, OperationOutcomeError> {
        match self {
            PGConnection::Pool(pool, _) => update_scope(pool, tenant, project, model).await,
            PGConnection::Transaction(tx, _, _) => {
                let mut tx = tx.lock().await;
                update_scope(&mut **tx, tenant, project, model).await
            }
        }
    }

    async fn delete(
        &self,
        tenant: &TenantId,
        project: &ProjectId,
        key: &ScopeKey,
    ) -> Result<(), OperationOutcomeError> {
        match self {
            PGConnection::Pool(pool, _) => delete_scope(pool, tenant, project, key).await,
            PGConnection::Transaction(tx, _, _) => {
                let mut tx = tx.lock().await;
                delete_scope(&mut **tx, tenant, project, key).await
            }
        }
    }

    async fn search(
        &self,
        tenant: &TenantId,
        project: &ProjectId,
        clauses: &ScopeSearchClaims,
    ) -> Result<Vec<Scope>, OperationOutcomeError> {
        match self {
            PGConnection::Pool(pool, _) => search_scopes(pool, tenant, project, clauses).await,
            PGConnection::Transaction(tx, _, _) => {
                let mut tx = tx.lock().await;
                search_scopes(&mut **tx, tenant, project, clauses).await
            }
        }
    }
}
