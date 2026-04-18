use elasticsearch::Elasticsearch;
use haste_jwt::{ProjectId, TenantId};
use moka::future::{Cache, CacheBuilder};
use std::sync::{Arc, LazyLock};

use crate::{SearchParameterResolve, memory::SearchParametersIndex};

#[allow(dead_code)]
pub struct ElasticSearchParameterResolver {
    client: Arc<Elasticsearch>,
}

#[allow(dead_code)]
static SEARCHPARAMETER_CACHE: LazyLock<Cache<(TenantId, ProjectId), SearchParametersIndex>> =
    LazyLock::new(|| {
        CacheBuilder::new(50_000)
            // Duration for 1 hour for search parameters.
            .time_to_idle(std::time::Duration::from_secs(60 * 60))
            .build()
    });

impl ElasticSearchParameterResolver {
    #[allow(dead_code)]
    pub fn new(client: Arc<Elasticsearch>) -> Self {
        ElasticSearchParameterResolver { client }
    }
}

impl SearchParameterResolve for ElasticSearchParameterResolver {
    async fn by_resource_type(
        &self,
        _tenant: &haste_jwt::TenantId,
        _project: &haste_jwt::ProjectId,
        _resource_type: &haste_fhir_model::r4::generated::resources::ResourceType,
    ) -> Vec<Arc<haste_fhir_model::r4::generated::resources::SearchParameter>> {
        todo!()
    }

    async fn by_name(
        &self,
        _tenant: &haste_jwt::TenantId,
        _project: &haste_jwt::ProjectId,
        _resource_type: Option<&haste_fhir_model::r4::generated::resources::ResourceType>,
        _code: &str,
    ) -> Option<Arc<haste_fhir_model::r4::generated::resources::SearchParameter>> {
        todo!()
    }

    async fn all(
        &self,
        _tenant: &haste_jwt::TenantId,
        _project: &haste_jwt::ProjectId,
    ) -> Vec<Arc<haste_fhir_model::r4::generated::resources::SearchParameter>> {
        todo!()
    }
}
