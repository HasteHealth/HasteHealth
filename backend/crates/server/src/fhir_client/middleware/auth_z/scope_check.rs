use crate::fhir_client::{
    ServerCTX,
    batch_transaction_processing::get_resource_type_from_fhir_request,
    middleware::{
        ServerMiddlewareContext, ServerMiddlewareNext, ServerMiddlewareOutput,
        ServerMiddlewareState,
    },
};

use haste_fhir_client::{
    middleware::MiddlewareChain,
    request::{FHIRRequest, FHIRResponse},
};
use haste_fhir_model::r4::generated::{resources::ResourceType, terminology::IssueType};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_jwt::scopes::{
    SMARTResourceScope, Scope, Scopes, SmartResourceScopeLevel, SmartResourceScopePermission,
    SmartResourceScopeUser, SmartScope,
};
use haste_repository::Repository;
use std::sync::Arc;

fn request_type_to_permission(
    request: &FHIRRequest,
) -> Result<SmartResourceScopePermission, OperationOutcomeError> {
    match request {
        FHIRRequest::Capabilities
        | FHIRRequest::Batch(_)
        | FHIRRequest::Transaction(_)
        | FHIRRequest::InvokeInstance(_)
        | FHIRRequest::InvokeType(_)
        | FHIRRequest::InvokeSystem(_) => Err(OperationOutcomeError::fatal(
            IssueType::Exception(None),
            "Cannot determine permission for this request type".to_string(),
        )),
        FHIRRequest::Create(_) => Ok(SmartResourceScopePermission::Create),

        FHIRRequest::Read(_) | FHIRRequest::VersionRead(_) => {
            Ok(SmartResourceScopePermission::Read)
        }

        FHIRRequest::UpdateInstance(_)
        | FHIRRequest::ConditionalUpdate(_)
        | FHIRRequest::Patch(_) => Ok(SmartResourceScopePermission::Update),

        FHIRRequest::DeleteInstance(_)
        | FHIRRequest::DeleteType(_)
        | FHIRRequest::DeleteSystem(_) => Ok(SmartResourceScopePermission::Delete),

        FHIRRequest::HistoryInstance(_)
        | FHIRRequest::HistoryType(_)
        | FHIRRequest::HistorySystem(_)
        | FHIRRequest::SearchType(_)
        | FHIRRequest::SearchSystem(_) => Ok(SmartResourceScopePermission::Search),
    }
}

fn fits_resource_type(
    scope: &SMARTResourceScope,
    request_resource_type: Option<&ResourceType>,
) -> bool {
    match &scope.level {
        SmartResourceScopeLevel::AllResources => true,
        SmartResourceScopeLevel::ResourceType(scope_resource_type) => {
            Some(scope_resource_type) == request_resource_type
        }
    }
}

fn get_user_weight_scope(user: &SmartResourceScopeUser) -> u8 {
    match user {
        SmartResourceScopeUser::Patient => 1,
        SmartResourceScopeUser::User => 2,
        SmartResourceScopeUser::System => 3,
    }
}

fn get_highest_value_for_request_scope<'a>(
    scopes: &'a Scopes,
    request: &FHIRRequest,
) -> Result<Option<&'a SMARTResourceScope>, OperationOutcomeError> {
    let request_scope_requested = request_type_to_permission(request)?;
    let request_resource_type = get_resource_type_from_fhir_request(request);

    let found_scopes = scopes
        .0
        .iter()
        .filter_map(|s| match s {
            Scope::SMART(SmartScope::Resource(scope)) => Some(scope),
            _ => None,
        })
        .filter(|s| {
            fits_resource_type(s, request_resource_type.as_ref())
                && s.permissions.has_permission(&request_scope_requested)
        })
        .collect::<Vec<_>>();

    // Sort by level weight if for example system scope grants permission and so does a patient scope.
    // Than system scope should take precedence.
    let mut sorted_scopes = found_scopes;
    sorted_scopes.sort_by(|a, b| {
        let a_weight = get_user_weight_scope(&a.user);
        let b_weight = get_user_weight_scope(&b.user);

        b_weight.cmp(&a_weight)
    });

    Ok(sorted_scopes.first().map(|s| *s))
}

pub struct SMARTScopeAccessMiddleware {}
impl SMARTScopeAccessMiddleware {
    pub fn new() -> Self {
        Self {}
    }
}
impl<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
>
    MiddlewareChain<
        ServerMiddlewareState<Repo, Search, Terminology>,
        Arc<ServerCTX<Repo, Search, Terminology>>,
        FHIRRequest,
        FHIRResponse,
        OperationOutcomeError,
    > for SMARTScopeAccessMiddleware
{
    fn call(
        &self,
        state: ServerMiddlewareState<Repo, Search, Terminology>,
        context: ServerMiddlewareContext<Repo, Search, Terminology>,
        next: Option<Arc<ServerMiddlewareNext<Repo, Search, Terminology>>>,
    ) -> ServerMiddlewareOutput<Repo, Search, Terminology> {
        Box::pin(async move {
            match &context.request {
                // Batch and transaction will call back into this middleware for their individual requests
                // at which point the permissions will be checked.
                FHIRRequest::Capabilities
                | FHIRRequest::Batch(_)
                | FHIRRequest::Transaction(_)
                | FHIRRequest::InvokeInstance(_)
                | FHIRRequest::InvokeType(_)
                | FHIRRequest::InvokeSystem(_) => {
                    if let Some(next) = next {
                        Ok(next(state, context).await?)
                    } else {
                        Ok(context)
                    }
                }
                FHIRRequest::Create(_)
                | FHIRRequest::Read(_)
                | FHIRRequest::VersionRead(_)
                | FHIRRequest::UpdateInstance(_)
                | FHIRRequest::ConditionalUpdate(_)
                | FHIRRequest::Patch(_)
                | FHIRRequest::DeleteInstance(_)
                | FHIRRequest::DeleteType(_)
                | FHIRRequest::DeleteSystem(_)
                | FHIRRequest::SearchType(_)
                | FHIRRequest::SearchSystem(_)
                | FHIRRequest::HistoryInstance(_)
                | FHIRRequest::HistoryType(_)
                | FHIRRequest::HistorySystem(_) => {
                    let user_scopes = &context.ctx.user.scope;

                    let matched_scope =
                        get_highest_value_for_request_scope(user_scopes, &context.request)?;

                    if let Some(matched_scope) = matched_scope
                        && matched_scope.user == SmartResourceScopeUser::Patient
                    {
                        return Err(OperationOutcomeError::error(
                            IssueType::Security(None),
                            "Patient-level SMART scopes are not supported for this request"
                                .to_string(),
                        ));
                    }

                    match matched_scope {
                        Some(_scope) => {
                            // Permission granted
                            if let Some(next) = next {
                                Ok(next(state, context).await?)
                            } else {
                                Ok(context)
                            }
                        }
                        None => {
                            // No matching scope found, deny access
                            Err(OperationOutcomeError::error(
                                IssueType::Security(None),
                                "Insufficient SMART scope for this request".to_string(),
                            ))
                        }
                    }
                }
            }
        })
    }
}
