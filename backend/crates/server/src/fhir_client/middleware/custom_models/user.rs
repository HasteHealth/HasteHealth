use crate::fhir_client::{
    ClientState, ServerCTX,
    middleware::{
        ServerMiddlewareContext, ServerMiddlewareNext, ServerMiddlewareOutput,
        ServerMiddlewareState,
    },
};
use oxidized_fhir_client::{
    middleware::MiddlewareChain,
    request::{FHIRRequest, FHIRResponse},
};
use oxidized_fhir_model::r4::generated::{
    resources::{Resource, User},
    terminology::IssueType,
};
use oxidized_fhir_operation_error::OperationOutcomeError;
use oxidized_fhir_search::SearchEngine;
use oxidized_fhir_terminology::FHIRTerminology;
use oxidized_repository::{
    Repository,
    admin::TenantAuthAdmin,
    types::user::{AuthMethod, CreateUser, UpdateUser},
};
use std::sync::Arc;

fn get_provider_id(user: &User) -> Option<String> {
    user.federated
        .as_ref()
        .and_then(|f| f.reference.as_ref())
        .and_then(|r| r.value.as_ref())
        .and_then(|s| s.split('/').last().map(|s| s.to_string()))
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
>
    MiddlewareChain<
        ServerMiddlewareState<Repo, Search, Terminology>,
        Arc<ServerCTX>,
        FHIRRequest,
        FHIRResponse,
        OperationOutcomeError,
    > for Middleware
{
    fn call(
        &self,
        state: ServerMiddlewareState<Repo, Search, Terminology>,
        context: ServerMiddlewareContext,
        next: Option<Arc<ServerMiddlewareNext<Repo, Search, Terminology>>>,
    ) -> ServerMiddlewareOutput {
        Box::pin(async move {
            if let Some(next) = next {
                let res = next(state.clone(), context).await?;

                match res.response.as_ref() {
                    Some(FHIRResponse::Create(create_response)) => {
                        if let Resource::User(user) = &create_response.resource
                            && let Some(email) = user.email.value.as_ref()
                            && let Some(id) = user.id.as_ref()
                        {
                            TenantAuthAdmin::create(
                                state.repo.as_ref(),
                                &res.ctx.tenant,
                                CreateUser {
                                    id: id.clone(),
                                    email: email.clone(),
                                    role: (*user.role).clone().into(),
                                    method: get_user_method(user),
                                    provider_id: get_provider_id(user),
                                    password: None,
                                },
                            )
                            .await?;

                            Ok(res)
                        } else {
                            Err(OperationOutcomeError::fatal(
                                IssueType::Invalid(None),
                                "User resource is invalid.".to_string(),
                            ))
                        }
                    }
                    Some(FHIRResponse::DeleteInstance(delete_response)) => {
                        if let Resource::User(user) = &delete_response.resource
                            && let Some(id) = user.id.as_ref()
                        {
                            TenantAuthAdmin::<CreateUser, _, _, _>::delete(
                                state.repo.as_ref(),
                                &res.ctx.tenant,
                                id,
                            )
                            .await?;

                            Ok(res)
                        } else {
                            Err(OperationOutcomeError::fatal(
                                IssueType::Invalid(None),
                                "User resource is invalid.".to_string(),
                            ))
                        }
                    }
                    Some(FHIRResponse::Update(update_response)) => {
                        if let Resource::User(user) = &update_response.resource
                            && let Some(email) = user.email.value.as_ref()
                            && let Some(id) = user.id.as_ref()
                        {
                            TenantAuthAdmin::<CreateUser, _, _, _>::update(
                                state.repo.as_ref(),
                                &res.ctx.tenant,
                                UpdateUser {
                                    id: id.clone(),
                                    email: Some(email.clone()),
                                    role: Some((*user.role).clone().into()),
                                    method: Some(get_user_method(user)),
                                    provider_id: get_provider_id(user),
                                    password: None,
                                },
                            )
                            .await?;

                            Ok(res)
                        } else {
                            Err(OperationOutcomeError::fatal(
                                IssueType::Invalid(None),
                                "User resource is invalid.".to_string(),
                            ))
                        }
                    }

                    _ => Ok(res),
                }
            } else {
                Err(OperationOutcomeError::fatal(
                    IssueType::Exception(None),
                    "No next middleware found".to_string(),
                ))
            }
        })
    }
}
