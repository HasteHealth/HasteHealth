use crate::fhir_client::{
    ServerCTX,
    middleware::{ServerMiddlewareContext, ServerMiddlewareNext, ServerMiddlewareOutput},
    utilities::request_to_resource_type,
};

use haste_fhir_client::{
    FHIRClient,
    middleware::MiddlewareChain,
    request::{FHIRRequest, FHIRResponse},
};
use haste_fhir_model::r4::generated::{resources::ResourceType, terminology::IssueType};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_jwt::scopes::{
    SMARTResourceScope, Scope, Scopes, SmartResourceScopeLevel, SmartResourceScopePermission,
    SmartResourceScopeUser, SmartScope,
};
use std::sync::Arc;

fn request_type_to_permission(
    request: &FHIRRequest,
) -> Result<SmartResourceScopePermission, OperationOutcomeError> {
    match request {
        FHIRRequest::Capabilities | FHIRRequest::Batch(_) | FHIRRequest::Transaction(_) => {
            Err(OperationOutcomeError::fatal(
                IssueType::exception(),
                "Cannot determine permission for this request type".to_string(),
            ))
        }

        // Operations are checked separately: the permission an invocation needs
        // depends on the OperationDefinition, not on the request shape.
        FHIRRequest::Invocation(_) => Err(OperationOutcomeError::fatal(
            IssueType::exception(),
            "Operation invocations are authorized by invocation_permitted".to_string(),
        )),
        FHIRRequest::Create(_) => Ok(SmartResourceScopePermission::Create),

        FHIRRequest::Read(_) | FHIRRequest::VersionRead(_) => {
            Ok(SmartResourceScopePermission::Read)
        }

        FHIRRequest::Update(_) | FHIRRequest::Patch(_) => Ok(SmartResourceScopePermission::Update),

        FHIRRequest::Delete(_) => Ok(SmartResourceScopePermission::Delete),

        FHIRRequest::Search(_) | FHIRRequest::History(_) => {
            Ok(SmartResourceScopePermission::Search)
        }

        FHIRRequest::Compartment(compartment_request) => {
            request_type_to_permission(&compartment_request.request)
        }
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
    let request_resource_type = request_to_resource_type(request);

    let found_scopes = scopes
        .0
        .iter()
        .filter_map(|s| match s {
            Scope::SMART(SmartScope::Resource(scope)) => Some(scope),
            _ => None,
        })
        .filter(|s| {
            fits_resource_type(s, request_resource_type)
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

    Ok(sorted_scopes.first().copied())
}

/// Whether any granted scope permits invoking an operation.
///
/// The permission an operation needs is declared by its `OperationDefinition`
/// (`affectsState`), which this middleware does not load -- resolving it here
/// would mean a repository read on the authorization path for every invocation.
/// Instead we require that the caller holds *some* resource-level scope covering
/// the operation's target, and let the operation's own FHIR reads and writes be
/// scope-checked as they pass back through this chain. That keeps a token with no
/// FHIR access from reaching operations at all, while the per-interaction checks
/// remain authoritative for whatever the operation actually touches.
fn invocation_permitted(scopes: &Scopes, request: &FHIRRequest) -> bool {
    let request_resource_type = request_to_resource_type(request);

    scopes.0.iter().any(|scope| {
        let Scope::SMART(SmartScope::Resource(resource_scope)) = scope else {
            return false;
        };

        // Patient-level scopes are rejected for every other request type here.
        if resource_scope.user == SmartResourceScopeUser::Patient {
            return false;
        }

        match request_resource_type {
            // A system-level operation has no target type, so holding any
            // resource scope is enough to reach it.
            None => true,
            Some(_) => fits_resource_type(resource_scope, request_resource_type),
        }
    })
}

pub struct SMARTScopeAccessMiddleware {}
impl SMARTScopeAccessMiddleware {
    pub fn new() -> Self {
        Self {}
    }
}
impl<
    State: Send + Sync + 'static,
    Client: FHIRClient<Arc<ServerCTX<Client>>, OperationOutcomeError> + 'static,
> MiddlewareChain<State, Arc<ServerCTX<Client>>, FHIRRequest, FHIRResponse, OperationOutcomeError>
    for SMARTScopeAccessMiddleware
{
    fn call(
        &self,
        state: State,
        context: ServerMiddlewareContext<Client>,
        next: Option<Arc<ServerMiddlewareNext<Client, State>>>,
    ) -> ServerMiddlewareOutput<Client> {
        Box::pin(async move {
            match &context.request {
                // Batch and transaction will call back into this middleware for their individual requests
                // at which point the permissions will be checked.
                FHIRRequest::Capabilities | FHIRRequest::Batch(_) | FHIRRequest::Transaction(_) => {
                    if let Some(next) = next {
                        Ok(next(state, context).await?)
                    } else {
                        Ok(context)
                    }
                }
                FHIRRequest::Invocation(_) => {
                    if !invocation_permitted(&context.ctx.user.claims.scope, &context.request) {
                        return Err(OperationOutcomeError::error(
                            IssueType::security(),
                            "Insufficient SMART scope to invoke this operation".to_string(),
                        ));
                    }

                    if let Some(next) = next {
                        Ok(next(state, context).await?)
                    } else {
                        Ok(context)
                    }
                }
                FHIRRequest::Compartment(_)
                | FHIRRequest::Create(_)
                | FHIRRequest::Read(_)
                | FHIRRequest::VersionRead(_)
                | FHIRRequest::Update(_)
                | FHIRRequest::Patch(_)
                | FHIRRequest::Delete(_)
                | FHIRRequest::Search(_)
                | FHIRRequest::History(_) => {
                    let user_scopes = &context.ctx.user.claims.scope;

                    let matched_scope =
                        get_highest_value_for_request_scope(user_scopes, &context.request)?;

                    if let Some(matched_scope) = matched_scope
                        && matched_scope.user == SmartResourceScopeUser::Patient
                    {
                        return Err(OperationOutcomeError::error(
                            IssueType::security(),
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
                                IssueType::security(),
                                "Insufficient SMART scope for this request".to_string(),
                            ))
                        }
                    }
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use haste_fhir_client::request::{FHIRInvokeSystemRequest, FHIRInvokeTypeRequest, Operation};
    use haste_fhir_model::r4::generated::resources::Parameters;

    fn scopes(raw: &str) -> Scopes {
        Scopes::try_from(raw).expect("valid scopes")
    }

    fn type_invocation(resource_type: ResourceType, code: &str) -> FHIRRequest {
        FHIRRequest::Invocation(haste_fhir_client::request::InvocationRequest::Type(
            FHIRInvokeTypeRequest {
                operation: Operation::new(code),
                resource_type,
                parameters: Parameters::default(),
            },
        ))
    }

    fn system_invocation(code: &str) -> FHIRRequest {
        FHIRRequest::Invocation(haste_fhir_client::request::InvocationRequest::System(
            FHIRInvokeSystemRequest {
                operation: Operation::new(code),
                parameters: Parameters::default(),
            },
        ))
    }

    #[test]
    fn invocation_requires_a_scope_on_the_target_type() {
        let request = type_invocation(ResourceType::ViewDefinition, "viewdefinition-run");

        assert!(invocation_permitted(
            &scopes("user/ViewDefinition.rs"),
            &request
        ));
        // A scope on some other resource type does not reach this operation.
        assert!(!invocation_permitted(&scopes("user/Patient.rs"), &request));
    }

    #[test]
    fn wildcard_scopes_permit_invocation() {
        let request = type_invocation(ResourceType::Patient, "everything");
        assert!(invocation_permitted(&scopes("system/*.cruds"), &request));
    }

    #[test]
    fn a_token_without_resource_scopes_cannot_invoke() {
        // Previously every invocation bypassed this middleware entirely.
        let request = system_invocation("current-project");
        assert!(!invocation_permitted(&scopes("openid profile"), &request));
    }

    #[test]
    fn patient_level_scopes_do_not_permit_invocation() {
        let request = type_invocation(ResourceType::Patient, "everything");
        assert!(!invocation_permitted(
            &scopes("patient/Patient.rs"),
            &request
        ));
    }

    #[test]
    fn system_level_invocation_accepts_any_resource_scope() {
        // System operations have no target type, so any resource scope qualifies.
        let request = system_invocation("current-project");
        assert!(invocation_permitted(&scopes("user/Patient.rs"), &request));
    }
}
