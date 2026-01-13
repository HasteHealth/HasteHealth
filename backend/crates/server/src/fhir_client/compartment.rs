use crate::fhir_client::FHIRServerClient;
use haste_artifacts::ARTIFACT_RESOURCES;
use haste_fhir_client::request::CompartmentRequest;
use haste_fhir_model::r4::generated::{
    resources::{CompartmentDefinition, Resource, ResourceType},
    terminology::{CompartmentType, IssueType},
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_repository::Repository;
use std::sync::LazyLock;

// Supported Compartment Definitions from R4.
static COMPARTMENTS: LazyLock<Vec<&'static CompartmentDefinition>> = LazyLock::new(|| {
    ARTIFACT_RESOURCES
        .iter()
        .filter_map(|r| match r.as_ref() {
            Resource::CompartmentDefinition(c) => Some(c),
            _ => None,
        })
        .collect::<Vec<_>>()
});

fn compartment_type_to_resource_type(compartment_type: &CompartmentType) -> Option<ResourceType> {
    match compartment_type {
        CompartmentType::Device(element) => Some(ResourceType::Device),
        CompartmentType::Encounter(element) => Some(ResourceType::Encounter),
        CompartmentType::Patient(element) => Some(ResourceType::Patient),
        CompartmentType::Practitioner(element) => Some(ResourceType::Practitioner),
        CompartmentType::RelatedPerson(element) => Some(ResourceType::RelatedPerson),
        CompartmentType::Null(element) => None,
    }
}

pub fn compartment_process<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
>(
    fhir_client: &FHIRServerClient<Repo, Search, Terminology>,
    compartment_request: &CompartmentRequest,
) -> Result<(), OperationOutcomeError> {
    let Some(compartment) = COMPARTMENTS.iter().find(|compartment_def| {
        let compartment_type = compartment_type_to_resource_type(&compartment_def.code);
        compartment_type.as_ref() == Some(&compartment_request.resource_type)
    }) else {
        return Err(OperationOutcomeError::error(
            IssueType::NotFound(None),
            format!(
                "Compartment definition for resource type {:?} not found.",
                compartment_request.resource_type
            ),
        ));
    };

    Ok(())
}
