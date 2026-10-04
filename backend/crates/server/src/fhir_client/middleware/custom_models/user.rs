use crate::fhir_client::{
    ServerCTX,
    middleware::{
        ServerMiddlewareContext, ServerMiddlewareNext, ServerMiddlewareOutput,
        ServerMiddlewareState,
    },
    utilities::request_to_resource_type,
};
use haste_fhir_client::{
    FHIRClient,
    middleware::MiddlewareChain,
    request::{DeleteRequest, DeleteResponse, FHIRRequest, FHIRResponse, UpdateRequest},
};
use haste_fhir_model::r4::generated::{
    resources::{Resource, ResourceType, User},
    terminology::IssueType,
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_jwt::{AuthorKind, TenantId, UserRole, claims::UserTokenClaims};
use haste_repository::{
    Repository,
    admin::TenantModelAdmin,
    types::user::{AuthMethod, CreateUser, UpdateUser, UserRole as RepoUserRole},
};
use std::sync::Arc;

/// The role hierarchy for `User` writes: the roles each caller may modify.
fn modifiable_roles(caller: &UserRole) -> &'static [RepoUserRole] {
    match caller {
        UserRole::Owner => &[
            RepoUserRole::Owner,
            RepoUserRole::Admin,
            RepoUserRole::Member,
        ],
        UserRole::Admin => &[RepoUserRole::Admin, RepoUserRole::Member],
        UserRole::Member => &[],
    }
}

/// The role a `User` resource would be stored with.
fn written_role(resource: &Resource) -> Option<RepoUserRole> {
    match resource {
        Resource::User(user) => Some(user.role.clone().into()),
        _ => None,
    }
}

/// The role of the stored user `id`, if there is one.
async fn stored_role<Repo: Repository + Send + Sync>(
    repo: &Repo,
    tenant: &TenantId,
    id: &String,
) -> Result<Option<RepoUserRole>, OperationOutcomeError> {
    Ok(
        TenantModelAdmin::<CreateUser, _, _, _, _>::read(repo, tenant, id)
            .await?
            .map(|user| user.role),
    )
}

/// Enforces the role hierarchy on `User` writes. The caller must be allowed to
/// modify both the role being written and the role the stored user holds, so
/// an admin (who bypasses access policies) cannot mint, demote or delete an
/// owner and take the tenant over.
async fn guard_role_hierarchy<Repo: Repository + Send + Sync>(
    repo: &Repo,
    tenant: &TenantId,
    claims: &UserTokenClaims,
    request: &FHIRRequest,
) -> Result<(), OperationOutcomeError> {
    // The server's own writes carry the owner role. A client application is
    // not a user and the member role in its token is only a placeholder, so
    // it ranks as an admin and its access policies decide the rest.
    let caller = match claims.resource_type {
        AuthorKind::ClientApplication => &UserRole::Admin,
        _ => &claims.user_role,
    };

    let (written, stored) = match request {
        FHIRRequest::Create(create) => (written_role(&create.resource), None),
        FHIRRequest::Update(UpdateRequest::Instance(update)) => (
            written_role(&update.resource),
            stored_role(repo, tenant, &update.id).await?,
        ),
        FHIRRequest::Delete(DeleteRequest::Instance(delete)) => {
            (None, stored_role(repo, tenant, &delete.id).await?)
        }
        // A write addressed by search could reach any user, an owner included.
        FHIRRequest::Update(UpdateRequest::Conditional(_))
        | FHIRRequest::Delete(DeleteRequest::Type(_)) => (None, Some(RepoUserRole::Owner)),
        _ => (None, None),
    };

    let modifiable = modifiable_roles(caller);
    if written
        .iter()
        .chain(&stored)
        .all(|role| modifiable.contains(role))
    {
        Ok(())
    } else {
        Err(OperationOutcomeError::error(
            IssueType::forbidden(),
            "Your role may not make this change: owners may modify any user, admins may modify \
             admins and members, and a conditional update or delete needs an owner."
                .to_string(),
        ))
    }
}

fn get_provider_id(user: &User) -> Option<String> {
    user.federated
        .as_ref()
        .and_then(|f| f.reference.as_ref())
        .and_then(|r| r.value.as_ref())
        .and_then(|s| s.split('/').next_back().map(|s| s.to_string()))
}

fn get_user_method(user: &User) -> AuthMethod {
    match get_provider_id(user) {
        Some(_) => AuthMethod::OIDC,
        None => AuthMethod::EmailPassword,
    }
}

pub struct Middleware {}
impl Middleware {
    pub fn new() -> Self {
        Middleware {}
    }
}
impl<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
    Client: FHIRClient<Arc<ServerCTX<Client>>, OperationOutcomeError> + 'static,
>
    MiddlewareChain<
        ServerMiddlewareState<Repo, Search, Terminology>,
        Arc<ServerCTX<Client>>,
        FHIRRequest,
        FHIRResponse,
        OperationOutcomeError,
    > for Middleware
{
    fn call(
        &self,
        state: ServerMiddlewareState<Repo, Search, Terminology>,
        context: ServerMiddlewareContext<Client>,
        next: Option<
            Arc<ServerMiddlewareNext<Client, ServerMiddlewareState<Repo, Search, Terminology>>>,
        >,
    ) -> ServerMiddlewareOutput<Client> {
        Box::pin(async move {
            if let Some(next) = next {
                if request_to_resource_type(&context.request)
                    .is_some_and(|resource_type| *resource_type == ResourceType::User)
                {
                    guard_role_hierarchy(
                        state.repo.as_ref(),
                        &context.ctx.tenant,
                        &context.ctx.user.claims,
                        &context.request,
                    )
                    .await?;
                }

                let res = next(state.clone(), context).await?;
                if let Some(resource_type) = request_to_resource_type(&res.request)
                    && *resource_type != ResourceType::User
                {
                    Ok(res)
                } else {
                    match res.response.as_ref() {
                        Some(FHIRResponse::Create(create_response)) => {
                            if let Resource::User(user) = &create_response.resource
                                && let Some(id) = user.id.as_ref()
                            {
                                TenantModelAdmin::create(
                                    state.repo.as_ref(),
                                    &res.ctx.tenant,
                                    CreateUser {
                                        id: id.clone(),
                                        email: user.email.clone().and_then(|e| e.value),
                                        role: user.role.clone().into(),
                                        method: get_user_method(user),
                                        provider_id: get_provider_id(user),
                                        password: None,
                                    },
                                )
                                .await?;

                                Ok(res)
                            } else {
                                Err(OperationOutcomeError::fatal(
                                    IssueType::invalid(),
                                    "User resource is invalid.".to_string(),
                                ))
                            }
                        }
                        Some(FHIRResponse::Delete(DeleteResponse::Instance(delete_response))) => {
                            if let Resource::User(user) = &delete_response.resource
                                && let Some(id) = user.id.as_ref()
                            {
                                TenantModelAdmin::<CreateUser, _, _, _, _>::delete(
                                    state.repo.as_ref(),
                                    &res.ctx.tenant,
                                    id,
                                )
                                .await?;

                                Ok(res)
                            } else {
                                Err(OperationOutcomeError::fatal(
                                    IssueType::invalid(),
                                    "User resource is invalid.".to_string(),
                                ))
                            }
                        }
                        Some(FHIRResponse::Update(update_response)) => {
                            if let Resource::User(user) = &update_response.resource
                                && let Some(id) = user.id.as_ref()
                            {
                                TenantModelAdmin::<CreateUser, _, _, _, _>::update(
                                    state.repo.as_ref(),
                                    &res.ctx.tenant,
                                    UpdateUser {
                                        id: id.clone(),
                                        email: user.email.clone().and_then(|e| e.value),
                                        role: Some(user.role.clone().into()),
                                        method: Some(get_user_method(user)),
                                        provider_id: get_provider_id(user),
                                        password: None,
                                    },
                                )
                                .await?;

                                Ok(res)
                            } else {
                                Err(OperationOutcomeError::fatal(
                                    IssueType::invalid(),
                                    "User resource is invalid.".to_string(),
                                ))
                            }
                        }

                        _ => Ok(res),
                    }
                }
            } else {
                Err(OperationOutcomeError::fatal(
                    IssueType::exception(),
                    "No next middleware found".to_string(),
                ))
            }
        })
    }
}
