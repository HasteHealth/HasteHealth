use serde::{Deserialize, Serialize};

#[derive(sqlx::FromRow, Debug, Deserialize, Serialize)]
pub struct Scope {
    pub client: String,
    pub user: String,
    pub scope: String,
}

pub struct UpdateMembership {
    pub client: String,
    pub user: String,
    pub scope: String,
}

pub struct MembershipSearchClaims {
    pub user: Option<String>,
    pub client: Option<String>,
}

pub struct CreateMembership {
    pub user_id: String,
    pub role: MembershipRole,
}

#[derive(Clone, Debug, PartialEq, PartialOrd, sqlx::Type, serde::Deserialize, serde::Serialize)]
#[sqlx(type_name = "membership_role", rename_all = "lowercase")] // only for PostgreSQL to match a type definition
#[serde(rename_all = "lowercase")]
pub enum MembershipRole {
    Admin,
    Member,
}
