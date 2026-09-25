use haste_fhir_model::r4::generated::{
    resources::{Resource, ResourceType},
    terminology::IssueType,
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_jwt::{ProjectId, TenantId};

use super::{
    keys,
    schema::{SharedTable, resource_table_name, shared_table_name},
};
use haste_repository::types::SupportedFHIRVersions;
use haste_repository::{Repository, fhir::CachePolicy};
use moka::future::{Cache, CacheBuilder};
use sqlx::{Pool, Postgres, Row};
use std::sync::{Arc, LazyLock};

use crate::{
    ResolvedParameter, SearchParameterResolve,
    memory::{R4_SEARCH_PARAMETERS_INDEX, SearchParametersIndex, create_index_map},
};

/// Resolves HL7 base parameters plus each project's own active
/// `SearchParameter` resources.
#[derive(Clone)]
pub struct PgSearchParameterResolver<Repo: Repository + Send + Sync> {
    pool: Pool<Postgres>,
    repo: Arc<Repo>,
}

impl<Repo: Repository + Send + Sync> PgSearchParameterResolver<Repo> {
    pub fn new(pool: Pool<Postgres>, repo: Arc<Repo>) -> Self {
        PgSearchParameterResolver { pool, repo }
    }
}

/// Per-project parameter indexes, evicted after two idle hours.
static PG_SEARCHPARAMETER_CACHE: LazyLock<
    Cache<(TenantId, ProjectId), Arc<SearchParametersIndex>>,
> = LazyLock::new(|| {
    CacheBuilder::new(50_000)
        .time_to_idle(std::time::Duration::from_hours(2))
        .build()
});

const CONFORMANCE_STATUS_URL: &str = "http://hl7.org/fhir/SearchParameter/conformance-status";

/// Builds a project's parameter index from its active `SearchParameter`s.
async fn create_project_sp_index<Repo: Repository + Send + Sync>(
    pool: &Pool<Postgres>,
    repo: &Repo,
    tenant: &TenantId,
    project: &ProjectId,
) -> Result<SearchParametersIndex, OperationOutcomeError> {
    // `SearchParameter.status` comes from `conformance-status`, a union that
    // can repeat, so it is read from the shared token table.
    let sql = format!(
        "SELECT sr.resource_id, sr.version_id \
         FROM {resource_table} sr \
         JOIN {token_table} t ON t.res_key = sr.res_key \
         WHERE sr.tenant = $1 AND sr.project = $2 \
             AND sr.resource_type = 'SearchParameter' \
             AND t.param_identity = $3 \
             AND t.code = 'active' \
         LIMIT 10000",
        resource_table = resource_table_name(&SupportedFHIRVersions::R4),
        token_table = shared_table_name(&SupportedFHIRVersions::R4, SharedTable::Token),
    );

    let rows = sqlx::query(&sql)
        .bind(tenant.as_ref())
        .bind(project.as_ref())
        .bind(keys::param_identity(
            tenant.as_ref(),
            project.as_ref(),
            CONFORMANCE_STATUS_URL,
        ))
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
        .map(|r| haste_jwt::VersionId::new(r.get("version_id")))
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

/// The project's cached parameter index, or `None` for the system project.
async fn project_index<Repo: Repository + Send + Sync>(
    resolver: &PgSearchParameterResolver<Repo>,
    tenant: &TenantId,
    project: &ProjectId,
) -> Result<Option<Arc<SearchParametersIndex>>, OperationOutcomeError> {
    if let (TenantId::System, ProjectId::System) = (tenant, project) {
        return Ok(None);
    }

    let index_key = (tenant.clone(), project.clone());
    PG_SEARCHPARAMETER_CACHE
        .try_get_with(index_key, async {
            create_project_sp_index(&resolver.pool, resolver.repo.as_ref(), tenant, project)
                .await
                .map(Arc::new)
        })
        .await
        .map(Some)
        .map_err(|e| OperationOutcomeError::fatal(IssueType::exception(), e.to_string()))
}

/// `base` followed by `extra`, reusing `base`'s allocation.
fn extended<T>(mut base: Vec<T>, extra: Vec<T>) -> Vec<T> {
    base.extend(extra);
    base
}

impl<Repo: Repository + Send + Sync> SearchParameterResolve for PgSearchParameterResolver<Repo> {
    async fn by_resource_type(
        &self,
        tenant: &TenantId,
        project: &ProjectId,
        resource_type: &ResourceType,
    ) -> Result<Vec<ResolvedParameter>, OperationOutcomeError> {
        let base = R4_SEARCH_PARAMETERS_INDEX
            .by_resource_type(tenant, project, resource_type)
            .await?;

        Ok(match project_index(self, tenant, project).await? {
            Some(index) => extended(
                base,
                index
                    .by_resource_type(tenant, project, resource_type)
                    .await?,
            ),
            None => base,
        })
    }

    async fn by_name(
        &self,
        tenant: &TenantId,
        project: &ProjectId,
        resource_type: Option<&ResourceType>,
        code: &str,
    ) -> Result<Option<ResolvedParameter>, OperationOutcomeError> {
        // Base parameters take precedence over a project's.
        if let Some(parameter) = R4_SEARCH_PARAMETERS_INDEX
            .by_name(tenant, project, resource_type, code)
            .await?
        {
            return Ok(Some(parameter));
        }

        match project_index(self, tenant, project).await? {
            Some(index) => index.by_name(tenant, project, resource_type, code).await,
            None => Ok(None),
        }
    }

    async fn all(
        &self,
        tenant: &TenantId,
        project: &ProjectId,
    ) -> Result<Vec<ResolvedParameter>, OperationOutcomeError> {
        let base = R4_SEARCH_PARAMETERS_INDEX.all(tenant, project).await?;

        Ok(match project_index(self, tenant, project).await? {
            Some(index) => extended(base, index.all(tenant, project).await?),
            None => base,
        })
    }
}
