use haste_fhir_model::r4::generated::terminology::IssueType;
use haste_fhir_operation_error::{OperationOutcomeError, derive::OperationOutcomeError};
use haste_jwt::{ProjectId, TenantId};
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

impl<'a> TryFrom<&'a str> for PKCECodeChallengeMethod {
    type Error = OperationOutcomeError;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        match value {
            "S256" => Ok(PKCECodeChallengeMethod::S256),
            "plain" => Ok(PKCECodeChallengeMethod::Plain),
            _ => Err(OperationOutcomeError::error(
                IssueType::Invalid(None),
                "Invalid PKCE code challenge method.".to_string(),
            )),
        }
    }
}

impl From<PKCECodeChallengeMethod> for String {
    fn from(method: PKCECodeChallengeMethod) -> Self {
        match method {
            PKCECodeChallengeMethod::S256 => "S256".to_string(),
            PKCECodeChallengeMethod::Plain => "plain".to_string(),
        }
    }
}

pub struct AuthorizationCodeSearchClaims {
    pub client_id: Option<String>,
    pub code: Option<String>,
    pub user_id: Option<String>,
}

pub struct CreateAuthorizationCode {
    pub membership: Option<String>,
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
    pub membership: Option<String>,
    pub tenant: TenantId,
    pub is_expired: Option<bool>,
    pub kind: AuthorizationCodeKind,
    pub code: String,
    pub user_id: String,
    pub project: Option<ProjectId>,
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
