use oxidized_fhir_operation_error::derive::OperationOutcomeError;
use sqlx::types::Json;
use std::time::Duration;

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

pub struct AuthorizationCodeSearchClaims {
    pub client_id: Option<String>,
    pub code: Option<String>,
    pub user_id: Option<String>,
}

pub struct CreateAuthorizationCode {
    pub expires_in: Duration,
    pub kind: AuthorizationCodeKind,
    pub user_id: String,
    pub client_id: Option<String>,
    pub pkce_code_challenge: Option<String>,
    pub pkce_code_challenge_method: Option<PKCECodeChallengeMethod>,
    pub redirect_uri: Option<String>,
    pub meta: Option<Json<serde_json::Value>>,
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
