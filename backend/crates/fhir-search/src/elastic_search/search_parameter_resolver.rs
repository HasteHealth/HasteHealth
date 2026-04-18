use elasticsearch::Elasticsearch;
use haste_jwt::{ProjectId, TenantId};
use moka::future::{Cache, CacheBuilder};
use std::sync::{Arc, LazyLock};

use crate::{
    SearchParameterResolve,
    memory::{SearchParameterMemoryResolve, SearchParametersIndex},
};

#[allow(dead_code)]
pub struct ElasticSearchParameterResolver {
    es: Arc<Elasticsearch>,
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
    pub fn new(es: Arc<Elasticsearch>) -> Self {
        ElasticSearchParameterResolver { es }
    }
}

impl SearchParameterResolve for ElasticSearchParameterResolver {
    async fn by_resource_type(
        &self,
        tenant: &haste_jwt::TenantId,
        project: &haste_jwt::ProjectId,
        resource_type: &haste_fhir_model::r4::generated::resources::ResourceType,
    ) -> Vec<Arc<haste_fhir_model::r4::generated::resources::SearchParameter>> {
        SearchParameterMemoryResolve::new()
            .by_resource_type(tenant, project, resource_type)
            .await
    }

    async fn by_name(
        &self,
        tenant: &haste_jwt::TenantId,
        project: &haste_jwt::ProjectId,
        resource_type: Option<&haste_fhir_model::r4::generated::resources::ResourceType>,
        code: &str,
    ) -> Option<Arc<haste_fhir_model::r4::generated::resources::SearchParameter>> {
        SearchParameterMemoryResolve::new()
            .by_name(tenant, project, resource_type, code)
            .await
    }

    async fn all(
        &self,
        tenant: &haste_jwt::TenantId,
        project: &haste_jwt::ProjectId,
    ) -> Vec<Arc<haste_fhir_model::r4::generated::resources::SearchParameter>> {
        SearchParameterMemoryResolve::new()
            .all(tenant, project)
            .await
    }
}
