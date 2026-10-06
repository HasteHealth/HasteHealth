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
    terminology::{BoundCode, CompartmentType, IssueType},
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_search::{indexing_conversion, memory::R4_SEARCH_PARAMETERS_INDEX};
use haste_fhirpath::FPEngine;
use std::{
    collections::HashMap,
    sync::{Arc, LazyLock},
};

// Supported Compartment Definitions from R4.
static COMPARTMENTS: LazyLock<Vec<&'static CompartmentDefinition>> = LazyLock::new(|| {
    ARTIFACT_RESOURCES
        .iter()
        .filter_map(|r| match r {
            Resource::CompartmentDefinition(c) => Some(c),
            _ => None,
        })
        .collect::<Vec<_>>()
});

static FP_ENGINE: LazyLock<FPEngine> = LazyLock::new(FPEngine::new);

/// Each Patient compartment member type with the codes of the search
/// parameters that place a resource of that type in a patient's compartment.
static PATIENT_COMPARTMENT_PARAMETERS: LazyLock<HashMap<String, Vec<String>>> =
    LazyLock::new(|| {
        COMPARTMENTS
            .iter()
            .find(|compartment| compartment.code == CompartmentType::patient())
            .and_then(|compartment| compartment.resource.as_ref())
            .into_iter()
            .flatten()
            .filter_map(|resource| {
                let code = resource.code.as_str()?;
                let parameters: Vec<String> = resource
                    .param
                    .as_ref()?
                    .iter()
                    .filter_map(|p| p.value.clone())
                    .collect();
                (!parameters.is_empty()).then(|| (code.to_string(), parameters))
            })
            .collect()
    });

/// The same membership as FHIRPath: the expression of each parameter above,
/// so a resource can be placed without the index.
static PATIENT_COMPARTMENT_EXPRESSIONS: LazyLock<HashMap<String, Vec<String>>> =
    LazyLock::new(|| {
        let parameters = R4_SEARCH_PARAMETERS_INDEX.all_parameters();
        let expressions: HashMap<(&str, &str), &str> = parameters
            .iter()
            .flat_map(|parameter| {
                let parameter = &parameter.search_parameter;
                let code = parameter.code.value.as_deref();
                let expression = parameter
                    .expression
                    .as_ref()
                    .and_then(|e| e.value.as_deref());
                parameter
                    .base
                    .iter()
                    .filter_map(move |base| Some(((base.as_str()?, code?), expression?)))
            })
            .collect();

        PATIENT_COMPARTMENT_PARAMETERS
            .iter()
            .map(|(resource_type, codes)| {
                let found = codes
                    .iter()
                    .filter_map(|code| expressions.get(&(resource_type.as_str(), code.as_str())))
                    .map(ToString::to_string)
                    .collect();
                (resource_type.clone(), found)
            })
            .collect()
    });

/// The search parameters that place a `resource_type` in a Patient
/// compartment; `None` when the type is not a member.
#[must_use]
pub fn patient_compartment_parameters(resource_type: &ResourceType) -> Option<&'static [String]> {
    PATIENT_COMPARTMENT_PARAMETERS
        .get(resource_type.as_ref())
        .map(Vec::as_slice)
}

/// Whether `resource` is in `patient_id`'s compartment: the Patient itself, or
/// a resource one of its compartment parameters, evaluated as indexing does,
/// points at `Patient/{patient_id}`.
pub async fn in_patient_compartment(resource: &Resource, patient_id: &str) -> bool {
    let resource_type = resource.resource_type();
    if resource_type == ResourceType::Patient && resource.id().as_deref() == Some(patient_id) {
        return true;
    }

    let Some(expressions) = PATIENT_COMPARTMENT_EXPRESSIONS.get(resource_type.as_ref()) else {
        return false;
    };
    for expression in expressions {
        match FP_ENGINE.evaluate(expression, vec![resource]).await {
            Ok(values) => {
                let references_patient = values
                    .iter()
                    .flat_map(indexing_conversion::index_reference_values)
                    .any(|r| r.resource_type() == Some("Patient") && r.id() == Some(patient_id));
                if references_patient {
                    return true;
                }
            }
            Err(e) => tracing::warn!(
                "compartment: failed to evaluate '{expression}' for {}: {e}",
                resource_type.as_ref()
            ),
        }
    }
    false
}

fn compartment_type_to_resource_type(
    compartment_type: &BoundCode<CompartmentType>,
) -> Option<ResourceType> {
    match compartment_type {
        c if c == &CompartmentType::device() => Some(ResourceType::Device),
        c if c == &CompartmentType::encounter() => Some(ResourceType::Encounter),
        c if c == &CompartmentType::patient() => Some(ResourceType::Patient),
        c if c == &CompartmentType::practitioner() => Some(ResourceType::Practitioner),
        c if c == &CompartmentType::related_person() => Some(ResourceType::RelatedPerson),
        _ => None,
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
            IssueType::not_found(),
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
                    let code = resource_param.code.as_str();
                    code == Some(type_search_request.resource_type.as_ref())
                })
            }) else {
                return Err(OperationOutcomeError::error(
                    IssueType::not_found(),
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
                    let v = p.value.as_ref()?;
                    Some(ParsedParameter::Resource(Parameter {
                        name: v.to_string(),
                        value: vec![format!(
                            "{}/{}",
                            compartment_request.resource_type.as_ref(),
                            compartment_request.id
                        )],
                        modifier: None,
                        chains: Vec::new(),
                    }))
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
        _ => Err(OperationOutcomeError::error(
            IssueType::not_supported(),
            "Only type search requests and reads are supported in compartment processing."
                .to_string(),
        )),
    }
}
