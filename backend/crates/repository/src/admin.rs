/// Authentication traits include management for user and Authorization codes.
use crate::types::user::{LoginMethod, LoginResult};
use oxidized_fhir_operation_error::OperationOutcomeError;
use oxidized_jwt::{ProjectId, TenantId};

pub trait Login {
    fn login(
        &self,
        tenant: &TenantId,
        method: &LoginMethod,
    ) -> impl Future<Output = Result<LoginResult, OperationOutcomeError>> + Send;
}

pub trait TenantAuthAdmin<CreatedModel, ReadModel, SearchClauses, UpdateModel, Key> {
    fn create(
        &self,
        tenant: &TenantId,
        model: CreatedModel,
    ) -> impl Future<Output = Result<ReadModel, OperationOutcomeError>> + Send;
    fn read(
        &self,
        tenant: &TenantId,
        id: &Key,
    ) -> impl Future<Output = Result<Option<ReadModel>, OperationOutcomeError>> + Send;
    fn update(
        &self,
        tenant: &TenantId,
        model: UpdateModel,
    ) -> impl Future<Output = Result<ReadModel, OperationOutcomeError>> + Send;
    fn delete(
        &self,
        tenant: &TenantId,
        id: &Key,
    ) -> impl Future<Output = Result<ReadModel, OperationOutcomeError>> + Send;
    fn search(
        &self,
        tenant: &TenantId,
        clauses: &SearchClauses,
    ) -> impl Future<Output = Result<Vec<ReadModel>, OperationOutcomeError>> + Send;
}

pub trait ProjectAuthAdmin<CreatedModel, ReadModel, SearchClauses, UpdateModel, Key> {
    fn create(
        &self,
        tenant: &TenantId,
        project: &ProjectId,
        model: CreatedModel,
    ) -> impl Future<Output = Result<ReadModel, OperationOutcomeError>> + Send;
    fn read(
        &self,
        tenant: &TenantId,
        project: &ProjectId,
        id: &Key,
    ) -> impl Future<Output = Result<Option<ReadModel>, OperationOutcomeError>> + Send;
    fn update(
        &self,
        tenant: &TenantId,
        project: &ProjectId,
        model: UpdateModel,
    ) -> impl Future<Output = Result<ReadModel, OperationOutcomeError>> + Send;
    fn delete(
        &self,
        tenant: &TenantId,
        project: &ProjectId,
        id: &Key,
    ) -> impl Future<Output = Result<ReadModel, OperationOutcomeError>> + Send;
    fn search(
        &self,
        tenant: &TenantId,
        project: &ProjectId,
        clauses: &SearchClauses,
    ) -> impl Future<Output = Result<Vec<ReadModel>, OperationOutcomeError>> + Send;
}
