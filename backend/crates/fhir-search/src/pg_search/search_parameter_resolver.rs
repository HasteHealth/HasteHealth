use haste_fhir_model::r4::generated::{
    resources::{Resource, ResourceType},
    terminology::IssueType,
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_jwt::{ProjectId, TenantId};
use haste_repository::{Repository, fhir::CachePolicy};
use moka::future::{Cache, CacheBuilder};
use sqlx::{Pool, Postgres, Row};
use std::sync::{Arc, LazyLock};

use crate::{
    ResolvedParameter, SearchParameterResolve,
    memory::{R4_SEARCH_PARAMETERS_INDEX, SearchParametersIndex, create_index_map},
};

#[derive(Clone)]
pub struct PgSearchParameterResolver<Repo: Repository + Send + Sync> {
    pool: Pool<Postgres>,
    repo: Arc<Repo>,
}

static PG_SEARCHPARAMETER_CACHE: LazyLock<
    Cache<(TenantId, ProjectId), Arc<SearchParametersIndex>>,
> = LazyLock::new(|| {
    CacheBuilder::new(50_000)
        .time_to_idle(std::time::Duration::from_hours(2))
        .build()
});

impl<Repo: Repository + Send + Sync> PgSearchParameterResolver<Repo> {
    pub fn new(pool: Pool<Postgres>, repo: Arc<Repo>) -> Self {
        PgSearchParameterResolver { pool, repo }
    }
}

/// Finds active SearchParameter resources in the PG search index and builds
/// a project-level search parameter index from them.
async fn create_project_sp_index<Repo: Repository + Send + Sync>(
    pool: &Pool<Postgres>,
    repo: &Repo,
    tenant: &TenantId,
    project: &ProjectId,
) -> Result<SearchParametersIndex, OperationOutcomeError> {
    // `SearchParameter.status` is an HL7 base parameter, so it lives in a
    // dedicated column on the per-resource-type table rather than in the
    // dynamic EAV tables.
    let rows = sqlx::query(
        "SELECT sr.resource_id, sr.version_id \
         FROM search_resource sr \
         JOIN search_searchparameter rt ON rt.tenant = sr.tenant AND rt.project = sr.project \
             AND rt.resource_id = sr.resource_id \
         WHERE sr.tenant = $1 AND sr.project = $2 \
             AND sr.resource_type = 'SearchParameter' \
             AND 'active' = ANY(rt.status_code) \
         LIMIT 10000",
    )
    .bind(tenant.as_ref())
    .bind(project.as_ref())
    .fetch_all(pool)
    .await
    .map_err(|e| {
        OperationOutcomeError::fatal(
            IssueType::exception(),
            format!("Failed to query PG search for SearchParameters: {e}"),
        )
    })?;

    let version_ids: Vec<haste_jwt::VersionId> = rows
        .iter()
        .map(|r| {
            let vid: String = r.get("version_id");
            haste_jwt::VersionId::new(vid)
        })
        .collect();

    let version_id_refs: Vec<&haste_jwt::VersionId> = version_ids.iter().collect();

    let project_sps = repo
        .read_by_version_ids(tenant, project, &version_id_refs, CachePolicy::Cache)
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
    pool: &Pool<Postgres>,
    repo: &Repo,
    tenant: TenantId,
    project: ProjectId,
) -> Result<Option<Arc<SearchParametersIndex>>, OperationOutcomeError> {
    if let (TenantId::System, ProjectId::System) = (&tenant, &project) {
        return Ok(None);
    }

    let index_key = (tenant, project);
    let pool = pool.clone();
    let index = PG_SEARCHPARAMETER_CACHE
        .try_get_with(index_key.clone(), async {
            create_project_sp_index(&pool, repo, &index_key.0, &index_key.1)
                .await
                .map(Arc::new)
        })
        .await
        .map_err(|e| OperationOutcomeError::fatal(IssueType::exception(), e.to_string()))?;

    Ok(Some(index))
}

impl<Repo: Repository + Send + Sync> SearchParameterResolve for PgSearchParameterResolver<Repo> {
    async fn by_resource_type(
        &self,
        tenant: &TenantId,
        project: &ProjectId,
        resource_type: &ResourceType,
    ) -> Result<Vec<ResolvedParameter>, OperationOutcomeError> {
        let mut sps = R4_SEARCH_PARAMETERS_INDEX
            .by_resource_type(tenant, project, resource_type)
            .await?;

        if let Some(project_index) = get_or_create_sp_index_for_project(
            &self.pool,
            self.repo.as_ref(),
            tenant.clone(),
            project.clone(),
        )
        .await?
        {
            let project_sps = project_index
                .by_resource_type(tenant, project, resource_type)
                .await?;
            sps.extend(project_sps);
        }

        Ok(sps)
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
            Ok(Some(parameter))
        } else if let Some(project_index) = get_or_create_sp_index_for_project(
            &self.pool,
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
        tenant: &TenantId,
        project: &ProjectId,
    ) -> Result<Vec<ResolvedParameter>, OperationOutcomeError> {
        let mut all_sps = R4_SEARCH_PARAMETERS_INDEX.all(tenant, project).await?;

        if let Some(project_index) = get_or_create_sp_index_for_project(
            &self.pool,
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
