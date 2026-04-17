use elasticsearch::Elasticsearch;
use std::sync::Arc;

use crate::SearchParameterResolve;

pub struct ElasticSearchParameterResolver {
    client: Arc<Elasticsearch>,
}

impl ElasticSearchParameterResolver {
    pub fn new(client: Arc<Elasticsearch>) -> Self {
        ElasticSearchParameterResolver { client }
    }
}

impl SearchParameterResolve for ElasticSearchParameterResolver {
    async fn by_resource_type(
        &self,
        tenant_id: &haste_jwt::TenantId,
        project_id: &haste_jwt::ProjectId,
        resource_type: &haste_fhir_model::r4::generated::resources::ResourceType,
    ) -> Vec<Arc<haste_fhir_model::r4::generated::resources::SearchParameter>> {
        todo!()
    }

    async fn by_name(
        &self,
        tenant_id: &haste_jwt::TenantId,
        project_id: &haste_jwt::ProjectId,
        resource_type: Option<&haste_fhir_model::r4::generated::resources::ResourceType>,
        code: &str,
    ) -> Option<Arc<haste_fhir_model::r4::generated::resources::SearchParameter>> {
        todo!()
    }

    async fn all(
        &self,
        tenant_id: &haste_jwt::TenantId,
        project_id: &haste_jwt::ProjectId,
    ) -> Vec<Arc<haste_fhir_model::r4::generated::resources::SearchParameter>> {
        todo!()
    }
}
