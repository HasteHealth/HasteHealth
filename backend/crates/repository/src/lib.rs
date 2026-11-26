use crate::{
    admin::{Login, Migrate, ProjectModelAdmin, SystemAdmin, TenantModelAdmin},
    async_job::AsyncJobProvider,
    client_assertion::ClientAssertionReplayProvider,
    failed_indexing::FailedIndexingProvider,
    fhir::FHIRRepository,
    sequence::ResourceSequential,
    tenant_key::TenantKeyProvider,
    tenant_secret::TenantSecretProvider,
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
pub mod async_job;
pub mod client_assertion;
pub mod config;
pub mod failed_indexing;
pub mod fhir;
pub mod pg;
pub mod sequence;
pub mod tenant_key;
pub mod tenant_secret;
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
    + FailedIndexingProvider
    + AsyncJobProvider
    + ClientAssertionReplayProvider
    + TenantKeyProvider
    + TenantSecretProvider
    + Login
    + ResourceSequential
    + Migrate
{
}
