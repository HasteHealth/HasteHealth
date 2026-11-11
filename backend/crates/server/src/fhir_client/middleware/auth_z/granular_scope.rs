//! What a granular scope reaches: the resources of its type that match its
//! search parameters (`patient/Observation.rs?category=...`, SMART App
//! Launch 2.x, "Finer-grained resource constraints using search parameters").
//!
//! The scopes granting a request on a type decide together. When one of them
//! is plain it reaches every resource of the type and nothing is filtered:
//! scopes are a union of access, so the broader one wins. When every one is
//! granular, a resource is reached if it matches all the parameters of any
//! one of them. That filter is `ANDed` into every search of the type by
//! storage ([`search_filter`]), and a read is checked against the resource it
//! returned ([`resource_in_scope`]), as a `patient/` scope's compartment is.
//!
//! Writes, history and system-level search are refused under granular scopes
//! alone ([`check_request`]): a search parameter can bound what a search
//! returns, not what is written or what a version once held.

use crate::fhir_client::middleware::auth_z::{forbidden, scope_check::granting_scopes};
use haste_fhir_client::{
    request::{FHIRRequest, SearchRequest},
    url::{ParsedParameter, ParsedParameters},
};
use haste_fhir_model::r4::generated::resources::{Resource, ResourceType};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_search::{AllOf, AnyOf, memory::search::satisfies};
use haste_jwt::scopes::{SMARTResourceScope, Scopes, SmartResourceScopePermission};

/// The filter the scopes granting `permission` on `resource_type` confine a
/// search to: `None` when a plain scope among them reaches every resource of
/// the type (or when none grants it, which is refused before a search runs),
/// otherwise one branch per granular scope.
#[must_use]
pub fn search_filter(
    scopes: &Scopes,
    resource_type: &ResourceType,
    permission: &SmartResourceScopePermission,
) -> Option<AnyOf> {
    filter_of(&granting_scopes(scopes, Some(resource_type), permission))
}

/// The filter the `granting` scopes, those of one level that grant a request
/// on one type, confine it to. See [`search_filter`].
#[must_use]
pub fn filter_of(granting: &[&SMARTResourceScope]) -> Option<AnyOf> {
    if granting.is_empty() || granting.iter().any(|scope| !scope.is_granular()) {
        return None;
    }
    Some(AnyOf(
        granting.iter().filter_map(|scope| branch(scope)).collect(),
    ))
}

/// A granular scope's constraints as the parameters of a search, parsed as
/// they would be from a URL (modifier, comma-separated values, escapes). A
/// scope whose constraints do not parse as resource parameters, say one
/// naming `_count`, reaches nothing: it has no branch.
fn branch(scope: &SMARTResourceScope) -> Option<AllOf> {
    let pairs: Vec<(&str, &str)> = scope
        .constraints
        .iter()
        .map(|constraint| (constraint.name.as_str(), constraint.value.as_str()))
        .collect();
    let parameters = match ParsedParameters::try_from(pairs.as_slice()) {
        Ok(parameters) => parameters.owned_parameters(),
        Err(error) => {
            tracing::warn!("granular scope: constraints do not parse, nothing reached: {error}");
            return None;
        }
    };
    parameters
        .into_iter()
        .map(|parameter| match parameter {
            ParsedParameter::Resource(parameter) => Some(parameter),
            ParsedParameter::Result(parameter) => {
                tracing::warn!(
                    "granular scope: '{}' is not a resource parameter, nothing reached",
                    parameter.name
                );
                None
            }
        })
        .collect::<Option<Vec<_>>>()
        .map(AllOf)
}

/// Refuses what a search parameter cannot bound.
///
/// # Errors
///
/// Writes, history and system-level search are forbidden.
pub fn check_request(request: &FHIRRequest) -> Result<(), OperationOutcomeError> {
    match request {
        FHIRRequest::Create(_)
        | FHIRRequest::Update(_)
        | FHIRRequest::Patch(_)
        | FHIRRequest::Delete(_) => Err(forbidden(
            "Writes are not supported under a granular scope: a scope with search parameters \
             grants reads and searches of the resources that match them",
        )),
        FHIRRequest::History(_) => {
            Err(forbidden("History is not supported under a granular scope"))
        }
        FHIRRequest::Search(SearchRequest::System(_)) => Err(forbidden(
            "System-level search is not supported under a granular scope",
        )),
        FHIRRequest::Search(SearchRequest::Type(_))
        | FHIRRequest::Read(_)
        | FHIRRequest::VersionRead(_)
        | FHIRRequest::Compartment(_)
        | FHIRRequest::Capabilities
        | FHIRRequest::Batch(_)
        | FHIRRequest::Transaction(_)
        | FHIRRequest::Invocation(_) => Ok(()),
    }
}

/// Whether `resource` is one the `filter` reaches, by the R4 search
/// parameters evaluated on the resource itself, as storage would index it.
/// A parameter that cannot be evaluated this way (one a project defines, a
/// chain) fails closed: the resource is not in scope.
pub async fn resource_in_scope(filter: &AnyOf, resource: &Resource) -> bool {
    match satisfies(resource, filter).await {
        Ok(reached) => reached,
        Err(error) => {
            tracing::warn!(
                "granular scope: cannot evaluate the filter on {}, refused: {error}",
                resource.resource_type().as_ref()
            );
            false
        }
    }
}
