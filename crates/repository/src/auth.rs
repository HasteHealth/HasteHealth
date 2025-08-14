/// Authentication traits include management for user and Authorization codes.
use crate::{ProjectId, TenantId, UserRole};
use oxidized_fhir_operation_error::OperationOutcomeError;

pub struct User {
    pub fhir_user_id: String,
    pub email: String,
    pub role: UserRole,
}

pub enum TenantModels {
    User(User),
    Tenant {
        id: String,
        name: String,
    },
    Project {
        id: String,
        name: String,
        description: String,
    },
}

pub enum LoginMethod {
    OIDC { email: String, provider_id: String },
    EmailPassword { email: String, password: String },
}

pub enum LoginResult {
    Success { user: User },
}

pub trait Login<CTX> {
    fn login(
        &self,
        ctx: CTX,
        tenant: &TenantId,
        method: &LoginMethod,
    ) -> impl Future<Output = Result<LoginResult, OperationOutcomeError>> + Send;
}

pub enum ProjectModels {}

pub trait TenantAuthAdmin<CTX, CreatedModel, ReadModel, SearchClauses> {
    fn create(
        ctx: CTX,
        tenant: TenantId,
        model: CreatedModel,
    ) -> Result<ReadModel, OperationOutcomeError>;
    fn read(ctx: CTX, tenant: TenantId, id: String) -> Result<ReadModel, OperationOutcomeError>;
    fn update(
        ctx: CTX,
        tenant: TenantId,
        model: ReadModel,
    ) -> Result<ReadModel, OperationOutcomeError>;
    fn delete(ctx: CTX, tenant: TenantId, id: String) -> Result<(), OperationOutcomeError>;
    fn search(
        ctx: CTX,
        tenant: TenantId,
        clauses: SearchClauses,
    ) -> Result<Vec<ReadModel>, OperationOutcomeError>;
}

pub trait ProjectAuthAdmin<CTX, CreatedModel, ReadModel, SearchClauses> {
    fn create(
        ctx: CTX,
        tenant: TenantId,
        project: ProjectId,
        model: CreatedModel,
    ) -> Result<ReadModel, OperationOutcomeError>;
    fn read(
        ctx: CTX,
        tenant: TenantId,
        project: ProjectId,
        id: String,
    ) -> Result<ReadModel, OperationOutcomeError>;
    fn update(
        ctx: CTX,
        tenant: TenantId,
        project: ProjectId,
        model: ReadModel,
    ) -> Result<ReadModel, OperationOutcomeError>;
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
        clauses: SearchClauses,
    ) -> Result<Vec<ReadModel>, OperationOutcomeError>;
}
