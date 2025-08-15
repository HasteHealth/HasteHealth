use std::time::Duration;

use oxidized_fhir_operation_error::{OperationOutcomeError, derive::OperationOutcomeError};
use sqlx::{Acquire, Postgres, QueryBuilder, types::Json};
use sqlx_postgres::types::PgInterval;

use crate::{
    ProjectId, TenantId,
    admin::{ProjectAuthAdmin, TenantAuthAdmin},
    pg::{PGConnection, StoreError},
    utilities::generate_id,
};

#[derive(Clone, Debug, PartialEq, PartialOrd, sqlx::Type, serde::Deserialize, serde::Serialize)]
#[sqlx(type_name = "code_kind", rename_all = "lowercase")] // only for PostgreSQL to match a type definition
pub enum AuthorizationCodeKind {
    #[sqlx(rename = "password_reset")]
    PasswordReset,
    #[sqlx(rename = "oauth2_code_grant")]
    OAuth2CodeGrant,
    #[sqlx(rename = "refresh_token")]
    RefreshToken,
}

#[derive(Clone, Debug, PartialEq, PartialOrd, sqlx::Type, serde::Deserialize, serde::Serialize)]
#[sqlx(type_name = "pkce_method")] // only for PostgreSQL to match a type definition
pub enum PKCECodeChallengeMethod {
    S256,
    #[sqlx(rename = "plain")]
    Plain,
}

struct CreateAuthorizationCode {
    expires_in: Duration,
    kind: AuthorizationCodeKind,
    user_id: String,
    client_id: Option<String>,
    pkce_code_challenge: Option<String>,
    pkce_code_challenge_method: Option<PKCECodeChallengeMethod>,
    redirect_uri: Option<String>,
    meta: Option<Json<serde_json::Value>>,
}

#[derive(sqlx::FromRow, Debug)]
pub struct AuthorizationCode {
    pub tenant: String,
    pub is_expired: Option<bool>,
    pub kind: AuthorizationCodeKind,
    pub code: String,
    pub user_id: String,
    pub project: Option<String>,
    pub client_id: Option<String>,
    pub pkce_code_challenge: Option<String>,
    pub pkce_code_challenge_method: Option<PKCECodeChallengeMethod>,
    pub redirect_uri: Option<String>,
    pub meta: Option<Json<serde_json::Value>>,
}

#[derive(OperationOutcomeError)]
pub enum CodeErrors {
    #[error(code = "invalid", diagnostic = "Invalid duration for expires.")]
    InvalidDuration,
}

fn create_code<'a, 'c, Connection: Acquire<'c, Database = Postgres> + Send + 'a>(
    connection: Connection,
    tenant: &'a TenantId,
    project: Option<&'a ProjectId>,
    authorization_code: CreateAuthorizationCode,
) -> impl Future<Output = Result<AuthorizationCode, OperationOutcomeError>> + Send + 'a {
    async move {
        let expires_in: PgInterval = authorization_code
            .expires_in
            .try_into()
            .map_err(|_e| CodeErrors::InvalidDuration)?;

        let code = generate_id(Some(45));

        let mut conn = connection.acquire().await.map_err(StoreError::SQLXError)?;

        let new_authorization_code = sqlx::query_as!(
            AuthorizationCode,
            r#"
        INSERT INTO authorization_code (
            tenant, project, client_id, kind, code, expires_in,
            user_id, pkce_code_challenge, pkce_code_challenge_method, redirect_uri, meta
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        RETURNING tenant,
                  kind as "kind: AuthorizationCodeKind",
                  code,
                  user_id,
                  project,
                  client_id,
                  pkce_code_challenge,
                  pkce_code_challenge_method as "pkce_code_challenge_method: PKCECodeChallengeMethod",
                  redirect_uri,
                  meta as "meta: Json<serde_json::Value>",
                  NOW() > created_at + expires_in as is_expired
        "#,
            tenant.as_ref(),
            project.map(|p| p.as_ref()),
            authorization_code.client_id,
            authorization_code.kind as AuthorizationCodeKind,
            code,
            expires_in as PgInterval,
            authorization_code.user_id,
            authorization_code.pkce_code_challenge,
            authorization_code.pkce_code_challenge_method as Option<PKCECodeChallengeMethod>,
            authorization_code.redirect_uri,
            authorization_code.meta as std::option::Option<Json<serde_json::Value>>,
        ).fetch_one(&mut *conn).await.map_err(StoreError::SQLXError)?;

        Ok(new_authorization_code)
    }
}

fn read_code<'a, 'c, Connection: Acquire<'c, Database = Postgres> + Send + 'a>(
    connection: Connection,
    tenant: &'a TenantId,
    project: Option<&'a ProjectId>,
    code: &'a str,
) -> impl Future<Output = Result<AuthorizationCode, OperationOutcomeError>> + Send + 'a {
    async move {
        let mut conn = connection.acquire().await.map_err(StoreError::SQLXError)?;

        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
            r#"
            SELECT tenant,
               kind as "kind: AuthorizationCodeKind",
               code,
               user_id,
               project,
               client_id,
               pkce_code_challenge,
               pkce_code_challenge_method as "pkce_code_challenge_method: PKCECodeChallengeMethod",
               redirect_uri,
               meta as "meta: Json<serde_json::Value>",
               NOW() > created_at + expires_in as is_expired
            FROM authorization_code
            WHERE 
        "#,
        );

        query_builder.push("tenant = $1");
        query_builder.push_bind(tenant.as_ref());

        query_builder.push(" AND code = $2");
        query_builder.push_bind(code);

        if let Some(project) = project {
            query_builder.push(" AND project = $3");
            query_builder.push_bind(project.as_ref());
        }

        let query = query_builder.build_query_as();

        let authorization_code: AuthorizationCode = query
            .fetch_one(&mut *conn)
            .await
            .map_err(StoreError::SQLXError)?;

        Ok(authorization_code)
    }
}

fn delete_code<'a, 'c, Connection: Acquire<'c, Database = Postgres> + Send + 'a>(
    connection: Connection,
    tenant: &'a TenantId,
    project: Option<&'a ProjectId>,
    code: &'a str,
) -> impl Future<Output = Result<AuthorizationCode, OperationOutcomeError>> + Send + 'a {
    async move {
        let mut conn = connection.acquire().await.map_err(StoreError::SQLXError)?;

        let mut query_builder = QueryBuilder::new(
            r#"
            DELETE FROM authorization_code
            WHERE tenant = $1 AND code = $2

            "#,
        );
        query_builder.push_bind(tenant.as_ref());
        query_builder.push_bind(code);

        if let Some(project) = project {
            query_builder.push(" AND project = $3");
            query_builder.push_bind(project.as_ref());
        }

        query_builder.push(r#" 
             RETURNING tenant,
                  kind as "kind: AuthorizationCodeKind",
                  code,
                  user_id,
                  project,
                  client_id,
                  pkce_code_challenge,
                  pkce_code_challenge_method as "pkce_code_challenge_method: PKCECodeChallengeMethod",
                  redirect_uri,
                  meta as "meta: Json<serde_json::Value>",
                  NOW() > created_at + expires_in as is_expired
        "#);

        let query = query_builder.build_query_as();

        let authorization_code: AuthorizationCode = query
            .fetch_one(&mut *conn)
            .await
            .map_err(StoreError::SQLXError)?;

        Ok(authorization_code)
    }
}

fn search_codes<'a, 'c, Connection: Acquire<'c, Database = Postgres> + Send + 'a>(
    connection: Connection,
    tenant: &'a TenantId,
    project: Option<&'a ProjectId>,
    clauses: &'a AuthorizationCodeSearchClaims,
) -> impl Future<Output = Result<Vec<AuthorizationCode>, OperationOutcomeError>> + Send + 'a {
    async move {
        let mut conn = connection.acquire().await.map_err(StoreError::SQLXError)?;
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
            r#"
            SELECT tenant,
               kind as "kind: AuthorizationCodeKind",
               code,
               user_id,
               project,
               client_id,
               pkce_code_challenge,
               pkce_code_challenge_method as "pkce_code_challenge_method: PKCECodeChallengeMethod",
               redirect_uri,
               meta as "meta: Json<serde_json::Value>",
               NOW() > created_at + expires_in as is_expired
            FROM authorization_code
            WHERE
        "#,
        );

        query_builder.push(" tenant = $1 ");
        query_builder.push_bind(tenant.as_ref());

        if let Some(project) = project {
            query_builder.push(" AND project = $2 ");
            query_builder.push_bind(project.as_ref());
        }

        if let Some(client_id) = &clauses.client_id {
            query_builder.push(" AND client_id = $3 ");
            query_builder.push_bind(client_id);
        }

        if let Some(code) = &clauses.code {
            query_builder.push(" AND code = $4 ");
            query_builder.push_bind(code);
        }

        if let Some(user_id) = &clauses.user_id {
            query_builder.push(" AND user_id = $5 ");
            query_builder.push_bind(user_id);
        }

        let query = query_builder.build_query_as();

        let authorization_codes: Vec<AuthorizationCode> = query
            .fetch_all(&mut *conn)
            .await
            .map_err(StoreError::SQLXError)?;

        Ok(authorization_codes)
    }
}

struct AuthorizationCodeSearchClaims {
    client_id: Option<String>,
    code: Option<String>,
    user_id: Option<String>,
}

impl TenantAuthAdmin<CreateAuthorizationCode, AuthorizationCode, AuthorizationCodeSearchClaims>
    for PGConnection
{
    async fn create(
        &self,
        tenant: &TenantId,
        authorization_code: CreateAuthorizationCode,
    ) -> Result<AuthorizationCode, OperationOutcomeError> {
        match &self {
            PGConnection::PgPool(pool) => {
                let res = create_code(pool, tenant, None, authorization_code).await?;
                Ok(res)
            }
            PGConnection::PgTransaction(tx) => {
                let mut tx = tx.lock().await;

                let res = create_code(&mut *tx, tenant, None, authorization_code).await?;
                Ok(res)
            }
        }
    }

    async fn read(
        &self,
        tenant: &TenantId,
        code: &str,
    ) -> Result<AuthorizationCode, OperationOutcomeError> {
        match &self {
            PGConnection::PgPool(pool) => {
                let res = read_code(pool, tenant, None, code).await?;
                Ok(res)
            }
            PGConnection::PgTransaction(tx) => {
                let mut tx = tx.lock().await;

                let res = read_code(&mut *tx, tenant, None, code).await?;
                Ok(res)
            }
        }
    }

    async fn update(
        &self,
        _tenant: &TenantId,
        _model: AuthorizationCode,
    ) -> Result<AuthorizationCode, OperationOutcomeError> {
        Err(OperationOutcomeError::fatal(
            "exception".to_string(),
            "Update operation for AuthorizationCode is not implemented.".to_string(),
        ))
    }

    async fn delete(
        &self,
        tenant: &TenantId,
        code: &str,
    ) -> Result<AuthorizationCode, OperationOutcomeError> {
        match &self {
            PGConnection::PgPool(pool) => {
                let res = delete_code(pool, tenant, None, code).await?;
                Ok(res)
            }
            PGConnection::PgTransaction(tx) => {
                let mut tx = tx.lock().await;

                let res = delete_code(&mut *tx, tenant, None, code).await?;
                Ok(res)
            }
        }
    }

    async fn search(
        &self,
        tenant: &TenantId,
        clauses: &AuthorizationCodeSearchClaims,
    ) -> Result<Vec<AuthorizationCode>, OperationOutcomeError> {
        match &self {
            PGConnection::PgPool(pool) => {
                let res = search_codes(pool, tenant, None, clauses).await?;
                Ok(res)
            }
            PGConnection::PgTransaction(tx) => {
                let mut tx = tx.lock().await;

                let res = search_codes(&mut *tx, tenant, None, clauses).await?;
                Ok(res)
            }
        }
    }
}

impl ProjectAuthAdmin<CreateAuthorizationCode, AuthorizationCode, AuthorizationCodeSearchClaims>
    for PGConnection
{
    async fn create(
        &self,
        tenant: &TenantId,
        project: &ProjectId,
        authorization_code: CreateAuthorizationCode,
    ) -> Result<AuthorizationCode, OperationOutcomeError> {
        match &self {
            PGConnection::PgPool(pool) => {
                let res = create_code(pool, tenant, Some(project), authorization_code).await?;
                Ok(res)
            }
            PGConnection::PgTransaction(tx) => {
                let mut tx = tx.lock().await;

                let res = create_code(&mut *tx, tenant, Some(project), authorization_code).await?;
                Ok(res)
            }
        }
    }

    async fn read(
        &self,
        tenant: &TenantId,
        project: &ProjectId,
        code: &str,
    ) -> Result<AuthorizationCode, OperationOutcomeError> {
        match &self {
            PGConnection::PgPool(pool) => {
                let res = read_code(pool, tenant, Some(project), code).await?;
                Ok(res)
            }
            PGConnection::PgTransaction(tx) => {
                let mut tx = tx.lock().await;

                let res = read_code(&mut *tx, tenant, Some(project), code).await?;
                Ok(res)
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
            "exception".to_string(),
            "Update operation for AuthorizationCode is not implemented.".to_string(),
        ))
    }

    async fn delete(
        &self,
        tenant: &TenantId,
        project: &ProjectId,
        code: &str,
    ) -> Result<AuthorizationCode, OperationOutcomeError> {
        match &self {
            PGConnection::PgPool(pool) => {
                let res = delete_code(pool, tenant, Some(project), code).await?;
                Ok(res)
            }
            PGConnection::PgTransaction(tx) => {
                let mut tx = tx.lock().await;

                let res = delete_code(&mut *tx, tenant, Some(project), code).await?;
                Ok(res)
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
            PGConnection::PgPool(pool) => {
                let res = search_codes(pool, tenant, Some(project), clauses).await?;
                Ok(res)
            }
            PGConnection::PgTransaction(tx) => {
                let mut tx = tx.lock().await;

                let res = search_codes(&mut *tx, tenant, Some(project), clauses).await?;
                Ok(res)
            }
        }
    }
}
