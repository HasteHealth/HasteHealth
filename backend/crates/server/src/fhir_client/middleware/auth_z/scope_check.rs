//! SMART resource scope enforcement.
//!
//! Picks the scopes that grant a request: those with the permission on the
//! type, at the highest level held (`system/` over `user/` over `patient/`).
//! A `system/` or `user/` grant reaches every resource of the type. A
//! `patient/` grant reaches one Patient compartment: [`patient_scope`] checks
//! the request, the context is confined to the patient so storage filters its
//! searches, and the response is checked against the resources it holds. A
//! grant made only of granular scopes (`patient/Observation.rs?category=...`)
//! reaches the resources matching any of them: [`granular_scope`] checks the
//! request, storage filters its searches by the same scopes, and reads are
//! checked against the resource returned.

use crate::fhir_client::{
    ClientState, ServerCTX, StorageError,
    middleware::{
        Backend, Middleware, Next, Output,
        auth_z::{forbidden, granular_scope, patient_scope},
    },
    utilities::{is_search_match, map_search_entries, request_to_resource_type},
};
use haste_fhir_client::request::{FHIRReadResponse, FHIRRequest, FHIRResponse};
use haste_fhir_model::r4::generated::{
    resources::{BundleEntry, Resource, ResourceType},
    terminology::IssueType,
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_search::AnyOf;
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

/// The scopes that grant `permission` on `resource_type`, at the highest
/// level any of them holds: when a `system/` scope grants it, the `patient/`
/// scopes that also would are left aside, since the system one reaches every
/// patient. Empty when nothing grants it. Every scope returned has the same
/// level, so the first one's `user` is the level of the grant.
pub(super) fn granting_scopes<'a>(
    scopes: &'a Scopes,
    resource_type: Option<&ResourceType>,
    permission: &SmartResourceScopePermission,
) -> Vec<&'a SMARTResourceScope> {
    let fitting: Vec<&SMARTResourceScope> = resource_scopes(scopes)
        .filter(|scope| {
            fits_resource_type(scope, resource_type) && scope.permissions.has_permission(permission)
        })
        .collect();
    let Some(highest) = fitting
        .iter()
        .map(|scope| get_user_weight_scope(&scope.user))
        .max()
    else {
        return Vec::new();
    };
    fitting
        .into_iter()
        .filter(|scope| get_user_weight_scope(&scope.user) == highest)
        .collect()
}

/// Whether every resource scope in the token is patient-level.
fn only_patient_scopes(scopes: &Scopes) -> bool {
    let mut scopes = resource_scopes(scopes).peekable();
    scopes.peek().is_some() && scopes.all(|scope| scope.user == SmartResourceScopeUser::Patient)
}

/// Whether the caller may see `resource`: in the `compartment` the request
/// was confined to, if any, and reached by the `granular` filter, if any.
async fn resource_allowed(
    resource: &Resource,
    compartment: Option<&str>,
    granular: Option<&AnyOf>,
) -> bool {
    let in_compartment = match compartment {
        Some(patient_id) => patient_scope::resource_in_scope(patient_id, resource).await,
        None => true,
    };
    in_compartment
        && match granular {
            Some(filter) => granular_scope::resource_in_scope(filter, resource).await,
            None => true,
        }
}

/// Keeps the matches, which the engine already filtered, and the includes the
/// caller's scopes allow: an include needs a read scope on its own type, at
/// patient level it must be in the compartment of `patient`, the token's
/// patient, and under granular read scopes alone it must match one of them.
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
        let granting = granting_scopes(
            scopes,
            Some(&resource.resource_type()),
            &SmartResourceScopePermission::Read,
        );
        let compartment = match (granting.first().map(|scope| &scope.user), patient) {
            (None, _) | (Some(SmartResourceScopeUser::Patient), None) => continue,
            (Some(SmartResourceScopeUser::Patient), Some(patient_id)) => Some(patient_id),
            (Some(SmartResourceScopeUser::User | SmartResourceScopeUser::System), _) => None,
        };
        let granular = granular_scope::filter_of(&granting);
        if resource_allowed(resource, compartment, granular.as_ref()).await {
            kept.push(entry);
        }
    }

    kept
}

/// A read as the caller may see it: a resource outside the compartment or
/// the granular filter answers exactly as one that does not exist.
///
/// # Errors
///
/// A version read of such a resource is not found.
async fn filter_read(
    response: FHIRResponse,
    compartment: Option<&str>,
    granular: Option<&AnyOf>,
) -> Result<FHIRResponse, OperationOutcomeError> {
    match response {
        FHIRResponse::Read(read) => {
            let resource = match read.resource {
                Some(resource) if resource_allowed(&resource, compartment, granular).await => {
                    Some(resource)
                }
                _ => None,
            };
            Ok(FHIRResponse::Read(FHIRReadResponse { resource }))
        }
        FHIRResponse::VersionRead(vread) => {
            if resource_allowed(&vread.resource, compartment, granular).await {
                Ok(FHIRResponse::VersionRead(vread))
            } else {
                Err(StorageError::NotFound(
                    vread.resource.resource_type(),
                    vread.resource.id().clone().unwrap_or_default(),
                )
                .into())
            }
        }
        other => Ok(other),
    }
}

/// The response as the caller may see it: a read is checked against the
/// `compartment` the request was confined to and the `granular` filter its
/// searches ran with, a search page's includes against the caller's scopes.
async fn filter_response(
    response: FHIRResponse,
    scopes: &Scopes,
    patient: Option<&str>,
    compartment: Option<&str>,
    granular: Option<&AnyOf>,
) -> Result<FHIRResponse, OperationOutcomeError> {
    match response {
        response @ (FHIRResponse::Read(_) | FHIRResponse::VersionRead(_))
            if compartment.is_some() || granular.is_some() =>
        {
            filter_read(response, compartment, granular).await
        }
        response => {
            Ok(
                map_search_entries(response, |entries| filter_entries(entries, scopes, patient))
                    .await,
            )
        }
    }
}

pub struct SmartScopeAccess;

impl<B: Backend> Middleware<B> for SmartScopeAccess {
    fn call<'a>(
        &'a self,
        state: &'a ClientState<B>,
        ctx: Arc<ServerCTX<B>>,
        request: FHIRRequest,
        next: Next<'a, B>,
    ) -> Output<'a> {
        match &request {
            // Batch and transaction will call back into this middleware for their individual requests
            // at which point the permissions will be checked.
            FHIRRequest::Capabilities | FHIRRequest::Batch(_) | FHIRRequest::Transaction(_) => {
                next.run(state, ctx, request)
            }
            // An operation's own data access comes back through this
            // client and is checked there.
            FHIRRequest::Invocation(invocation) => {
                let claims = &ctx.user.claims;
                if only_patient_scopes(&claims.scope)
                    && !claims.patient.as_deref().is_some_and(|patient_id| {
                        patient_scope::allows_invocation(patient_id, invocation)
                    })
                {
                    return Box::pin(std::future::ready(Err(forbidden(
                        "Operations are not supported under a patient-level scope, except \
                         the patient's own $everything",
                    ))));
                }
                next.run(state, ctx, request)
            }
            FHIRRequest::Compartment(_)
            | FHIRRequest::Create(_)
            | FHIRRequest::Read(_)
            | FHIRRequest::VersionRead(_)
            | FHIRRequest::Update(_)
            | FHIRRequest::Patch(_)
            | FHIRRequest::Delete(_)
            | FHIRRequest::Search(_)
            | FHIRRequest::History(_) => Box::pin(async move {
                // The caller's claims outlive the context the request runs in.
                let caller = ctx.clone();
                let claims = &caller.user.claims;
                let granting = granting_scopes(
                    &claims.scope,
                    request_to_resource_type(&request),
                    &request_type_to_permission(&request)?,
                );
                let Some(level) = granting.first().map(|scope| &scope.user) else {
                    return Err(forbidden("Insufficient SMART scope for this request"));
                };

                // The compartment this request is confined to: the token's
                // patient, when a `patient/` scope is what grants it.
                let patient = claims.patient.as_deref();
                let compartment = match (level, patient) {
                    (SmartResourceScopeUser::Patient, Some(patient_id)) => Some(patient_id),
                    (SmartResourceScopeUser::Patient, None) => {
                        return Err(forbidden(
                            "A patient-level scope grants nothing without a patient in context",
                        ));
                    }
                    (SmartResourceScopeUser::User | SmartResourceScopeUser::System, _) => None,
                };

                // The resources the request is confined to, when only
                // granular scopes grant it. Storage derives the same filter
                // from the claims for the searches it runs.
                let granular = granular_scope::filter_of(&granting);
                if granular.is_some() {
                    granular_scope::check_request(&request)?;
                }

                let ctx = match compartment {
                    None => ctx,
                    Some(patient_id) => {
                        patient_scope::check_request(&request)?;
                        Arc::new(ctx.with_compartment(patient_id.to_string()))
                    }
                };

                let response = next.run(state, ctx, request).await?;
                filter_response(
                    response,
                    &claims.scope,
                    patient,
                    compartment,
                    granular.as_ref(),
                )
                .await
            }),
        }
    }
}
