use crate::SearchParameterResolve;
use haste_artifacts::R4_SEARCH_PARAMETERS;
use haste_fhir_model::r4::generated::resources::{Resource, ResourceType, SearchParameter};
use haste_fhir_operation_error::OperationOutcomeError;
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

impl SearchParameterResolve for SearchParametersIndex {
    async fn by_resource_type(
        &self,
        _tenant: &TenantId,
        _project: &ProjectId,
        resource_type: &ResourceType,
    ) -> Result<Vec<Arc<SearchParameter>>, OperationOutcomeError> {
        let mut return_vec = Vec::new();

        if let Some(domain_params) = self
            .by_resource_type
            .get("DomainResource")
            .map(|d| d.values().cloned())
        {
            return_vec.extend(domain_params);
        }

        if let Some(resource_params) = self
            .by_resource_type
            .get("Resource")
            .map(|r| r.values().cloned())
        {
            return_vec.extend(resource_params);
        }

        if let Some(params) = self.by_resource_type.get(resource_type.as_ref()) {
            return_vec.extend(params.values().cloned());
        }

        Ok(return_vec)
    }

    async fn by_name(
        &self,
        _tenant: &TenantId,
        _project: &ProjectId,
        resource_type: Option<&ResourceType>,
        name: &str,
    ) -> Result<Option<Arc<SearchParameter>>, OperationOutcomeError> {
        Ok(resource_type
            .and_then(|resource_type| self.by_resource_type.get(resource_type.as_ref()))
            .and_then(|params| params.get(name))
            .or_else(|| {
                self.by_resource_type
                    .get("Resource")
                    .and_then(|params| params.get(name))
            })
            .or_else(|| {
                self.by_resource_type
                    .get("DomainResource")
                    .and_then(|params| params.get(name))
            })
            .cloned())
    }

    async fn all(
        &self,
        _tenant: &TenantId,
        _project: &ProjectId,
    ) -> Result<Vec<Arc<SearchParameter>>, OperationOutcomeError> {
        Ok(self.by_url.values().cloned().collect::<Vec<_>>())
    }
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

pub static R4_SEARCH_PARAMETERS_INDEX: LazyLock<Arc<SearchParametersIndex>> = LazyLock::new(|| {
    Arc::new(create_index_map(
        R4_SEARCH_PARAMETERS
            .iter()
            .map(|param| param.as_ref().clone())
            .collect(),
    ))
});

pub fn create_index_map(search_parameters: Vec<SearchParameter>) -> SearchParametersIndex {
    let mut index = SearchParametersIndex::default();
    for param in search_parameters.into_iter() {
        build_search_parameter_index_map(&mut index, Resource::SearchParameter(param))
            .expect("Failed to build search parameter index");
    }

    index
}
