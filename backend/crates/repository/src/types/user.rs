use oxidized_fhir_model::r4::generated::terminology::UserRole as FHIRUserRole;
use oxidized_jwt::TenantId;
use serde::{Deserialize, Serialize};

#[derive(sqlx::FromRow, Debug, Deserialize, Serialize)]
pub struct User {
    pub id: String,
    pub tenant: TenantId,
    pub email: String,
    pub role: UserRole,
    pub method: AuthMethod,
    pub provider_id: Option<String>,
}

pub struct UpdateUser {
    pub id: String,
    pub email: Option<String>,
    pub role: Option<UserRole>,
    pub method: Option<AuthMethod>,
    pub provider_id: Option<String>,
    pub password: Option<String>,
}

pub enum LoginMethod {
    OIDC { email: String, provider_id: String },
    EmailPassword { email: String, password: String },
}

pub enum LoginResult {
    Success { user: User },
    Failure,
}

pub struct UserSearchClauses {
    pub email: Option<String>,
    pub role: Option<UserRole>,
    pub method: Option<AuthMethod>,
}

pub struct CreateUser {
    pub id: String,
    pub email: String,
    pub role: UserRole,
    pub method: AuthMethod,
    pub provider_id: Option<String>,
    pub password: Option<String>,
}

#[derive(Clone, Debug, PartialEq, PartialOrd, sqlx::Type, serde::Deserialize, serde::Serialize)]
#[sqlx(type_name = "auth_method", rename_all = "lowercase")] // only for PostgreSQL to match a type definition
pub enum AuthMethod {
    #[sqlx(rename = "email-password")]
    EmailPassword,
    OIDC,
}

#[derive(Clone, Debug, PartialEq, PartialOrd, sqlx::Type, serde::Deserialize, serde::Serialize)]
#[sqlx(type_name = "user_role", rename_all = "lowercase")] // only for PostgreSQL to match a type definition
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    Owner,
    Admin,
    Member,
}

impl From<FHIRUserRole> for UserRole {
    fn from(role: FHIRUserRole) -> Self {
        match role {
            FHIRUserRole::Owner(_) => UserRole::Owner,
            FHIRUserRole::Admin(_) => UserRole::Admin,
            FHIRUserRole::Member(_) => UserRole::Member,
            FHIRUserRole::Null(_) => UserRole::Member,
        }
    }
}
