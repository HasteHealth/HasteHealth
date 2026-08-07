use crate::{
    admin::{ProjectModelAdmin, TenantModelAdmin},
    pg::{PGConnection, StoreError},
    types::authorization_code::{
        AuthorizationCode, AuthorizationCodeSearchClaims, CodeErrors, CreateAuthorizationCode,
    },
    utilities::generate_id,
};
use haste_fhir_model::r4::generated::terminology::IssueType;
use haste_fhir_operation_error::OperationOutcomeError;
use haste_jwt::{ProjectId, TenantId};
use sqlx::{PgExecutor, QueryBuilder};
use sqlx_postgres::types::PgInterval;

async fn create_code<'a, 'e, E>(
    executor: E,
    tenant: &'a TenantId,
    project: Option<&'a ProjectId>,
    authorization_code: CreateAuthorizationCode,
) -> Result<AuthorizationCode, OperationOutcomeError>
where
    E: PgExecutor<'e>,
{
    let expires_in: PgInterval = authorization_code
        .expires_in
        .try_into()
        .map_err(|_e| CodeErrors::InvalidDuration)?;

    let code = generate_id(Some(45));

    let new_authorization_code = sqlx::query_as::<_, AuthorizationCode>(
        r"
            INSERT INTO authorization_code (
                tenant, project, client_id, kind, code, expires_in,
                user_id, pkce_code_challenge, pkce_code_challenge_method, redirect_uri, meta, membership
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING
                tenant,
                kind,
                code,
                user_id,
                project,
                client_id,
                pkce_code_challenge,
                pkce_code_challenge_method,
                redirect_uri,
                meta,
                NOW() > (created_at + expires_in) as is_expired,
                membership,
                created_at
        ",
    )
    .bind(tenant)
    .bind(project)
    .bind(authorization_code.client_id)
    .bind(authorization_code.kind)
    .bind(code)
    .bind(expires_in)
    .bind(authorization_code.user_id)
    .bind(authorization_code.pkce_code_challenge)
    .bind(authorization_code.pkce_code_challenge_method)
    .bind(authorization_code.redirect_uri)
    .bind(authorization_code.meta)
    .bind(authorization_code.membership)
    .fetch_one(executor)
    .await
    .map_err(StoreError::SQLXError)?;

    Ok(new_authorization_code)
}

async fn read_code<'a, 'e, E>(
    executor: E,
    tenant: &'a TenantId,
    project: Option<&'a ProjectId>,
    code: &'a str,
) -> Result<Option<AuthorizationCode>, OperationOutcomeError>
where
    E: PgExecutor<'e>,
{
    let mut query_builder: QueryBuilder<sqlx::Postgres> = QueryBuilder::new(
        r"
            SELECT tenant,
               kind,
               code,
               user_id,
               project,
               client_id,
               pkce_code_challenge,
               pkce_code_challenge_method,
               redirect_uri,
               meta,
               NOW() > (created_at + expires_in) as is_expired,
               membership,
               created_at
            FROM authorization_code
            WHERE
        ",
    );

    query_builder.push("tenant = ").push_bind(tenant.as_ref());
    query_builder.push(" AND code = ").push_bind(code);

    if let Some(project) = project {
        query_builder
            .push(" AND project = ")
            .push_bind(project.as_ref());
    }

    let query = query_builder.build_query_as::<AuthorizationCode>();

    let authorization_code = query
        .fetch_optional(executor)
        .await
        .map_err(StoreError::SQLXError)?;

    Ok(authorization_code)
}

async fn delete_code<'a, 'e, E>(
    executor: E,
    tenant: &'a TenantId,
    project: Option<&'a ProjectId>,
    code: &'a str,
) -> Result<(), OperationOutcomeError>
where
    E: PgExecutor<'e>,
{
    let mut query_builder: QueryBuilder<sqlx::Postgres> = QueryBuilder::new(
        r"
            DELETE FROM authorization_code
            WHERE
        ",
    );

    query_builder.push(" tenant = ").push_bind(tenant.as_ref());
    query_builder.push(" AND code = ").push_bind(code);

    if let Some(project) = project {
        query_builder
            .push(" AND project = ")
            .push_bind(project.as_ref());
    }

    let query = query_builder.build();

    query
        .execute(executor)
        .await
        .map_err(StoreError::SQLXError)?;

    Ok(())
}

async fn search_codes<'a, 'e, E>(
    executor: E,
    tenant: &'a TenantId,
    project: Option<&'a ProjectId>,
    clauses: &'a AuthorizationCodeSearchClaims,
) -> Result<Vec<AuthorizationCode>, OperationOutcomeError>
where
    E: PgExecutor<'e>,
{
    let mut query_builder: QueryBuilder<sqlx::Postgres> = QueryBuilder::new(
        r"
            SELECT tenant,
               kind,
               code,
               user_id,
               project,
               client_id,
               pkce_code_challenge,
               pkce_code_challenge_method,
               redirect_uri,
               meta,
               NOW() > (created_at + expires_in) as is_expired,
               membership,
               created_at
            FROM authorization_code
            WHERE
        ",
    );

    query_builder.push(" tenant = ").push_bind(tenant.as_ref());

    if let Some(project) = project {
        query_builder
            .push(" AND project = ")
            .push_bind(project.as_ref());
    }

    if let Some(client_id) = &clauses.client_id {
        query_builder.push(" AND client_id = ").push_bind(client_id);
    }

    if let Some(code) = &clauses.code {
        query_builder.push(" AND code = ").push_bind(code);
    }

    if let Some(user_id) = &clauses.user_id {
        query_builder.push(" AND user_id = ").push_bind(user_id);
    }

    if let Some(kind) = &clauses.kind {
        query_builder.push(" AND kind = ").push_bind(kind);
    }

    if let Some(user_agent) = &clauses.user_agent {
        query_builder
            .push(" AND meta->>'user_agent' = ")
            .push_bind(user_agent);
    }

    if let Some(is_expired) = &clauses.is_expired {
        query_builder
            .push(" AND (NOW() > (created_at + expires_in)) = ")
            .push_bind(is_expired);
    }

    let query = query_builder.build_query_as::<AuthorizationCode>();

    let authorization_codes = query
        .fetch_all(executor)
        .await
        .map_err(StoreError::SQLXError)?;

    Ok(authorization_codes)
}

impl<Key: AsRef<str> + Send + Sync>
    TenantModelAdmin<
        CreateAuthorizationCode,
        AuthorizationCode,
        AuthorizationCodeSearchClaims,
        AuthorizationCode,
        Key,
    > for PGConnection
{
    async fn create(
        &self,
        tenant: &TenantId,
        authorization_code: CreateAuthorizationCode,
    ) -> Result<AuthorizationCode, OperationOutcomeError> {
        match &self {
            PGConnection::Pool(pool, _) => {
                create_code(pool, tenant, None, authorization_code).await
            }
            PGConnection::Transaction(tx, _) => {
                let mut tx = tx.lock().await;
                create_code(&mut **tx, tenant, None, authorization_code).await
            }
        }
    }

    async fn read(
        &self,
        tenant: &TenantId,
        code: &Key,
    ) -> Result<Option<AuthorizationCode>, OperationOutcomeError> {
        match &self {
            PGConnection::Pool(pool, _) => read_code(pool, tenant, None, code.as_ref()).await,
            PGConnection::Transaction(tx, _) => {
                let mut tx = tx.lock().await;
                read_code(&mut **tx, tenant, None, code.as_ref()).await
            }
        }
    }

    async fn update(
        &self,
        _tenant: &TenantId,
        _model: AuthorizationCode,
    ) -> Result<AuthorizationCode, OperationOutcomeError> {
        Err(OperationOutcomeError::fatal(
            IssueType::exception(),
            "Update operation for AuthorizationCode is not implemented.".to_string(),
        ))
    }

    async fn delete(&self, tenant: &TenantId, code: &Key) -> Result<(), OperationOutcomeError> {
        match &self {
            PGConnection::Pool(pool, _) => delete_code(pool, tenant, None, code.as_ref()).await,
            PGConnection::Transaction(tx, _) => {
                let mut tx = tx.lock().await;
                delete_code(&mut **tx, tenant, None, code.as_ref()).await
            }
        }
    }

    async fn search(
        &self,
        tenant: &TenantId,
        clauses: &AuthorizationCodeSearchClaims,
    ) -> Result<Vec<AuthorizationCode>, OperationOutcomeError> {
        match &self {
            PGConnection::Pool(pool, _) => search_codes(pool, tenant, None, clauses).await,
            PGConnection::Transaction(tx, _) => {
                let mut tx = tx.lock().await;
                search_codes(&mut **tx, tenant, None, clauses).await
            }
        }
    }
}

impl<Key: AsRef<str> + Send + Sync>
    ProjectModelAdmin<
        CreateAuthorizationCode,
        AuthorizationCode,
        AuthorizationCodeSearchClaims,
        AuthorizationCode,
        Key,
    > for PGConnection
{
    async fn create(
        &self,
        tenant: &TenantId,
        project: &ProjectId,
        authorization_code: CreateAuthorizationCode,
    ) -> Result<AuthorizationCode, OperationOutcomeError> {
        match &self {
            PGConnection::Pool(pool, _) => {
                create_code(pool, tenant, Some(project), authorization_code).await
            }
            PGConnection::Transaction(tx, _) => {
                let mut tx = tx.lock().await;
                create_code(&mut **tx, tenant, Some(project), authorization_code).await
            }
        }
    }

    async fn read(
        &self,
        tenant: &TenantId,
        project: &ProjectId,
        code: &Key,
    ) -> Result<Option<AuthorizationCode>, OperationOutcomeError> {
        match &self {
            PGConnection::Pool(pool, _) => {
                read_code(pool, tenant, Some(project), code.as_ref()).await
            }
            PGConnection::Transaction(tx, _) => {
                let mut tx = tx.lock().await;
                read_code(&mut **tx, tenant, Some(project), code.as_ref()).await
            }
        }
    }

    async fn update(
        &self,
        _tenant: &TenantId,
        _project: &ProjectId,
        _model: AuthorizationCode,
    ) -> Result<AuthorizationCode, OperationOutcomeError> {
        Err(OperationOutcomeError::fatal(
            IssueType::exception(),
            "Update operation for AuthorizationCode is not implemented.".to_string(),
        ))
    }

    async fn delete(
        &self,
        tenant: &TenantId,
        project: &ProjectId,
        code: &Key,
    ) -> Result<(), OperationOutcomeError> {
        match &self {
            PGConnection::Pool(pool, _) => {
                delete_code(pool, tenant, Some(project), code.as_ref()).await
            }
            PGConnection::Transaction(tx, _) => {
                let mut tx = tx.lock().await;
                delete_code(&mut **tx, tenant, Some(project), code.as_ref()).await
            }
        }
    }

    async fn search(
        &self,
        tenant: &TenantId,
        project: &ProjectId,
        clauses: &AuthorizationCodeSearchClaims,
    ) -> Result<Vec<AuthorizationCode>, OperationOutcomeError> {
        match &self {
            PGConnection::Pool(pool, _) => search_codes(pool, tenant, Some(project), clauses).await,
            PGConnection::Transaction(tx, _) => {
                let mut tx = tx.lock().await;
                search_codes(&mut **tx, tenant, Some(project), clauses).await
            }
        }
    }
}
