use oxidized_fhir_model::r4::generated::resources::Membership;

use crate::{
    admin::{Login, ProjectAuthAdmin, TenantAuthAdmin},
    fhir::FHIRRepository,
    types::{
        authorization_code::{
            AuthorizationCode, AuthorizationCodeSearchClaims, CreateAuthorizationCode,
        },
        membership::{CreateMembership, UpdateMembership},
        tenant::{CreateTenant, Tenant, TenantSearchClaims},
        user::{CreateUser, UpdateUser, User, UserSearchClauses},
    },
};

pub mod admin;
pub mod fhir;
pub mod pg;
pub mod types;
pub mod utilities;

/// Repository trait which encompasses all repository operations.
pub trait Repository:
    FHIRRepository
    + TenantAuthAdmin<CreateAuthorizationCode, AuthorizationCode, AuthorizationCodeSearchClaims>
    + ProjectAuthAdmin<CreateMembership, Membership, MembershipSearchClaims, UpdateMembership>
    + TenantAuthAdmin<CreateTenant, Tenant, TenantSearchClaims>
    + TenantAuthAdmin<CreateUser, User, UserSearchClauses, UpdateUser>
    + Login
{
}
