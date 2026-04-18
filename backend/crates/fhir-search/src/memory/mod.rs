use crate::SearchParameterResolve;
use haste_artifacts::R4_SEARCH_PARAMETERS;
use haste_fhir_model::r4::generated::resources::{Resource, ResourceType, SearchParameter};
use haste_jwt::{ProjectId, TenantId};
use std::{
    collections::HashMap,
    sync::{Arc, LazyLock},
};

#[derive(Debug)]
pub enum ArtifactError {
    InvalidResource(String),
}

#[derive(Clone)]
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

fn build_search_parameter_index_map(
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

static R4_SEARCH_PARAMETERS_INDEX: LazyLock<SearchParametersIndex> = LazyLock::new(|| {
    create_index_map(
        R4_SEARCH_PARAMETERS
            .iter()
            .map(|param| param.as_ref().clone())
            .collect(),
    )
});

pub fn create_index_map(search_parameters: Vec<SearchParameter>) -> SearchParametersIndex {
    let mut index = SearchParametersIndex::default();
    for param in search_parameters.into_iter() {
        build_search_parameter_index_map(&mut index, Resource::SearchParameter(param))
            .expect("Failed to build search parameter index");
    }

    index
}

#[derive(Clone)]
pub struct SearchParameterMemoryResolve {}
impl SearchParameterMemoryResolve {
    pub fn new() -> Self {
        Self {}
    }
}
impl SearchParameterResolve for SearchParameterMemoryResolve {
    async fn by_resource_type(
        &self,
        _tenant: &TenantId,
        _project: &ProjectId,
        resource_type: &ResourceType,
    ) -> Vec<Arc<SearchParameter>> {
        let resource_params = R4_SEARCH_PARAMETERS_INDEX
            .by_resource_type
            .get("Resource")
            .unwrap();
        let domain_params = R4_SEARCH_PARAMETERS_INDEX
            .by_resource_type
            .get("DomainResource")
            .unwrap();
        let mut return_vec = Vec::new();
        return_vec.extend(resource_params.values().cloned());
        return_vec.extend(domain_params.values().cloned());

        if let Some(params) = R4_SEARCH_PARAMETERS_INDEX
            .by_resource_type
            .get(resource_type.as_ref())
        {
            return_vec.extend(params.values().cloned());
        }

        return_vec
    }

    async fn by_name(
        &self,
        _tenant: &TenantId,
        _project: &ProjectId,
        resource_type: Option<&ResourceType>,
        name: &str,
    ) -> Option<Arc<SearchParameter>> {
        resource_type
            .and_then(|resource_type| {
                R4_SEARCH_PARAMETERS_INDEX
                    .by_resource_type
                    .get(resource_type.as_ref())
            })
            .and_then(|params| params.get(name))
            .or_else(|| {
                R4_SEARCH_PARAMETERS_INDEX
                    .by_resource_type
                    .get("Resource")
                    .and_then(|params| params.get(name))
            })
            .or_else(|| {
                R4_SEARCH_PARAMETERS_INDEX
                    .by_resource_type
                    .get("DomainResource")
                    .and_then(|params| params.get(name))
            })
            .cloned()
    }

    async fn all(&self, _tenant: &TenantId, _project: &ProjectId) -> Vec<Arc<SearchParameter>> {
        R4_SEARCH_PARAMETERS_INDEX
            .by_url
            .values()
            .cloned()
            .collect::<Vec<_>>()
    }
}
