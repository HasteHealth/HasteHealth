use once_cell::sync::Lazy;
use oxidized_fhir_model::r4::types::{Resource, ResourceType, SearchParameter};
use oxidized_macro_loads::load_artifacts;
use std::{collections::HashMap, sync::Arc};

static SEARCH_PARAMETERS_STRS: &[&str] = load_artifacts!(
    "../artifacts/r4/hl7/search-parameters.min.json"
    "../artifacts/r4/oxidized_health/search_parameter"
);

#[derive(Debug)]
pub enum ArtifactError {
    InvalidResource(String),
}

pub struct SearchParametersIndex {
    by_url: HashMap<String, Arc<SearchParameter>>,
    by_resource_type: HashMap<String, HashMap<String, Arc<SearchParameter>>>,
}

impl Default for SearchParametersIndex {
    fn default() -> Self {
        SearchParametersIndex {
            by_url: HashMap::new(),
            by_resource_type: HashMap::new(),
        }
    }
}

fn index_parameter(
    index: &mut SearchParametersIndex,
    resource: Resource,
) -> Result<(), ArtifactError> {
    match resource {
        Resource::Bundle(bundle) => {
            let params = bundle
                .entry
                .unwrap_or(vec![])
                .into_iter()
                .flat_map(|e| e.resource)
                .filter_map(|resource| match *resource {
                    Resource::SearchParameter(search_param) => Some(Arc::new(search_param)),
                    _ => None,
                });

            for param in params {
                index
                    .by_url
                    .insert(param.id.clone().unwrap(), param.clone());
                for resource_type in &param.base {
                    if let Some(resource_type) = resource_type.as_ref().value.as_ref() {
                        index
                            .by_resource_type
                            .entry(resource_type.to_string())
                            .or_default()
                            .insert(
                                param.code.value.as_ref().unwrap().to_string(),
                                param.clone(),
                            );
                    }
                }
            }

            Ok(())
        }
        Resource::SearchParameter(search_param) => {
            let param = Arc::new(search_param);
            index
                .by_url
                .insert(param.id.clone().unwrap(), param.clone());
            for resource_type in &param.base {
                if let Some(resource_type) = resource_type.as_ref().value.as_ref() {
                    index
                        .by_resource_type
                        .entry(resource_type.to_string())
                        .or_default()
                        .insert(
                            param.code.value.as_ref().unwrap().to_string(),
                            param.clone(),
                        );
                }
            }
            Ok(())
        }
        _ => Err(ArtifactError::InvalidResource(
            "Expected a Bundle resource".to_string(),
        )),
    }
}

static R4_SEARCH_PARAMETERS: Lazy<SearchParametersIndex> = Lazy::new(|| {
    let mut index = SearchParametersIndex::default();
    for search_str in SEARCH_PARAMETERS_STRS {
        let bundle = oxidized_fhir_serialization_json::from_str::<Resource>(search_str)
            .expect("Failed to parse search parameters JSON");
        index_parameter(&mut index, bundle).expect("Failed to extract search parameters");
    }
    index
});

pub fn get_all_search_parameters() -> Vec<Arc<SearchParameter>> {
    R4_SEARCH_PARAMETERS
        .by_url
        .values()
        .cloned()
        .collect::<Vec<_>>()
}

pub fn get_search_parameters_for_resource(
    resource_type: &ResourceType,
) -> Vec<Arc<SearchParameter>> {
    let resource_params = R4_SEARCH_PARAMETERS
        .by_resource_type
        .get("Resource")
        .unwrap();
    let domain_params = R4_SEARCH_PARAMETERS
        .by_resource_type
        .get("DomainResource")
        .unwrap();
    let mut return_vec = Vec::new();
    return_vec.extend(resource_params.values().cloned());
    return_vec.extend(domain_params.values().cloned());

    if let Some(params) = R4_SEARCH_PARAMETERS
        .by_resource_type
        .get(resource_type.as_str())
    {
        return_vec.extend(params.values().cloned());
    }

    return_vec
}

pub fn get_search_parameter_for_name(
    resource_type: &ResourceType,
    name: &str,
) -> Option<Arc<SearchParameter>> {
    R4_SEARCH_PARAMETERS
        .by_resource_type
        .get(resource_type.as_str())
        .and_then(|params| params.get(name))
        .or_else(|| {
            R4_SEARCH_PARAMETERS
                .by_resource_type
                .get("Resource")
                .and_then(|params| params.get(name))
        })
        .or_else(|| {
            R4_SEARCH_PARAMETERS
                .by_resource_type
                .get("DomainResource")
                .and_then(|params| params.get(name))
        })
        .cloned()
}
