use haste_artifacts::ARTIFACT_RESOURCES;
use haste_fhir_client::FHIRClient;
use haste_fhir_model::r4::generated::{
    resources::{CompartmentDefinition, Resource, ResourceType},
    terminology::{CompartmentType, IssueType},
};
use haste_fhir_operation_error::OperationOutcomeError;
use std::{collections::HashSet, sync::LazyLock};

static PATIENT_COMPARTMENT: LazyLock<Option<&'static CompartmentDefinition>> =
    LazyLock::new(|| {
        ARTIFACT_RESOURCES.iter().find_map(|r| match r {
            Resource::CompartmentDefinition(c) if c.code == CompartmentType::patient() => Some(c),
            _ => None,
        })
    });

fn patient_search_param_names(target_resource_type: &str) -> Vec<String> {
    let Some(compartment) = *PATIENT_COMPARTMENT else {
        return Vec::new();
    };

    compartment
        .resource
        .as_ref()
        .into_iter()
        .flatten()
        .filter(|resource_param| resource_param.code.as_str() == Some(target_resource_type))
        .flat_map(|resource_param| resource_param.param.as_ref().into_iter().flatten())
        .filter_map(|p| p.value.as_ref().map(std::string::ToString::to_string))
        .collect()
}

/// Fetches every resource of `target_resource_type` belonging to any of
/// `patient_references` (values like `"Patient/123"`), via the Patient
/// compartment's search parameter mapping. Optionally restricts to
/// resources matching a `_lastUpdated` filter expression (e.g. `"gt2024-01-01"`).
pub(crate) async fn resources_for_patients<
    CTX: Send + Sync + Clone + 'static,
    Client: FHIRClient<CTX, OperationOutcomeError> + Send + Sync + 'static,
>(
    context: CTX,
    client: &Client,
    target_resource_type: ResourceType,
    patient_references: &[String],
    last_updated_filter: Option<&str>,
) -> Result<Vec<Resource>, OperationOutcomeError> {
    if patient_references.is_empty() {
        return Ok(Vec::new());
    }

    if target_resource_type == ResourceType::Patient {
        let mut resources = Vec::with_capacity(patient_references.len());

        for reference in patient_references {
            let id = reference
                .rsplit('/')
                .next()
                .unwrap_or(reference)
                .to_string();

            if let Some(resource) = client
                .read(context.clone(), ResourceType::Patient, id)
                .await?
            {
                resources.push(resource);
            }
        }

        return Ok(resources);
    }

    let param_names = patient_search_param_names(target_resource_type.as_ref());

    if param_names.is_empty() {
        return Err(OperationOutcomeError::error(
            IssueType::not_supported(),
            format!(
                "Resource type '{}' is not part of the Patient compartment, so it can't be scoped by 'patient' or 'group'.",
                target_resource_type.as_ref()
            ),
        ));
    }

    let mut seen_ids = HashSet::new();
    let mut resources = Vec::new();

    for param_name in param_names {
        let mut search_params: Vec<(String, Vec<String>)> =
            vec![(param_name, patient_references.to_vec())];

        if let Some(filter) = last_updated_filter {
            search_params.push(("_lastUpdated".to_string(), vec![filter.to_string()]));
        }

        let bundle = client
            .search_type(
                context.clone(),
                target_resource_type.clone(),
                search_params.into(),
            )
            .await?;

        for entry in bundle.entry.into_iter().flatten() {
            let Some(resource) = entry.resource else {
                continue;
            };

            if seen_ids.insert((resource.resource_type(), resource.id().clone())) {
                resources.push(*resource);
            }
        }
    }

    Ok(resources)
}
