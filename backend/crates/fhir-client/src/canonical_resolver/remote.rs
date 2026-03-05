use dashmap::DashMap;
use haste_fhir_model::r4::generated::resources::{Resource, ResourceType};
use std::sync::Arc;

use crate::FHIRClient;
use crate::canonical_resolver::CanonicalResolver;
use crate::url::{Parameter, ParsedParameter, ParsedParameters};

fn generate_key(resource_type: &ResourceType, url: &str) -> String {
    format!("{:?}::{}", resource_type, url)
}

pub struct LRUCanonicalRemoteResolver<CTX: Send + Sync, Error: Send + Sync, Client: FHIRClient<CTX, Error> + Send + Sync> {
    cache: Arc<DashMap<String, Arc<Resource>>>,
    _phantom: std::marker::PhantomData<(CTX, Error)>,
    client: Client,
}

impl<CTX: Send + Sync, Error: Send + Sync, Client: FHIRClient<CTX, Error> + Send + Sync>
    LRUCanonicalRemoteResolver<CTX, Error, Client>
{
    pub fn new(client: Client) -> Self {
        Self {
            cache: Arc::new(DashMap::new()),
            _phantom: std::marker::PhantomData,
            client,
        }
    }

    pub fn insert_into_cache(&self, resource_type: &ResourceType, url: &str, resource: Arc<Resource>) {
        let key = generate_key(resource_type, url);
        self.cache.insert(key, resource);
    }
 
    pub fn clear_cache(&self) {
        self.cache.clear();
    }
}

impl<CTX: Send + Sync, Error: Send + Sync, Client: FHIRClient<CTX, Error> + Send + Sync> CanonicalResolver<CTX, Error>
    for LRUCanonicalRemoteResolver<CTX, Error, Client>
{
    async fn resolve(
        &self,
        ctx: CTX,
        resource_type: ResourceType,
        canonical_url: String,
    ) -> Result<Option<Arc<Resource>>, Error> {
        let key = generate_key(&resource_type, &canonical_url);
        if let Some(cached) = self.cache.get(&key) {
            Ok(Some(cached.clone()))
        } else {
            if let Some(url) = canonical_url.split('|').next()
                // Perform search for an entry with the given canonical URL.
                && let Some(resource) = self.client
                    .search_type(
                        ctx,
                        resource_type,
            ParsedParameters::new(vec![ParsedParameter::Resource(Parameter {
                                name: "url".to_string(),
                                value: vec![url.to_string()],
                                modifier: None,
                                chains: None,
                            })]),
            
                
                    )
                    .await?
                    .entry
                    .and_then(|mut e| e.pop()).and_then(|e| e.resource)

            {
                let resource = Arc::new(*resource);
                self.cache.insert(key, resource.clone());
                return Ok(Some(resource));
            }
            
            Ok(None)
        }

    }
}
