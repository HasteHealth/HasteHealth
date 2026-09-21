use haste_fhir_model::r4::generated::{
    resources::{Resource, ResourceType},
    terminology::IssueType,
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_jwt::{ProjectId, TenantId};

use super::schema::{SharedTable, resource_table_name};
use haste_repository::types::SupportedFHIRVersions;
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

/// Finds active `SearchParameter` resources in the PG search index and builds
/// a project-level search parameter index from them.
async fn create_project_sp_index<Repo: Repository + Send + Sync>(
    pool: &Pool<Postgres>,
    repo: &Repo,
    tenant: &TenantId,
    project: &ProjectId,
) -> Result<SearchParametersIndex, OperationOutcomeError> {
    // `SearchParameter.status` comes from `conformance-status`, whose
    // expression is a union across every conformance resource. A union can
    // yield more than one value, so the parameter has no column of its own and
    // is read from the shared token table like any other repeating parameter.
    let sql = format!(
        "SELECT sr.resource_id, sr.version_id \
         FROM {resource_table} sr \
         JOIN {token_table} t ON t.tenant = sr.tenant AND t.project = sr.project \
             AND t.resource_type = sr.resource_type AND t.resource_id = sr.resource_id \
         WHERE sr.tenant = $1 AND sr.project = $2 \
             AND sr.resource_type = 'SearchParameter' \
             AND t.param_url = 'http://hl7.org/fhir/SearchParameter/conformance-status' \
             AND t.code = 'active' \
         LIMIT 10000",
        resource_table = resource_table_name(SupportedFHIRVersions::R4),
        token_table = SharedTable::Token.table_name(SupportedFHIRVersions::R4),
    );

    let rows = sqlx::query(&sql)
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
