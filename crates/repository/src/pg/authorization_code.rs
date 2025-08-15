use std::time::Duration;

use oxidized_fhir_operation_error::{OperationOutcomeError, derive::OperationOutcomeError};
use sqlx::{Acquire, Postgres, QueryBuilder, types::Json};
use sqlx_postgres::types::PgInterval;

use crate::{
    ProjectId, TenantId,
    admin::TenantAuthAdmin,
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
    tenant: String,
    expires_in: Duration,
    kind: AuthorizationCodeKind,
    code: String,
    user_id: String,
    project: Option<String>,
    client_id: Option<String>,
    pkce_code_challenge: Option<String>,
    pkce_code_challenge_method: Option<PKCECodeChallengeMethod>,
    redirect_uri: Option<String>,
    meta: Option<Json<serde_json::Value>>,
}

#[derive(sqlx::FromRow, Debug)]
struct AuthorizationCode {
    tenant: String,
    is_expired: Option<bool>,
    kind: AuthorizationCodeKind,
    code: String,
    user_id: String,
    project: Option<String>,
    client_id: Option<String>,
    pkce_code_challenge: Option<String>,
    pkce_code_challenge_method: Option<PKCECodeChallengeMethod>,
    redirect_uri: Option<String>,
    meta: Option<Json<serde_json::Value>>,
}

#[derive(OperationOutcomeError)]
pub enum CodeErrors {
    #[error(code = "invalid", diagnostic = "Invalid duration for expires.")]
    InvalidDuration,
}

fn create_code<'a, 'c, Connection: Acquire<'c, Database = Postgres> + Send + 'a>(
    connection: Connection,
    tenant: &'a TenantId,
    project_id: Option<&'a ProjectId>,
    authorization_code: CreateAuthorizationCode,
) -> impl Future<Output = Result<AuthorizationCode, OperationOutcomeError>> + Send + 'a {
    async move {
        let expires_in: PgInterval = authorization_code
            .expires_in
            .try_into()
            .map_err(|_e| CodeErrors::InvalidDuration)?;
        sqlx::query_as!(
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
            authorization_code.tenant,
            authorization_code.project,
            authorization_code.client_id,
            authorization_code.kind as AuthorizationCodeKind,
            authorization_code.code,
            expires_in as PgInterval,
            authorization_code.user_id,
            authorization_code.pkce_code_challenge,
            authorization_code.pkce_code_challenge_method as Option<PKCECodeChallengeMethod>,
            authorization_code.redirect_uri,
            authorization_code.meta as std::option::Option<Json<serde_json::Value>>,
        );

        todo!();
    }
}

struct AuthorizationCodeSearchClaims {}

impl TenantAuthAdmin<CreateAuthorizationCode, AuthorizationCode, AuthorizationCodeSearchClaims>
    for PGConnection
{
    async fn create(
        &self,
        tenant: &TenantId,
        authorization_code: CreateAuthorizationCode,
    ) -> Result<AuthorizationCode, OperationOutcomeError> {
        let code = generate_id(Some(45));

        match &self {
            PGConnection::PgPool(pool) => {
                let res = create_code(pool, tenant, authorization_code).await?;
                Ok(res)
            }
            PGConnection::PgTransaction(tx) => {
                let mut tx = tx.lock().await;

                let res = create_code(&mut *tx, tenant, authorization_code).await?;
                Ok(res)
            }
        }
    }

    async fn read(
        &self,
        tenant: &TenantId,
        id: &str,
    ) -> Result<AuthorizationCode, OperationOutcomeError> {
        todo!()
    }

    async fn update(
        &self,
        tenant: &TenantId,
        model: AuthorizationCode,
    ) -> Result<AuthorizationCode, OperationOutcomeError> {
        todo!()
    }

    async fn delete(
        &self,
        tenant: &TenantId,
        id: &str,
    ) -> Result<AuthorizationCode, OperationOutcomeError> {
        todo!()
    }

    async fn search(
        &self,
        tenant: &TenantId,
        clauses: &AuthorizationCodeSearchClaims,
    ) -> Result<Vec<AuthorizationCode>, OperationOutcomeError> {
        todo!()
    }
}
