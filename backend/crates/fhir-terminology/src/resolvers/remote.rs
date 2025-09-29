use dashmap::DashMap;
use oxidized_fhir_client::request::FHIRSearchTypeRequest;
use oxidized_fhir_client::url::{Parameter, ParsedParameter};
use oxidized_fhir_model::r4::generated::resources::{Resource, ResourceType};
use oxidized_fhir_model::r4::generated::terminology::IssueType;
use oxidized_fhir_operation_error::OperationOutcomeError;
use oxidized_fhir_search::{SearchEngine, SearchRequest};
use oxidized_repository::Repository;
use oxidized_repository::types::SupportedFHIRVersions::R4;
use oxidized_repository::types::{ProjectId, TenantId, VersionIdRef};
use std::pin::Pin;
use std::sync::Arc;

use crate::resolvers::CanonicalResolver;

fn generate_key(resource_type: &ResourceType, url: &str) -> String {
    format!("{:?}::{}", resource_type, url)
}

pub struct LRUCanonicalRemoteResolver<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
> {
    cache: Arc<DashMap<String, Resource>>,
    search: Arc<Search>,
    repository: Arc<Repo>,
}

impl<Repo: Repository + Send + Sync + 'static, Search: SearchEngine + Send + Sync + 'static>
    LRUCanonicalRemoteResolver<Repo, Search>
{
    pub fn new(repository: Arc<Repo>, search: Arc<Search>, size: Option<usize>) -> Self {
        Self {
            cache: Arc::new(DashMap::new()),
            search,
            repository,
        }
    }
}

impl<Repo: Repository + Send + Sync + 'static, Search: SearchEngine + Send + Sync + 'static>
    CanonicalResolver for LRUCanonicalRemoteResolver<Repo, Search>
{
    fn resolve(
        &self,
        resource_type: ResourceType,
        url: String,
    ) -> Pin<Box<dyn Future<Output = Result<Resource, OperationOutcomeError>> + Send>> {
        let cache = self.cache.clone();
        let search = self.search.clone();
        let repository = self.repository.clone();
        Box::pin(async move {
            let key = generate_key(&resource_type, &url);
            if let Some(cached) = cache.get(&key) {
                Ok(cached.clone())
            } else {
                let result = search
                    .search(
                        &R4,
                        &TenantId::System,
                        &ProjectId::System,
                        SearchRequest::TypeSearch(&FHIRSearchTypeRequest {
                            resource_type: resource_type.clone(),
                            parameters: vec![ParsedParameter::Resource(Parameter {
                                name: "url".to_string(),
                                value: vec![url.clone()],
                                modifier: None,
                                chains: None,
                            })],
                        }),
                        None,
                    )
                    .await?;
                if let Some(entry) = result.entries.first()
                    && let Some(resource) = repository
                        .read_by_version_ids(
                            &TenantId::System,
                            &ProjectId::System,
                            vec![VersionIdRef::new(entry.version_id.as_ref())],
                        )
                        .await?
                        .pop()
                {
                    cache.insert(key, resource.clone());
                    Ok(resource)
                } else {
                    Err(OperationOutcomeError::error(
                        IssueType::NotFound(None),
                        format!(
                            "Could not find resource of type {:?} with url {}",
                            resource_type, url
                        ),
                    ))
                }
            }
        })
    }
}
