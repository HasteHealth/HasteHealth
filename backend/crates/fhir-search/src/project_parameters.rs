//! A project's own `SearchParameter`s, resolved on top of the HL7 base
//! parameters.
//!
//! A process caches a project's parameters for [`PROJECT_PARAMETERS_TTL`].
//! Two things refresh a copy sooner: the indexing worker drops a project's
//! copy once it has indexed one of its `SearchParameter`s
//! ([`index_search_parameters_first`]), and a lookup by name that misses
//! rebuilds the copy once, so a new parameter is searchable as soon as it is
//! indexed. The TTL bounds what neither catches: an update or delete seen by
//! a server, a list by type (`_include=*`), and a second worker process.

use std::{collections::HashSet, sync::Arc, time::Duration};

use haste_fhir_client::request::{FHIRSearchTypeRequest, SearchRequest};
use haste_fhir_model::r4::generated::{
    resources::{Resource, ResourceType},
    terminology::IssueType,
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_jwt::{ProjectId, TenantId};
use haste_repository::{Repository, fhir::CachePolicy};
use moka::future::{Cache, CacheBuilder};

use crate::{
    IndexOutcome, IndexResource, MAX_COUNT, ParameterLevel, ResolvedParameter, SearchOptions,
    SearchParameterResolve, SearchReturn,
    memory::{R4_SEARCH_PARAMETERS_INDEX, SearchParametersIndex, create_index_map},
};

/// How long a process trusts its cached copy of a project's parameters.
///
/// A refresh is one small search per project per TTL, paid by the first
/// lookup after expiry, and only for projects being indexed or searched.
/// A new parameter does not wait for it (see the module doc), so the TTL
/// only sets how long an update, a delete or a second worker can go
/// unnoticed. A minute keeps that short at a negligible refresh rate.
pub const PROJECT_PARAMETERS_TTL: Duration = Duration::from_secs(60);

/// A search backend's index, as far as the resolver needs it: a type-level
/// search whose parameters are resolved with the base parameters alone.
pub trait IndexSearch: Send + Sync {
    fn search(
        &self,
        tenant: &TenantId,
        project: &ProjectId,
        request: &SearchRequest,
        options: &SearchOptions,
    ) -> impl Future<Output = Result<SearchReturn, OperationOutcomeError>> + Send;
}

/// Resolves the HL7 base parameters plus each project's own active
/// `SearchParameter`s, which `Index` finds and `Repo` reads. A base parameter
/// takes precedence over a project's of the same name.
#[derive(Clone)]
pub struct ProjectParameterResolver<Index, Repo> {
    index: Index,
    repo: Arc<Repo>,
    /// Per-project copies, see [`PROJECT_PARAMETERS_TTL`].
    cache: Cache<(TenantId, ProjectId), Arc<SearchParametersIndex>>,
}

impl<Index: IndexSearch, Repo: Repository + Send + Sync> ProjectParameterResolver<Index, Repo> {
    pub fn new(index: Index, repo: Arc<Repo>) -> Self {
        ProjectParameterResolver {
            index,
            repo,
            cache: CacheBuilder::new(50_000)
                .time_to_live(PROJECT_PARAMETERS_TTL)
                .build(),
        }
    }

    /// Builds the project's parameter index from its active `SearchParameter`s.
    ///
    /// An ordinary `SearchParameter?status=active` search, so `status` is read
    /// from wherever the backend's schema stores it, as any other search would.
    async fn build(
        &self,
        tenant: &TenantId,
        project: &ProjectId,
    ) -> Result<Arc<SearchParametersIndex>, OperationOutcomeError> {
        let request = SearchRequest::Type(FHIRSearchTypeRequest {
            resource_type: ResourceType::SearchParameter,
            parameters: vec![("status".to_string(), vec!["active".to_string()])].into(),
        });
        let options = SearchOptions {
            count_limit: Some(MAX_COUNT),
            any_of: None,
        };
        let found = self
            .index
            .search(tenant, project, &request, &options)
            .await?;
        let version_ids = found
            .entries
            .iter()
            .map(|entry| &entry.version_id)
            .collect::<Vec<_>>();

        let parameters = self
            .repo
            .read_by_version_ids(tenant, project, &version_ids, CachePolicy::Cache)
            .await?
            .into_iter()
            .filter_map(|resource| match resource {
                Resource::SearchParameter(parameter) => Some(parameter),
                _ => None,
            })
            .collect();

        Ok(Arc::new(create_index_map(
            &ParameterLevel::Project,
            parameters,
        )))
    }

    /// The project's cached parameters, or `None` for the system project,
    /// which has none of its own.
    async fn cached(
        &self,
        tenant: &TenantId,
        project: &ProjectId,
    ) -> Result<Option<Arc<SearchParametersIndex>>, OperationOutcomeError> {
        if let (TenantId::System, ProjectId::System) = (tenant, project) {
            return Ok(None);
        }

        // Concurrent misses share one build.
        self.cache
            .try_get_with(
                (tenant.clone(), project.clone()),
                self.build(tenant, project),
            )
            .await
            .map(Some)
            .map_err(|e| OperationOutcomeError::fatal(IssueType::exception(), e.to_string()))
    }
}

impl<Index: IndexSearch, Repo: Repository + Send + Sync> SearchParameterResolve
    for ProjectParameterResolver<Index, Repo>
{
    async fn by_resource_type(
        &self,
        tenant: &TenantId,
        project: &ProjectId,
        resource_type: &ResourceType,
    ) -> Result<Vec<ResolvedParameter>, OperationOutcomeError> {
        let mut parameters = R4_SEARCH_PARAMETERS_INDEX
            .by_resource_type(tenant, project, resource_type)
            .await?;
        if let Some(index) = self.cached(tenant, project).await? {
            parameters.extend(
                index
                    .by_resource_type(tenant, project, resource_type)
                    .await?,
            );
        }
        Ok(parameters)
    }

    async fn by_name(
        &self,
        tenant: &TenantId,
        project: &ProjectId,
        resource_type: Option<&ResourceType>,
        code: &str,
    ) -> Result<Option<ResolvedParameter>, OperationOutcomeError> {
        if let Some(parameter) = R4_SEARCH_PARAMETERS_INDEX
            .by_name(tenant, project, resource_type, code)
            .await?
        {
            return Ok(Some(parameter));
        }

        let Some(index) = self.cached(tenant, project).await? else {
            return Ok(None);
        };
        if let Some(parameter) = index.get(resource_type, code) {
            return Ok(Some(parameter.clone()));
        }

        // The name may belong to a `SearchParameter` indexed since the copy
        // was built. An unknown name costs one rebuild per lookup.
        let rebuilt = self.build(tenant, project).await?;
        self.cache
            .insert((tenant.clone(), project.clone()), rebuilt.clone())
            .await;
        Ok(rebuilt.get(resource_type, code).cloned())
    }

    async fn all(
        &self,
        tenant: &TenantId,
        project: &ProjectId,
    ) -> Result<Vec<ResolvedParameter>, OperationOutcomeError> {
        let mut parameters = R4_SEARCH_PARAMETERS_INDEX.all(tenant, project).await?;
        if let Some(index) = self.cached(tenant, project).await? {
            parameters.extend(index.all_parameters());
        }
        Ok(parameters)
    }

    async fn invalidate_project(&self, tenant: &TenantId, project: &ProjectId) {
        self.cache
            .invalidate(&(tenant.clone(), project.clone()))
            .await;
    }
}

/// Indexes a batch's `SearchParameter` writes before the rest of it, then
/// drops the cached parameters of their projects, so every other resource in
/// the batch is indexed with the project's parameters as they now stand.
///
/// `index` indexes one part. It is passed `true` for the `SearchParameter`
/// part, whose writes must be searchable once it returns, since the next
/// build reads them.
pub(crate) async fn index_search_parameters_first<Resolver, Index, Indexed>(
    resolver: &Resolver,
    resources: Vec<IndexResource>,
    index: Index,
) -> Result<IndexOutcome, OperationOutcomeError>
where
    Resolver: SearchParameterResolve,
    Index: Fn(Vec<IndexResource>, bool) -> Indexed,
    Indexed: Future<Output = Result<IndexOutcome, OperationOutcomeError>>,
{
    let is_parameter =
        |resource: &IndexResource| resource.resource_type == ResourceType::SearchParameter;
    // Most batches have none, and are indexed as they are.
    if !resources.iter().any(is_parameter) {
        return index(resources, false).await;
    }

    let (parameters, rest): (Vec<_>, Vec<_>) = resources.into_iter().partition(is_parameter);
    let projects = parameters
        .iter()
        .map(|resource| (resource.tenant.clone(), resource.project.clone()))
        .collect::<HashSet<_>>();
    let mut outcome = index(parameters, true).await?;
    for (tenant, project) in &projects {
        resolver.invalidate_project(tenant, project).await;
    }

    if !rest.is_empty() {
        let rest = index(rest, false).await?;
        outcome.succeeded += rest.succeeded;
        outcome.failed.extend(rest.failed);
    }
    Ok(outcome)
}
