use crate::fhir_client::ServerCTX;
use dashmap::DashMap;
use haste_fhir_client::{
    FHIRClient,
    canonical_resolver::CanonicalResolver,
    url::{Parameter, ParsedParameter, ParsedParameters},
};
use haste_fhir_model::r4::generated::resources::{Resource, ResourceType};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_jwt::{ProjectId, TenantId};
use std::sync::{Arc, LazyLock};

fn generate_key(
    tenant_id: &TenantId,
    project_id: &ProjectId,
    resource_type: &ResourceType,
    url: &str,
) -> String {
    format!(
        "{}::{}::{}::{}",
        tenant_id,
        project_id,
        resource_type.as_ref(),
        url
    )
}

static CACHE: LazyLock<DashMap<String, Arc<Resource>>> = LazyLock::new(DashMap::new);

pub struct ServerCTXResolver<Client: FHIRClient<Arc<ServerCTX<Client>>, OperationOutcomeError>>(
    Arc<ServerCTX<Client>>,
);

impl<Client: FHIRClient<Arc<ServerCTX<Client>>, OperationOutcomeError>> ServerCTXResolver<Client> {
    pub fn new(ctx: Arc<ServerCTX<Client>>) -> Self {
        Self(ctx)
    }
}

impl<Client: FHIRClient<Arc<ServerCTX<Client>>, OperationOutcomeError>> Clone
    for ServerCTXResolver<Client>
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<Client: FHIRClient<Arc<ServerCTX<Client>>, OperationOutcomeError>> CanonicalResolver
    for ServerCTXResolver<Client>
{
    async fn resolve(
        &self,
        resource_type: ResourceType,
        canonical_url: &str,
    ) -> Result<Option<Arc<Resource>>, OperationOutcomeError> {
        let key = generate_key(
            &self.0.tenant,
            &self.0.project,
            &resource_type,
            canonical_url,
        );
        if let Some(cached) = CACHE.get(&key) {
            Ok(Some(cached.clone()))
        } else {
            if let Some(url) = canonical_url.split('|').next()
                // Perform search for an entry with the given canonical URL.
                && let Some(resolved_resource) = self.0.client
                    .search_type(
                        self.0.clone(),
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
                let resource = Arc::new(*resolved_resource);
                CACHE.insert(key, resource.clone());
                return Ok(Some(resource));
            }

            Ok(None)
        }
    }
}
