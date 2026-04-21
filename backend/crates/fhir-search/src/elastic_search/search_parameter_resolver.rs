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
    ResolvedParameter, SearchOptions, SearchParameterResolve,
    elastic_search::search,
    memory::{R4_SEARCH_PARAMETERS_INDEX, SearchParametersIndex, create_index_map},
};

#[derive(Clone)]
pub struct ElasticSearchParameterResolver<Repo: Repository + Send + Sync> {
    es: Arc<Elasticsearch>,
    repo: Arc<Repo>,
}

static SEARCHPARAMETER_CACHE: LazyLock<Cache<(TenantId, ProjectId), Arc<SearchParametersIndex>>> =
    LazyLock::new(|| {
        CacheBuilder::new(50_000)
            // Duration for 2 hour for search parameters.
            .time_to_idle(std::time::Duration::from_secs(2 * 60 * 60))
            .build()
    });

impl<Repo: Repository + Send + Sync> ElasticSearchParameterResolver<Repo> {
    pub fn new(es: Arc<Elasticsearch>, repo: Arc<Repo>) -> Self {
        ElasticSearchParameterResolver { es, repo }
    }
}

async fn create_project_sp_index<Repo: Repository + Send + Sync>(
    es: Arc<Elasticsearch>,
    repo: &Repo,
    tenant: &TenantId,
    project: &ProjectId,
) -> Result<SearchParametersIndex, OperationOutcomeError> {
    let result = search::execute_search(
        es,
        R4_SEARCH_PARAMETERS_INDEX.clone(),
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

    Ok(create_index_map(
        &crate::ParameterLevel::Project,
        project_sps,
    ))
}

async fn get_or_create_sp_index_for_project<Repo: Repository + Send + Sync>(
    es: Arc<Elasticsearch>,
    repo: &Repo,
    tenant: TenantId,
    project: ProjectId,
) -> Result<Option<Arc<SearchParametersIndex>>, OperationOutcomeError> {
    match (&tenant, &project) {
        (TenantId::System, ProjectId::System) => Ok(None),
        _ => {
            let index_key = (tenant, project);
            if let Some(index) = SEARCHPARAMETER_CACHE.get(&index_key).await {
                Ok(Some(index))
            } else {
                let index =
                    Arc::new(create_project_sp_index(es, repo, &index_key.0, &index_key.1).await?);
                SEARCHPARAMETER_CACHE.insert(index_key, index.clone()).await;

                Ok(Some(index))
            }
        }
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
    ) -> Result<Vec<ResolvedParameter>, OperationOutcomeError> {
        let mut sps_by_resource_type = R4_SEARCH_PARAMETERS_INDEX
            .by_resource_type(tenant, project, resource_type)
            .await?;

        if let Some(project_index) = get_or_create_sp_index_for_project(
            self.es.clone(),
            self.repo.as_ref(),
            tenant.clone(),
            project.clone(),
        )
        .await?
        {
            let project_sps = project_index
                .by_resource_type(tenant, project, resource_type)
                .await?;

            sps_by_resource_type.extend(project_sps);
        }

        Ok(sps_by_resource_type)
    }

    async fn by_name(
        &self,
        tenant: &haste_jwt::TenantId,
        project: &haste_jwt::ProjectId,
        resource_type: Option<&haste_fhir_model::r4::generated::resources::ResourceType>,
        code: &str,
    ) -> Result<Option<ResolvedParameter>, OperationOutcomeError> {
        if let Some(parameter) = R4_SEARCH_PARAMETERS_INDEX
            .by_name(tenant, project, resource_type, code)
            .await?
        {
            Ok(Some(parameter))
        } else if let Some(project_index) = get_or_create_sp_index_for_project(
            self.es.clone(),
            self.repo.as_ref(),
            tenant.clone(),
            project.clone(),
        )
        .await?
        {
            project_index
                .by_name(tenant, project, resource_type, code)
                .await
        } else {
            Ok(None)
        }
    }

    async fn all(
        &self,
        tenant: &haste_jwt::TenantId,
        project: &haste_jwt::ProjectId,
    ) -> Result<Vec<ResolvedParameter>, OperationOutcomeError> {
        let mut all_sps = R4_SEARCH_PARAMETERS_INDEX.all(tenant, project).await?;

        if let Some(project_index) = get_or_create_sp_index_for_project(
            self.es.clone(),
            self.repo.as_ref(),
            tenant.clone(),
            project.clone(),
        )
        .await?
        {
            all_sps.extend(project_index.all(tenant, project).await?);
        }

        Ok(all_sps)
    }
}
