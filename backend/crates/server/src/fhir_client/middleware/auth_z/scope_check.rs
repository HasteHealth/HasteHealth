//! SMART resource scope enforcement.
//!
//! Picks the scope that grants a request. A `system/` or `user/` grant reaches
//! every resource of the type. A `patient/` grant reaches one Patient
//! compartment: [`patient_scope`] checks the request, the context is confined
//! to the patient so storage filters its searches, and the response is checked
//! against the resources it holds.

use crate::fhir_client::{
    ServerCTX,
    middleware::{
        ServerMiddlewareContext, ServerMiddlewareNext, ServerMiddlewareOutput,
        auth_z::{forbidden, patient_scope},
    },
    utilities::{is_search_match, map_search_entries, request_to_resource_type},
};

use haste_fhir_client::{
    FHIRClient,
    middleware::{Context, MiddlewareChain},
    request::{FHIRRequest, FHIRResponse},
};
use haste_fhir_model::r4::generated::{
    resources::{BundleEntry, ResourceType},
    terminology::IssueType,
};
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
        FHIRRequest::Capabilities
        | FHIRRequest::Batch(_)
        | FHIRRequest::Transaction(_)
        | FHIRRequest::Invocation(_) => Err(OperationOutcomeError::fatal(
            IssueType::exception(),
            "Cannot determine permission for this request type".to_string(),
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

fn resource_scopes(scopes: &Scopes) -> impl Iterator<Item = &SMARTResourceScope> {
    scopes.0.iter().filter_map(|s| match s {
        Scope::SMART(SmartScope::Resource(scope)) => Some(scope),
        _ => None,
    })
}

fn get_highest_value_for_request_scope<'a>(
    scopes: &'a Scopes,
    request: &FHIRRequest,
) -> Result<Option<&'a SMARTResourceScope>, OperationOutcomeError> {
    let request_scope_requested = request_type_to_permission(request)?;
    let request_resource_type = request_to_resource_type(request);

    let found_scopes = resource_scopes(scopes)
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

/// The broadest level at which `scopes` grant `read` on `resource_type`.
fn read_scope_level<'a>(
    scopes: &'a Scopes,
    resource_type: &ResourceType,
) -> Option<&'a SmartResourceScopeUser> {
    resource_scopes(scopes)
        .filter(|scope| {
            fits_resource_type(scope, Some(resource_type))
                && scope
                    .permissions
                    .has_permission(&SmartResourceScopePermission::Read)
        })
        .map(|scope| &scope.user)
        .max_by_key(|user| get_user_weight_scope(user))
}

/// Whether every resource scope in the token is patient-level.
fn only_patient_scopes(scopes: &Scopes) -> bool {
    let mut scopes = resource_scopes(scopes).peekable();
    scopes.peek().is_some() && scopes.all(|scope| scope.user == SmartResourceScopeUser::Patient)
}

/// Keeps the matches, which the engine already filtered, and the includes the
/// caller's scopes allow: an include needs a read scope on its own type, and
/// at patient level it must be in the compartment of `patient`, the token's
/// patient.
async fn filter_entries(
    entries: Vec<BundleEntry>,
    scopes: &Scopes,
    patient: Option<&str>,
) -> Vec<BundleEntry> {
    let mut kept = Vec::with_capacity(entries.len());
    let mut includes = Vec::new();

    // Matches are already filtered in SQL. Includes need to be checked against the caller's scopes.
    for entry in entries {
        if is_search_match(&entry) {
            kept.push(entry);
        } else {
            includes.push(entry);
        }
    }

    // Includes linked by ID they need to have a pass through to confirm they are allowed by the caller's scopes.
    for entry in includes {
        let Some(resource) = entry.resource.as_deref() else {
            continue;
        };
        let allowed = match (read_scope_level(scopes, &resource.resource_type()), patient) {
            (None, _) | (Some(SmartResourceScopeUser::Patient), None) => false,
            (Some(SmartResourceScopeUser::Patient), Some(patient_id)) => {
                patient_scope::resource_in_scope(patient_id, resource).await
            }
            (Some(SmartResourceScopeUser::User | SmartResourceScopeUser::System), _) => true,
        };
        if allowed {
            kept.push(entry);
        }
    }

    kept
}

/// The response as the caller may see it: a read is checked against the
/// `compartment` the request was confined to, a search page's includes
/// against the caller's scopes.
async fn filter_response(
    response: FHIRResponse,
    scopes: &Scopes,
    patient: Option<&str>,
    compartment: Option<&str>,
) -> Option<FHIRResponse> {
    match (compartment, response) {
        (Some(patient_id), response @ (FHIRResponse::Read(_) | FHIRResponse::VersionRead(_))) => {
            patient_scope::filter_read(patient_id, response).await
        }
        (_, response) => Some(
            map_search_entries(response, |entries| filter_entries(entries, scopes, patient)).await,
        ),
    }
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
                // An operation's own data access comes back through this
                // client and is checked there.
                FHIRRequest::Invocation(invocation) => {
                    let claims = &context.ctx.user.claims;
                    if only_patient_scopes(&claims.scope)
                        && !claims.patient.as_deref().is_some_and(|patient_id| {
                            patient_scope::allows_invocation(patient_id, invocation)
                        })
                    {
                        return Err(forbidden(
                            "Operations are not supported under a patient-level scope, except \
                             the patient's own $everything",
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
                    let ctx = context.ctx.clone();
                    let claims = &ctx.user.claims;
                    let Some(matched_scope) =
                        get_highest_value_for_request_scope(&claims.scope, &context.request)?
                    else {
                        return Err(forbidden("Insufficient SMART scope for this request"));
                    };

                    // The compartment this request is confined to: the token's
                    // patient, when a `patient/` scope is what grants it.
                    let patient = claims.patient.as_deref();
                    let compartment = match (&matched_scope.user, patient) {
                        (SmartResourceScopeUser::Patient, Some(patient_id)) => Some(patient_id),
                        (SmartResourceScopeUser::Patient, None) => {
                            return Err(forbidden(
                                "A patient-level scope grants nothing without a patient in context",
                            ));
                        }
                        (SmartResourceScopeUser::User | SmartResourceScopeUser::System, _) => None,
                    };

                    let context = match compartment {
                        None => context,
                        Some(patient_id) => {
                            patient_scope::check_request(&context.request)?;
                            Context {
                                ctx: Arc::new(ctx.with_compartment(patient_id.to_string())),
                                request: context.request,
                                response: context.response,
                            }
                        }
                    };

                    let Some(next) = next else {
                        return Ok(context);
                    };

                    let mut result = next(state, context).await?;
                    if let Some(response) = result.response.take() {
                        result.response =
                            filter_response(response, &claims.scope, patient, compartment).await;
                    }
                    Ok(result)
                }
            }
        })
    }
}
