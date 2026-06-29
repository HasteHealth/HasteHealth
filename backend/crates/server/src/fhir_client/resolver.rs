use crate::fhir_client::ServerCTX;
use haste_fhir_client::{FHIRClient, canonical_resolver::CanonicalResolver};
use haste_fhir_model::r4::generated::resources::{Resource, ResourceType};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_jwt::{ProjectId, TenantId};
use moka::future::{Cache, CacheBuilder};
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

static CACHE: LazyLock<Cache<String, Option<Arc<Resource>>>> = LazyLock::new(|| {
    // Cache entries live for 2 hours, after which they will be automatically evicted.
    CacheBuilder::new(10_000)
        .time_to_idle(std::time::Duration::from_secs(2 * 60 * 60))
        .build()
});

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
        if let Some(cached) = CACHE.get(&key).await {
            Ok(cached.clone())
        } else {
            if let Some(url) = canonical_url.split('|').next()
                // Perform search for an entry with the given canonical URL.
                && let resolved_resource = self.0.client
                    .search_type(
                        self.0.clone(),
                        resource_type,
            vec![
                            ("url".to_string(), vec![url.to_string()])
                        ].into()
                    )
                    .await?
                    .entry
                    .and_then(|mut e| e.pop()).and_then(|e| e.resource)
            {
                let arced_resource = resolved_resource.map(|r| Arc::new(*r));
                CACHE.insert(key, arced_resource.clone()).await;
                return Ok(arced_resource);
            }

            Ok(None)
        }
    }
}
