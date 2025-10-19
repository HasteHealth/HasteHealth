use crate::{
    admin::{Login, ProjectAuthAdmin, TenantAuthAdmin},
    fhir::FHIRRepository,
    types::{
        authorization_code::{
            AuthorizationCode, AuthorizationCodeSearchClaims, CreateAuthorizationCode,
        },
        membership::{CreateMembership, Membership, MembershipSearchClaims, UpdateMembership},
        project::{CreateProject, Project, ProjectSearchClaims},
        scope::{CreateScope, Scope, ScopeKey, ScopeSearchClaims, UpdateScope},
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
pub trait Repository: FHIRRepository
    + TenantAuthAdmin<
        CreateAuthorizationCode,
        AuthorizationCode,
        AuthorizationCodeSearchClaims,
        AuthorizationCode,
        String,
    > + TenantAuthAdmin<CreateTenant, Tenant, TenantSearchClaims, Tenant, String>
    + TenantAuthAdmin<CreateUser, User, UserSearchClauses, UpdateUser, String>
    + TenantAuthAdmin<CreateProject, Project, ProjectSearchClaims, Project, String>
    + ProjectAuthAdmin<
        CreateAuthorizationCode,
        AuthorizationCode,
        AuthorizationCodeSearchClaims,
        AuthorizationCode,
        String,
    > + ProjectAuthAdmin<CreateMembership, Membership, MembershipSearchClaims, UpdateMembership, String>
    + ProjectAuthAdmin<CreateScope, Scope, ScopeSearchClaims, UpdateScope, ScopeKey>
    + Login
{
}
