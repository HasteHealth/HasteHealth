use oxidized_fhir_operation_error::OperationOutcomeError;
use sqlx::{Acquire, Postgres, QueryBuilder, types::Json};

use crate::{
    TenantId,
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
    expires_in: String,
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
    id: String,
    tenant: String,
    is_expired: bool,
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

struct AuthorizationCodeSearchClaims {}

impl TenantAuthAdmin<CreateAuthorizationCode, AuthorizationCode, AuthorizationCodeSearchClaims>
    for PGConnection
{
    async fn create(
        &self,
        tenant: &TenantId,
        model: CreateAuthorizationCode,
    ) -> Result<AuthorizationCode, OperationOutcomeError> {
        let code = generate_id(Some(45));
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
