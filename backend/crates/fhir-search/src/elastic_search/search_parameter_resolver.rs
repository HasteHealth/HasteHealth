use elasticsearch::Elasticsearch;
use haste_fhir_client::{
    request::{FHIRSearchTypeRequest, SearchRequest},
    url::ParsedParameters,
};
use haste_fhir_model::r4::generated::resources::{Resource, ResourceType};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_jwt::{ProjectId, TenantId};
use haste_repository::{Repository, fhir::CachePolicy};
use moka::future::{Cache, CacheBuilder};
use std::sync::{Arc, LazyLock};

use crate::{
    SearchOptions, SearchParameterResolve,
    elastic_search::search,
    memory::{SearchParameterMemoryResolve, SearchParametersIndex, create_index_map},
};

#[allow(dead_code)]
pub struct ElasticSearchParameterResolver<Repo: Repository + Send + Sync> {
    es: Arc<Elasticsearch>,
    repo: Repo,
}

#[allow(dead_code)]
static SEARCHPARAMETER_CACHE: LazyLock<Cache<(TenantId, ProjectId), Arc<SearchParametersIndex>>> =
    LazyLock::new(|| {
        CacheBuilder::new(50_000)
            // Duration for 1 hour for search parameters.
            .time_to_idle(std::time::Duration::from_secs(60 * 60))
            .build()
    });

impl<Repo: Repository + Send + Sync> ElasticSearchParameterResolver<Repo> {
    #[allow(dead_code)]
    pub fn new(es: Arc<Elasticsearch>, repo: Repo) -> Self {
        ElasticSearchParameterResolver { es, repo }
    }
}

#[allow(dead_code)]
async fn create_project_sp_index<Repo: Repository + Send + Sync>(
    es: Arc<Elasticsearch>,
    repo: &Repo,
    tenant: &TenantId,
    project: &ProjectId,
) -> Result<SearchParametersIndex, OperationOutcomeError> {
    let result = search::execute_search(
        es,
        Arc::new(SearchParameterMemoryResolve::new()),
        &haste_repository::types::SupportedFHIRVersions::R4,
        tenant,
        project,
        &SearchRequest::Type(FHIRSearchTypeRequest {
            resource_type: ResourceType::SearchParameter,
            parameters: ParsedParameters::new(vec![]),
        }),
        &Some(SearchOptions { count_limit: false }),
    )
    .await?;

    let version_ids = result
        .entries
        .iter()
        .map(|r| &r.version_id)
        .collect::<Vec<_>>();

    let project_sps = repo
        .read_by_version_ids(tenant, project, &version_ids, CachePolicy::Cache)
        .await?
        .into_iter()
        .filter_map(|r| match r {
            Resource::SearchParameter(sp) => Some(sp),
            _ => None,
        })
        .collect::<Vec<_>>();

    Ok(create_index_map(project_sps))
}

#[allow(dead_code)]
async fn get_or_create_sp_index_for_project<Repo: Repository + Send + Sync>(
    es: Arc<Elasticsearch>,
    repo: &Repo,
    tenant: TenantId,
    project: ProjectId,
) -> Result<Arc<SearchParametersIndex>, OperationOutcomeError> {
    let index_key = (tenant, project);
    if let Some(index) = SEARCHPARAMETER_CACHE.get(&index_key).await {
        Ok(index)
    } else {
        let index = Arc::new(create_project_sp_index(es, repo, &index_key.0, &index_key.1).await?);
        SEARCHPARAMETER_CACHE.insert(index_key, index.clone()).await;

        Ok(index)
    }
}

impl<Repo: Repository + Send + Sync> SearchParameterResolve
    for ElasticSearchParameterResolver<Repo>
{
    async fn by_resource_type(
        &self,
        tenant: &haste_jwt::TenantId,
        project: &haste_jwt::ProjectId,
        resource_type: &haste_fhir_model::r4::generated::resources::ResourceType,
    ) -> Result<
        Vec<Arc<haste_fhir_model::r4::generated::resources::SearchParameter>>,
        OperationOutcomeError,
    > {
        // let project_index = get_or_create_sp_index_for_project(
        //     self.es.as_ref(),
        //     &self.repo,
        //     tenant.clone(),
        //     project.clone(),
        // )
        // .await?;

        let root = SearchParameterMemoryResolve::new()
            .by_resource_type(tenant, project, resource_type)
            .await?;

        // root.extend(;

        Ok(root)
    }

    async fn by_name(
        &self,
        tenant: &haste_jwt::TenantId,
        project: &haste_jwt::ProjectId,
        resource_type: Option<&haste_fhir_model::r4::generated::resources::ResourceType>,
        code: &str,
    ) -> Result<
        Option<Arc<haste_fhir_model::r4::generated::resources::SearchParameter>>,
        OperationOutcomeError,
    > {
        if let Some(parameter) = SearchParameterMemoryResolve::new()
            .by_name(tenant, project, resource_type, code)
            .await?
        {
            Ok(Some(parameter))
        } else {
            Ok(None)
        }
    }

    async fn all(
        &self,
        tenant: &haste_jwt::TenantId,
        project: &haste_jwt::ProjectId,
    ) -> Result<
        Vec<Arc<haste_fhir_model::r4::generated::resources::SearchParameter>>,
        OperationOutcomeError,
    > {
        SearchParameterMemoryResolve::new()
            .all(tenant, project)
            .await
    }
}
