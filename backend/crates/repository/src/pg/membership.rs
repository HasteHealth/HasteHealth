use crate::{
    admin::ProjectAuthAdmin,
    pg::PGConnection,
    types::membership::{CreateMembership, MembershipSearchClaims, UpdateMembership},
};
use oxidized_fhir_model::r4::generated::resources::Membership;
use oxidized_fhir_operation_error::OperationOutcomeError;
use sqlx::{Acquire, Postgres, QueryBuilder};

impl ProjectAuthAdmin<CreateMembership, Membership, MembershipSearchClaims, UpdateMembership>
    for PGConnection
{
    fn create(
        &self,
        tenant: &crate::types::TenantId,
        project: &crate::types::ProjectId,
        model: CreateMembership,
    ) -> impl Future<Output = Result<Membership, OperationOutcomeError>> + Send {
        todo!()
    }

    fn read(
        &self,
        tenant: &crate::types::TenantId,
        project: &crate::types::ProjectId,
        id: &str,
    ) -> impl Future<Output = Result<Membership, OperationOutcomeError>> + Send {
        todo!()
    }

    fn update(
        &self,
        tenant: &crate::types::TenantId,
        project: &crate::types::ProjectId,
        model: UpdateMembership,
    ) -> impl Future<Output = Result<Membership, OperationOutcomeError>> + Send {
        todo!()
    }

    fn delete(
        &self,
        tenant: &crate::types::TenantId,
        project: &crate::types::ProjectId,
        id: &str,
    ) -> impl Future<Output = Result<Membership, OperationOutcomeError>> + Send {
        todo!()
    }

    fn search(
        &self,
        tenant: &crate::types::TenantId,
        project: &crate::types::ProjectId,
        clauses: &MembershipSearchClaims,
    ) -> impl Future<Output = Result<Vec<Membership>, OperationOutcomeError>> + Send {
        todo!()
    }
}
