use crate::types::scope::UserId;
use oxidized_jwt::{ProjectId, TenantId};
use serde::{Deserialize, Serialize};

#[derive(sqlx::FromRow, Debug, Deserialize, Serialize)]
pub struct Membership {
    pub resource_id: String,
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
    pub resource_id: String,
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
