use haste_fhir_model::r4::generated::resources::{Resource, ResourceType, SearchParameter};
use once_cell::sync::Lazy;
use rust_embed::Embed;
use std::{collections::HashMap, sync::Arc};

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
                    let resource_type: Option<String> = (&**resource_type).into();
                    if let Some(resource_type) = resource_type {
                        index
                            .by_resource_type
                            .entry(resource_type)
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
                let resource_type: Option<String> = (&**resource_type).into();
                if let Some(resource_type) = resource_type.as_ref() {
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

#[derive(Embed)]
#[folder = "./artifacts/r4"]
#[include = "haste_health/search_parameter/*.json"]
#[include = "hl7/minified/search-parameters.min.json"]

struct EmbededSearchParameterAssets;

static R4_SEARCH_PARAMETERS: Lazy<SearchParametersIndex> = Lazy::new(|| {
    let mut index = SearchParametersIndex::default();

    for path in EmbededSearchParameterAssets::iter() {
        let data = EmbededSearchParameterAssets::get(path.as_ref()).unwrap();
        let bundle = haste_fhir_serialization_json::from_str::<Resource>(
            std::str::from_utf8(&data.data).unwrap(),
        )
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
        .get(resource_type.as_ref())
    {
        return_vec.extend(params.values().cloned());
    }

    return_vec
}

pub fn get_search_parameter_for_name(
    resource_type: Option<&ResourceType>,
    name: &str,
) -> Option<Arc<SearchParameter>> {
    resource_type
        .and_then(|resource_type| {
            R4_SEARCH_PARAMETERS
                .by_resource_type
                .get(resource_type.as_ref())
        })
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
