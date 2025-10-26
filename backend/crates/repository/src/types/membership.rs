use serde::{Deserialize, Serialize};

use crate::types::{ProjectId, TenantId, scope::UserId};

#[derive(sqlx::FromRow, Debug, Deserialize, Serialize)]
pub struct Membership {
    pub tenant: TenantId,
    pub project: ProjectId,
    pub user_id: String,
    pub role: MembershipRole,
}

pub struct UpdateMembership {
    pub user_id: String,
    pub role: MembershipRole,
}

pub struct MembershipSearchClaims {
    pub user_id: Option<UserId>,
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
