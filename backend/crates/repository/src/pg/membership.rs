use crate::{
    admin::ProjectAuthAdmin,
    pg::{PGConnection, StoreError},
    types::{
        TenantId,
        tenant::{CreateTenant, Tenant, TenantSearchClaims},
    },
    utilities::generate_id,
};
use oxidized_fhir_operation_error::OperationOutcomeError;
use sqlx::{Acquire, Postgres, QueryBuilder};

impl ProjectAuthAdmin<CreateUser, User, UserSearchClauses, UpdateUser> for PGConnection {}
