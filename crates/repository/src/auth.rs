/// Authentication traits include management for user and Authorization codes.
use crate::{AuthMethod, ProjectId, TenantId, UserRole};
use oxidized_fhir_operation_error::OperationOutcomeError;

pub struct User {
    pub id: String,
    pub email: String,
    pub role: UserRole,
    pub method: AuthMethod,
    pub provider_id: Option<String>,
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
        &self,
        ctx: CTX,
        tenant: TenantId,
        model: CreatedModel,
    ) -> impl Future<Output = Result<ReadModel, OperationOutcomeError>> + Send;
    fn read(
        &self,
        ctx: CTX,
        tenant: TenantId,
        id: String,
    ) -> impl Future<Output = Result<ReadModel, OperationOutcomeError>> + Send;
    fn update(
        &self,
        ctx: CTX,
        tenant: TenantId,
        model: ReadModel,
    ) -> impl Future<Output = Result<ReadModel, OperationOutcomeError>> + Send;
    fn delete(
        &self,
        ctx: CTX,
        tenant: TenantId,
        id: String,
    ) -> impl Future<Output = Result<(), OperationOutcomeError>> + Send;
    fn search(
        &self,
        ctx: CTX,
        tenant: TenantId,
        clauses: SearchClauses,
    ) -> impl Future<Output = Result<Vec<ReadModel>, OperationOutcomeError>> + Send;
}

pub trait ProjectAuthAdmin<CTX, CreatedModel, ReadModel, SearchClauses> {
    fn create(
        &self,
        ctx: CTX,
        tenant: TenantId,
        project: ProjectId,
        model: CreatedModel,
    ) -> impl Future<Output = Result<ReadModel, OperationOutcomeError>> + Send;
    fn read(
        &self,
        ctx: CTX,
        tenant: TenantId,
        project: ProjectId,
        id: String,
    ) -> impl Future<Output = Result<ReadModel, OperationOutcomeError>> + Send;
    fn update(
        &self,
        ctx: CTX,
        tenant: TenantId,
        project: ProjectId,
        model: ReadModel,
    ) -> impl Future<Output = Result<ReadModel, OperationOutcomeError>> + Send;
    fn delete(
        &self,
        ctx: CTX,
        tenant: TenantId,
        project: ProjectId,
        id: String,
    ) -> impl Future<Output = Result<(), OperationOutcomeError>> + Send;
    fn search(
        &self,
        ctx: CTX,
        tenant: TenantId,
        project: ProjectId,
        clauses: SearchClauses,
    ) -> impl Future<Output = Result<Vec<ReadModel>, OperationOutcomeError>> + Send;
}
