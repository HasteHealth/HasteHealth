use haste_artifacts::ARTIFACT_RESOURCES;
use haste_fhir_model::r4::generated::{
    resources::{CompartmentDefinition, Resource, ResourceType},
    terminology::{CompartmentType, IssueType},
};
use haste_fhir_operation_error::OperationOutcomeError;
use std::sync::LazyLock;

static PATIENT_COMPARTMENT: LazyLock<Option<&CompartmentDefinition>> = LazyLock::new(|| {
    ARTIFACT_RESOURCES.iter().find_map(|r| match r {
        Resource::CompartmentDefinition(c) if c.code == CompartmentType::patient() => Some(c),
        _ => None,
    })
});

/// The search parameters that link a resource of `resource_type` to a patient, from the
/// Patient compartment. A Patient is found by its own `_id`. A type outside the compartment
/// can't be limited to patients, so it is refused.
pub(crate) fn patient_search_parameters(
    resource_type: &ResourceType,
) -> Result<Vec<String>, OperationOutcomeError> {
    if resource_type == &ResourceType::Patient {
        return Ok(vec!["_id".to_string()]);
    }

    let parameters: Vec<String> = PATIENT_COMPARTMENT
        .iter()
        .flat_map(|compartment| compartment.resource.iter().flatten())
        .filter(|entry| entry.code.as_str() == Some(resource_type.as_ref()))
        .flat_map(|entry| entry.param.iter().flatten())
        .filter_map(|param| param.value.clone())
        .collect();

    if parameters.is_empty() {
        return Err(OperationOutcomeError::error(
            IssueType::not_supported(),
            format!(
                "Resource type '{}' is not part of the Patient compartment, so it can't be scoped by 'patient' or 'group'.",
                resource_type.as_ref()
            ),
        ));
    }

    Ok(parameters)
}
