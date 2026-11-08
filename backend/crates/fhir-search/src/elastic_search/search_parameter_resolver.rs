use std::sync::Arc;

use elasticsearch::Elasticsearch;
use haste_fhir_client::request::SearchRequest;
use haste_fhir_operation_error::OperationOutcomeError;
use haste_jwt::{ProjectId, TenantId};

use crate::{
    SearchOptions, SearchReturn,
    elastic_search::search,
    memory::R4_SEARCH_PARAMETERS_INDEX,
    project_parameters::{IndexSearch, ProjectParameterResolver},
};

/// Resolves the base parameters plus each project's own, found in
/// Elasticsearch.
pub type ElasticSearchParameterResolver<Repo> = ProjectParameterResolver<ElasticIndexSearch, Repo>;

/// The Elasticsearch index, searched with the base parameters.
#[derive(Clone)]
pub struct ElasticIndexSearch {
    es: Arc<Elasticsearch>,
}

impl ElasticIndexSearch {
    #[must_use]
    pub fn new(es: Arc<Elasticsearch>) -> Self {
        ElasticIndexSearch { es }
    }
}

impl IndexSearch for ElasticIndexSearch {
    async fn search(
        &self,
        tenant: &TenantId,
        project: &ProjectId,
        request: &SearchRequest,
        options: &SearchOptions,
    ) -> Result<SearchReturn, OperationOutcomeError> {
        search::execute_search(
            self.es.clone(),
            R4_SEARCH_PARAMETERS_INDEX.clone(),
            tenant,
            project,
            request,
            Some(options),
        )
        .await
    }
}
