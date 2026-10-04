use crate::{
    auth_n::invitations::{resource_email_verified, send_invitation, set_resource_email_verified},
    fhir_client::{
        ServerCTX,
        middleware::{
            ServerMiddlewareContext, ServerMiddlewareNext, ServerMiddlewareOutput,
            ServerMiddlewareState,
        },
        utilities::request_to_resource_type,
    },
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
    types::user::{
        AuthMethod, CreateUser, UpdateUser, User as StoredUser, UserRole as RepoUserRole,
    },
};
use std::sync::Arc;

/// The roles each caller may modify.
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

/// The stored user a write names by id. A conditional update names one only
/// when its body has an id.
async fn stored_user<Repo: Repository>(
    repo: &Repo,
    tenant: &TenantId,
    request: &FHIRRequest,
) -> Result<Option<StoredUser>, OperationOutcomeError> {
    let id = match request {
        FHIRRequest::Update(UpdateRequest::Instance(update)) => &update.id,
        FHIRRequest::Delete(DeleteRequest::Instance(delete)) => &delete.id,
        FHIRRequest::Update(UpdateRequest::Conditional(update)) => match &update.resource {
            Resource::User(User { id: Some(id), .. }) => id,
            _ => return Ok(None),
        },
        _ => return Ok(None),
    };

    TenantModelAdmin::<CreateUser, _, _, _, _>::read(repo, tenant, id).await
}

/// The caller must be allowed to modify both the role written and the role
/// the stored user holds, so an admin cannot mint, demote or delete an owner.
fn guard_role_hierarchy(
    claims: &UserTokenClaims,
    request: &FHIRRequest,
    stored: Option<&StoredUser>,
) -> Result<(), OperationOutcomeError> {
    // The server's own writes carry the owner role. A client application's
    // token role is a placeholder: it ranks as an admin.
    let caller = match claims.resource_type {
        AuthorKind::ClientApplication => &UserRole::Admin,
        _ => &claims.user_role,
    };

    let stored = stored.map(|user| &user.role);
    let (written, stored) = match request {
        FHIRRequest::Create(create) => (written_role(&create.resource), None),
        FHIRRequest::Update(UpdateRequest::Instance(update)) => {
            (written_role(&update.resource), stored)
        }
        FHIRRequest::Delete(DeleteRequest::Instance(_)) => (None, stored),
        // A write addressed by search could reach any user, an owner included.
        FHIRRequest::Update(UpdateRequest::Conditional(_))
        | FHIRRequest::Delete(DeleteRequest::Type(_)) => (None, Some(&RepoUserRole::Owner)),
        _ => (None, None),
    };

    let modifiable = modifiable_roles(caller);
    if written
        .iter()
        .chain(stored)
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

/// Only the server sets `emailVerified`. A client's write keeps the stored
/// value, or false for a new user.
fn constrain_email_verified(request: &mut FHIRRequest, stored: Option<&StoredUser>) {
    let resource = match request {
        FHIRRequest::Create(create) => &mut create.resource,
        FHIRRequest::Update(UpdateRequest::Instance(update)) => &mut update.resource,
        FHIRRequest::Update(UpdateRequest::Conditional(update)) => &mut update.resource,
        _ => return,
    };

    if let Resource::User(user) = resource {
        set_resource_email_verified(user, stored.is_some_and(|stored| stored.email_verified));
    }
}

/// A client cannot change a stored user's email or sign-in method. The
/// acceptance, password and emailed links belong to that address, and an
/// identity provider vouching for an address is not the address accepting.
async fn guard_identity<Repo: Repository>(
    repo: &Repo,
    tenant: &TenantId,
    id: &String,
    written: &User,
) -> Result<(), OperationOutcomeError> {
    let Some(stored) = TenantModelAdmin::<CreateUser, _, _, _, _>::read(repo, tenant, id).await?
    else {
        return Ok(());
    };

    let changed = if stored.email != written.email.clone().and_then(|e| e.value) {
        "email"
    } else if stored.method != get_user_method(written) {
        "sign-in method"
    } else {
        return Ok(());
    };

    Err(OperationOutcomeError::error(
        IssueType::forbidden(),
        format!("A user's {changed} cannot be changed. Delete the user and create a new one."),
    ))
}

/// The `User` in a response, with its id.
fn response_user(resource: &Resource) -> Result<(&User, &String), OperationOutcomeError> {
    if let Resource::User(user) = resource
        && let Some(id) = user.id.as_ref()
    {
        Ok((user, id))
    } else {
        Err(OperationOutcomeError::fatal(
            IssueType::invalid(),
            "User resource is invalid.".to_string(),
        ))
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
        mut context: ServerMiddlewareContext<Client>,
        next: Option<
            Arc<ServerMiddlewareNext<Client, ServerMiddlewareState<Repo, Search, Terminology>>>,
        >,
    ) -> ServerMiddlewareOutput<Client> {
        Box::pin(async move {
            let Some(next) = next else {
                return Err(OperationOutcomeError::fatal(
                    IssueType::exception(),
                    "No next middleware found".to_string(),
                ));
            };

            if request_to_resource_type(&context.request) != Some(&ResourceType::User) {
                return next(state, context).await;
            }

            let claims = &context.ctx.user.claims;
            let by_client = claims.resource_type != AuthorKind::System;
            let stored =
                stored_user(state.repo.as_ref(), &context.ctx.tenant, &context.request).await?;

            guard_role_hierarchy(claims, &context.request, stored.as_ref())?;
            if by_client {
                constrain_email_verified(&mut context.request, stored.as_ref());
            }

            let res = next(state.clone(), context).await?;

            match res.response.as_ref() {
                Some(FHIRResponse::Create(create_response)) => {
                    let (user, id) = response_user(&create_response.resource)?;

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
                            email_verified: resource_email_verified(user),
                        },
                    )
                    .await?;

                    // A password user created by a client is an invitation.
                    if by_client && get_user_method(user) == AuthMethod::EmailPassword {
                        send_invitation(
                            state.repo.as_ref(),
                            state.config.as_ref(),
                            &res.ctx.tenant,
                            user,
                        )
                        .await;
                    }
                }
                Some(FHIRResponse::Delete(DeleteResponse::Instance(delete_response))) => {
                    let (_, id) = response_user(&delete_response.resource)?;

                    TenantModelAdmin::<CreateUser, _, _, _, _>::delete(
                        state.repo.as_ref(),
                        &res.ctx.tenant,
                        id,
                    )
                    .await?;
                }
                Some(FHIRResponse::Update(update_response)) => {
                    let (user, id) = response_user(&update_response.resource)?;

                    // Checked after the write, when even a conditional update
                    // shows which user it reached. An error rolls it back.
                    if by_client {
                        guard_identity(state.repo.as_ref(), &res.ctx.tenant, id, user).await?;
                    }

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
                            email_verified: Some(resource_email_verified(user)),
                        },
                    )
                    .await?;
                }
                _ => {}
            }

            Ok(res)
        })
    }
}
