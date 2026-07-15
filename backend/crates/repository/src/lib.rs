use crate::{
    admin::{Login, Migrate, ProjectModelAdmin, SystemAdmin, TenantModelAdmin},
    fhir::FHIRRepository,
    sequence::ResourceSequential,
    types::{
        authorization_code::{
            AuthorizationCode, AuthorizationCodeSearchClaims, CreateAuthorizationCode,
        },
        membership::{CreateMembership, Membership, MembershipSearchClaims},
        mfa::{
            MFAKey, UserMFACredential, UserMFACredentialCreate, UserMFACredentialUpdate,
            UserMFASearchClaims,
        },
        project::{CreateProject, Project, ProjectSearchClaims},
        scope::{CreateScope, Scope, ScopeKey, ScopeSearchClaims, UpdateScope},
        tenant::{CreateTenant, Tenant, TenantSearchClaims},
        user::{CreateUser, UpdateUser, User, UserSearchClauses},
    },
};

pub mod admin;
pub mod fhir;
pub mod pg;
pub mod sequence;
pub mod types;
pub mod utilities;

/// Repository trait which encompasses all repository operations.
pub trait Repository:
    FHIRRepository
    + SystemAdmin<User, UserSearchClauses>
    + TenantModelAdmin<
        CreateAuthorizationCode,
        AuthorizationCode,
        AuthorizationCodeSearchClaims,
        AuthorizationCode,
        String,
    > + TenantModelAdmin<CreateTenant, Tenant, TenantSearchClaims, Tenant, String>
    + TenantModelAdmin<CreateUser, User, UserSearchClauses, UpdateUser, String>
    + TenantModelAdmin<CreateProject, Project, ProjectSearchClaims, Project, String>
    + TenantModelAdmin<
        UserMFACredentialCreate,
        UserMFACredential,
        UserMFASearchClaims,
        UserMFACredentialUpdate,
        MFAKey,
    > + ProjectModelAdmin<
        CreateAuthorizationCode,
        AuthorizationCode,
        AuthorizationCodeSearchClaims,
        AuthorizationCode,
        String,
    > + ProjectModelAdmin<CreateMembership, Membership, MembershipSearchClaims, Membership, String>
    + ProjectModelAdmin<CreateScope, Scope, ScopeSearchClaims, UpdateScope, ScopeKey>
    + Login
    + ResourceSequential
    + Migrate
{
}
