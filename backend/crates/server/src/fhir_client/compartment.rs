use crate::fhir_client::FHIRServerClient;
use haste_artifacts::ARTIFACT_RESOURCES;
use haste_fhir_client::{
    request::{CompartmentRequest, FHIRRequest, FHIRResponse, SearchRequest},
    url::{Parameter, ParsedParameter},
};
use haste_fhir_model::r4::generated::{
    resources::{CompartmentDefinition, OperationOutcome, Resource, ResourceType},
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
        CompartmentType::Device(_) => Some(ResourceType::Device),
        CompartmentType::Encounter(_) => Some(ResourceType::Encounter),
        CompartmentType::Patient(_) => Some(ResourceType::Patient),
        CompartmentType::Practitioner(_) => Some(ResourceType::Practitioner),
        CompartmentType::RelatedPerson(_) => Some(ResourceType::RelatedPerson),
        CompartmentType::Null(_) => None,
    }
}

pub fn compartment_process<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
>(
    fhir_client: &FHIRServerClient<Repo, Search, Terminology>,
    compartment_request: &CompartmentRequest,
) -> Result<FHIRResponse, OperationOutcomeError> {
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

    match compartment_request.request.as_ref() {
        FHIRRequest::Search(SearchRequest::Type(type_search_request)) => {
            let Some(compartment_resource) = compartment
                .resource
                .as_ref()
                .unwrap_or(&vec![])
                .iter()
                .find(|resource_param| {
                    let code: Option<String> = resource_param.code.as_ref().into();
                    code.as_ref().map(|s| s.as_str())
                        == Some(compartment_request.resource_type.as_ref())
                })
            else {
                return Err(OperationOutcomeError::error(
                    IssueType::NotFound(None),
                    format!(
                        "Compartment definition for resource type '{}' does not include resource type '{}'.",
                        compartment_request.resource_type.as_ref(),
                        type_search_request.resource_type.as_ref()
                    ),
                ));
            };

            let parameters = compartment_resource
                .param
                .as_ref()
                .unwrap_or(&vec![])
                .iter()
                .filter_map(|p| {
                    if let Some(v) = p.value.as_ref() {
                        Some(ParsedParameter::Resource(Parameter {
                            name: v.to_string(),
                            value: format!(
                                "{}/{}",
                                compartment_request.resource_type.as_ref(),
                                compartment_request.id
                            ),
                            modifier: None,
                            chains: None,
                        }))
                    } else {
                        return None;
                    }
                })
                .collect::<Vec<ParsedParameter>>();

            Ok(())
        }
        // FHIRRequest::Read(read_request) => Ok(()),
        _ => {
            return Err(OperationOutcomeError::error(
                IssueType::NotSupported(None),
                "Only type search requests and reads are supported in compartment processing."
                    .to_string(),
            ));
        }
    }
}
