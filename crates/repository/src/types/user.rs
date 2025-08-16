#[derive(sqlx::FromRow, Debug)]
pub struct User {
    pub id: String,
    pub email: String,
    pub role: UserRole,
    pub method: AuthMethod,
    pub provider_id: Option<String>,
}

pub struct UpdateUser {
    pub id: String,
    pub email: String,
    pub role: UserRole,
    pub method: AuthMethod,
    pub provider_id: Option<String>,
    pub password: Option<String>,
}

pub enum LoginMethod {
    OIDC { email: String, provider_id: String },
    EmailPassword { email: String, password: String },
}

pub enum LoginResult {
    Success { user: User },
}

pub struct UserSearchClauses {
    pub email: Option<String>,
    pub role: Option<UserRole>,
}

pub struct CreateUser {
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
pub enum UserRole {
    Owner,
    Admin,
    Member,
}
