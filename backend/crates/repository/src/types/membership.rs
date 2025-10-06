use serde::{Deserialize, Serialize};

use crate::types::{ProjectId, TenantId};

#[derive(sqlx::FromRow, Debug, Deserialize, Serialize)]
pub struct Membership {
    pub tenant: String,
    pub project: String,
    pub user_id: String,
    pub role: MembershipRole,
}

pub struct UpdateMembership {
    pub role: MembershipRole,
}

pub struct MembershipSearchClaims {
    pub tenant: Option<String>,
    pub project: Option<String>,
    pub user_id: Option<String>,
    pub role: Option<MembershipRole>,
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
