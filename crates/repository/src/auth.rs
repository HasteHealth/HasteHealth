/// Authentication traits include management for user and Authorization codes.
use crate::{ProjectId, TenantId};
use oxidized_fhir_operation_error::OperationOutcomeError;

pub enum Models {
    User {
        id: String,
        email: String,
        name: String,
    },
    Tenant {
        id: String,
        name: String,
    },
    AuthorizationCode {
        id: String,
        user_id: String,
        scope: String,
    },
}

pub trait TenantAuthAdmin<CTX, Model> {
    fn create(ctx: CTX, tenant: TenantId, model: Model) -> Result<Model, OperationOutcomeError>;
    fn read(ctx: CTX, tenant: TenantId, id: String) -> Result<Model, OperationOutcomeError>;
    fn update(ctx: CTX, tenant: TenantId, model: Model) -> Result<Model, OperationOutcomeError>;
    fn delete(ctx: CTX, tenant: TenantId, id: String) -> Result<(), OperationOutcomeError>;
    fn search(ctx: CTX, tenant: TenantId) -> Result<Vec<Model>, OperationOutcomeError>;
}

pub trait ProjectAuthAdmin<CTX, Model> {
    fn create(
        ctx: CTX,
        tenant: TenantId,
        project: ProjectId,
        model: Model,
    ) -> Result<Model, OperationOutcomeError>;
    fn read(
        ctx: CTX,
        tenant: TenantId,
        project: ProjectId,
        id: String,
    ) -> Result<Model, OperationOutcomeError>;
    fn update(
        ctx: CTX,
        tenant: TenantId,
        project: ProjectId,
        model: Model,
    ) -> Result<Model, OperationOutcomeError>;
    fn delete(
        ctx: CTX,
        tenant: TenantId,
        project: ProjectId,
        id: String,
    ) -> Result<(), OperationOutcomeError>;
    fn search(
        ctx: CTX,
        tenant: TenantId,
        project: ProjectId,
    ) -> Result<Vec<Model>, OperationOutcomeError>;
}
