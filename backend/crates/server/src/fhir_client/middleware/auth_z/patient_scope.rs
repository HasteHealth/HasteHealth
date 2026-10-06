//! What a `patient/` scope reaches: one Patient compartment, read-only.
//!
//! Every function takes the id of the patient the token is bound to. A search
//! on a compartment member type runs with an [`AnyOf`] filter over the R4
//! compartment parameters; a read is checked against the resource it returned;
//! writes, history, system-level search, chained and reverse-chained
//! parameters, and every operation but the patient's own `$everything` are
//! refused. Reference data outside the compartment ([`SHARED_TYPES`]) is
//! readable as a whole.

use crate::fhir_client::{
    compartment::{in_patient_compartment, patient_compartment_parameters},
    middleware::auth_z::forbidden,
};
use haste_fhir_client::{
    request::{FHIRReadResponse, FHIRRequest, FHIRResponse, InvocationRequest, SearchRequest},
    url::{Parameter, ParsedParameter, ParsedParameters},
};
use haste_fhir_model::r4::generated::resources::{Resource, ResourceType};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_search::AnyOf;

/// Types a patient app needs that no CompartmentDefinition can place in a
/// compartment, since they carry no patient reference. Read and search only.
static SHARED_TYPES: &[ResourceType] = &[
    ResourceType::Practitioner,
    ResourceType::PractitionerRole,
    ResourceType::Organization,
    ResourceType::Location,
    ResourceType::Medication,
];

/// Refuses what a compartment cannot bound.
///
/// # Errors
///
/// Writes, history, system-level search, chained or `_has` parameters, and
/// types outside the compartment are forbidden.
pub fn check_request(request: &FHIRRequest) -> Result<(), OperationOutcomeError> {
    match request {
        FHIRRequest::Create(_)
        | FHIRRequest::Update(_)
        | FHIRRequest::Patch(_)
        | FHIRRequest::Delete(_) => Err(forbidden(
            "Writes are not supported under a patient-level scope",
        )),
        FHIRRequest::History(_) => Err(forbidden(
            "History is not supported under a patient-level scope",
        )),
        FHIRRequest::Search(SearchRequest::System(_)) => Err(forbidden(
            "System-level search is not supported under a patient-level scope",
        )),
        FHIRRequest::Search(SearchRequest::Type(search)) => {
            check_search_parameters(&search.parameters)?;
            check_type(&search.resource_type)
        }
        FHIRRequest::Read(read) => check_type(&read.resource_type),
        FHIRRequest::VersionRead(vread) => check_type(&vread.resource_type),
        // The compartment handler runs its searches back through the client,
        // where each one is checked and filtered on its own.
        FHIRRequest::Compartment(compartment) => match compartment.request.as_ref() {
            FHIRRequest::Search(SearchRequest::Type(search)) => {
                check_search_parameters(&search.parameters)?;
                check_type(&search.resource_type)
            }
            _ => Err(forbidden(
                "Only compartment searches are supported under a patient-level scope",
            )),
        },
        // Handled before this is reached.
        FHIRRequest::Capabilities
        | FHIRRequest::Batch(_)
        | FHIRRequest::Transaction(_)
        | FHIRRequest::Invocation(_) => Ok(()),
    }
}

/// Whether the token may invoke `invocation`: only its own
/// `Patient/{id}/$everything`, whose searches come back through the client
/// and are filtered there. Other operations may read the repository directly.
#[must_use]
pub fn allows_invocation(patient_id: &str, invocation: &InvocationRequest) -> bool {
    match invocation {
        InvocationRequest::Instance(request) => {
            request.resource_type == ResourceType::Patient
                && request.id == patient_id
                && request.operation.name() == "everything"
        }
        InvocationRequest::Type(_) | InvocationRequest::System(_) => false,
    }
}

/// The filter a search on `resource_type` runs with: any compartment
/// parameter referencing the patient, plus the patient's own record for
/// Patient (R4 lists a Patient in its own compartment only through `link`).
/// Shared types get no filter. Any other type, and a system-level search,
/// gets a filter nothing matches: [`check_request`] refuses those, and a
/// search that reaches storage another way must fail closed.
#[must_use]
pub fn search_filter(patient_id: &str, resource_type: Option<&ResourceType>) -> Option<AnyOf> {
    let resource_type = resource_type?;
    if SHARED_TYPES.contains(resource_type) {
        return None;
    }

    let parameter = |name: &str, value: String| Parameter {
        name: name.to_string(),
        value: vec![value],
        modifier: None,
        chains: Vec::new(),
    };

    let mut parameters: Vec<Parameter> = patient_compartment_parameters(resource_type)
        .unwrap_or_default()
        .iter()
        .map(|name| parameter(name, format!("Patient/{patient_id}")))
        .collect();
    if *resource_type == ResourceType::Patient {
        parameters.push(parameter("_id", patient_id.to_string()));
    }

    Some(AnyOf(parameters))
}

/// Whether `resource` may be returned: a shared type or a compartment member.
pub async fn resource_in_scope(patient_id: &str, resource: &Resource) -> bool {
    SHARED_TYPES.contains(&resource.resource_type())
        || in_patient_compartment(resource, patient_id).await
}

/// A read as the caller may see it: a resource outside the compartment
/// answers exactly as one that does not exist.
pub async fn filter_read(patient_id: &str, response: FHIRResponse) -> Option<FHIRResponse> {
    match response {
        FHIRResponse::Read(read) => {
            let resource = match read.resource {
                Some(resource) if resource_in_scope(patient_id, &resource).await => Some(resource),
                _ => None,
            };
            Some(FHIRResponse::Read(FHIRReadResponse { resource }))
        }
        FHIRResponse::VersionRead(vread) => {
            if resource_in_scope(patient_id, &vread.resource).await {
                Some(FHIRResponse::VersionRead(vread))
            } else {
                None
            }
        }
        other => Some(other),
    }
}

fn check_type(resource_type: &ResourceType) -> Result<(), OperationOutcomeError> {
    if SHARED_TYPES.contains(resource_type)
        || patient_compartment_parameters(resource_type).is_some()
    {
        return Ok(());
    }
    Err(forbidden(format!(
        "Resource type '{}' is not available under a patient-level scope: it is outside the \
         Patient compartment and is not shared reference data",
        resource_type.as_ref()
    )))
}

/// A chain evaluates a property of whatever a match references, which can be
/// another patient's record (a Coverage's subscriber); `_has` runs a search
/// the filter never sees. Neither can be bounded, so both are refused.
fn check_search_parameters(parameters: &ParsedParameters) -> Result<(), OperationOutcomeError> {
    for parameter in parameters.parameters() {
        if parameter.name().starts_with("_has") {
            return Err(forbidden(
                "Reverse chaining (_has) is not supported under a patient-level scope",
            ));
        }
        if let ParsedParameter::Resource(param) = parameter
            && param.is_chained()
        {
            return Err(forbidden(format!(
                "Chained parameter '{}' is not supported under a patient-level scope",
                param.name
            )));
        }
    }
    Ok(())
}
