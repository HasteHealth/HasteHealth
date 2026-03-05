use crate::fhir_client::ServerCTX;
use haste_artifacts::ARTIFACT_RESOURCES;
use haste_fhir_client::{
    FHIRClient,
    request::{
        CompartmentRequest, FHIRRequest, FHIRResponse, FHIRSearchTypeResponse, SearchRequest,
        SearchResponse,
    },
    url::{Parameter, ParsedParameter, ParsedParameters},
};
use haste_fhir_model::r4::generated::{
    resources::{Bundle, CompartmentDefinition, Resource, ResourceType},
    terminology::{CompartmentType, IssueType},
};
use haste_fhir_operation_error::OperationOutcomeError;
use std::sync::{Arc, LazyLock};

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

/// See https://build.fhir.org/compartmentdefinition.html
/// Use CompartmentDefinition resource (only hl7 provided ones) to process compartment requests.
/// An example of a compartment request is /Patient/123/Observation which utilizes patient compartmentdefinition
/// To determine query parameters for pulling observations for patient 123.
pub async fn process_compartment_request<
    Client: FHIRClient<Arc<ServerCTX<Client>>, OperationOutcomeError>,
>(
    fhir_client: &Client,
    ctx: Arc<ServerCTX<Client>>,
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
            let Some(compartment_resource) = compartment.resource.as_ref().and_then(|resources| {
                resources.iter().find(|resource_param| {
                    let code: Option<String> = resource_param.code.as_ref().into();
                    code.as_ref().map(|s| s.as_str())
                        == Some(type_search_request.resource_type.as_ref())
                })
            }) else {
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
                            value: vec![format!(
                                "{}/{}",
                                compartment_request.resource_type.as_ref(),
                                compartment_request.id
                            )],
                            modifier: None,
                            chains: None,
                        }))
                    } else {
                        return None;
                    }
                })
                .collect::<Vec<ParsedParameter>>();

            let mut return_bundle = Bundle::default();

            for search_param in parameters.into_iter() {
                let mut parameters = type_search_request.parameters.parameters().clone();
                parameters.extend(vec![search_param]);

                let bundle = fhir_client
                    .search_type(
                        ctx.clone(),
                        type_search_request.resource_type.clone(),
                        ParsedParameters::new(parameters),
                    )
                    .await?;

                let entries = bundle.entry.unwrap_or_default();
                return_bundle
                    .entry
                    .get_or_insert_with(Vec::new)
                    .extend(entries);
            }

            Ok(FHIRResponse::Search(SearchResponse::Type(
                FHIRSearchTypeResponse {
                    bundle: return_bundle,
                },
            )))
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
