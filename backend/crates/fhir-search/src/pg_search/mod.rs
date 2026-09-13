use std::sync::{Arc, LazyLock};

use haste_fhir_client::request::SearchRequest;
use haste_fhir_model::r4::generated::{resources::Resource, terminology::IssueType};
use haste_fhir_operation_error::{OperationOutcomeError, derive::OperationOutcomeError};
use haste_fhirpath::FPEngine;
use haste_jwt::{ProjectId, TenantId};
use haste_repository::types::{FHIRMethod, SupportedFHIRVersions};
use sqlx::{Pool, Postgres, postgres::PgPoolOptions};

use crate::{
    IndexOutcome, IndexResource, ParameterLevel, ResolvedParameter, SearchEngine, SearchOptions,
    SearchParameterResolve, SearchReturn,
    elastic_search::is_mapped_search_parameter_type,
    indexing_conversion::{self, InsertableIndex},
    memory::R4_SEARCH_PARAMETERS_INDEX,
    pg_search::schema::{SchemaRegistry, generate_schemas},
};

mod indexing;
pub mod migration;
pub mod schema;
mod search;
pub mod search_parameter_resolver;

#[derive(OperationOutcomeError, Debug)]
pub enum PgSearchError {
    #[fatal(
        code = "exception",
        diagnostic = "Failed to evaluate fhirpath expression."
    )]
    FHIRPathError(#[from] haste_fhirpath::FHIRPathError),
    #[fatal(
        code = "exception",
        diagnostic = "PG search does not support the fhir method: '{arg0:?}'"
    )]
    UnsupportedFHIRMethod(FHIRMethod),
    #[fatal(code = "exception", diagnostic = "PG search database error: '{arg0}'")]
    SqlxError(String),
}

impl From<sqlx::Error> for PgSearchError {
    fn from(e: sqlx::Error) -> Self {
        PgSearchError::SqlxError(e.to_string())
    }
}

#[derive(Clone)]
pub struct PgSearchEngine<SearchParameterResolver: SearchParameterResolve + 'static> {
    parameter_resolver: Arc<SearchParameterResolver>,
    fp_engine: Arc<FPEngine>,
    pool: Pool<Postgres>,
    /// Per-resource-type table layouts for the HL7 base search parameters.
    /// Derived once from the static R4 parameter set, which is the same source
    /// the migration builds the tables from, so the two can't drift.
    schema_registry: Arc<SchemaRegistry>,
}

/// The per-resource-type schemas for the R4 base search parameters. Built once
/// and shared by every engine instance — deriving them walks every HL7
/// SearchParameter, which is wasted work to repeat.
static R4_SCHEMA_REGISTRY: LazyLock<Arc<SchemaRegistry>> = LazyLock::new(|| {
    Arc::new(generate_schemas(
        &R4_SEARCH_PARAMETERS_INDEX.all_parameters(),
    ))
});

/// Creates a separate PostgreSQL connection pool for the search index database.
pub async fn create_pg_search_pool(
    database_url: &str,
    max_connections: u32,
) -> Result<Pool<Postgres>, OperationOutcomeError> {
    PgPoolOptions::new()
        .max_connections(max_connections)
        .connect(database_url)
        .await
        .map_err(|e| {
            OperationOutcomeError::fatal(
                IssueType::exception(),
                format!("Failed to create PG search database pool: {e}"),
            )
        })
}

impl<SearchParameterResolver: SearchParameterResolve + 'static>
    PgSearchEngine<SearchParameterResolver>
{
    pub fn new(
        parameter_resolver: Arc<SearchParameterResolver>,
        fp_engine: Arc<FPEngine>,
        pool: Pool<Postgres>,
    ) -> Self {
        PgSearchEngine {
            parameter_resolver,
            fp_engine,
            pool,
            schema_registry: R4_SCHEMA_REGISTRY.clone(),
        }
    }
}

/// A single resource's evaluated search values, split the way the hybrid
/// schema stores them.
pub(crate) struct ResourceSearchIndex {
    /// System-level parameters, keyed by search parameter `code`. These land
    /// in dedicated columns on the per-resource-type table, which is looked up
    /// by code rather than by URL.
    pub system_entries: Vec<(String, InsertableIndex)>,
    /// Project-level parameters, keyed by canonical URL. These land in the
    /// `search_dynamic_*` EAV tables, where `param_url` discriminates them.
    pub dynamic_entries: Vec<(String, InsertableIndex)>,
}

/// Evaluates FHIRPath expressions for all applicable search parameters and
/// converts results into `InsertableIndex` values.
///
/// Backend-agnostic beyond the routing: reuses
/// `indexing_conversion::to_insertable_index` and `FPEngine` from the shared
/// crate, then splits the results by `ParameterLevel` so each half can be
/// written to the storage that fits it.
pub(crate) async fn resource_to_search_index(
    fp_engine: Arc<FPEngine>,
    parameters: &[ResolvedParameter],
    resource: &Resource,
) -> Result<ResourceSearchIndex, OperationOutcomeError> {
    let mut system_entries = Vec::new();
    let mut dynamic_entries = Vec::new();

    for param in parameters {
        if let Some(expression) = param
            .search_parameter
            .expression
            .as_ref()
            .and_then(|e| e.value.as_ref())
            && let Some(url) = param.search_parameter.url.value.as_ref()
        {
            // A parameter of an unmapped type (composite, special, ...) has
            // nowhere to be written, so evaluating it would only discard the
            // result.
            if !is_mapped_search_parameter_type(&param.search_parameter.type_) {
                continue;
            }

            let result = fp_engine
                .evaluate(expression, vec![resource])
                .await
                .map_err(PgSearchError::from);

            if let Err(err) = result {
                tracing::error!(
                    "Failed to evaluate FHIRPath expression: '{}' for resource.",
                    expression,
                );
                return Err(err.into());
            }

            let insertable = indexing_conversion::to_insertable_index(
                param,
                &result?.iter().collect::<Vec<_>>(),
            )?;

            match &param.level {
                // Keyed by code: the per-resource-type table's columns are
                // derived from the code, not the URL.
                ParameterLevel::System => {
                    if let Some(code) = param.search_parameter.code.value.as_ref() {
                        system_entries.push((code.clone(), insertable));
                    }
                }
                ParameterLevel::Project => {
                    dynamic_entries.push((url.clone(), insertable));
                }
            }
        }
    }

    Ok(ResourceSearchIndex {
        system_entries,
        dynamic_entries,
    })
}

impl<SearchParameterResolver: SearchParameterResolve> SearchEngine
    for PgSearchEngine<SearchParameterResolver>
{
    async fn search(
        &self,
        _fhir_version: &SupportedFHIRVersions,
        tenant: &TenantId,
        project: &ProjectId,
        search_request: &SearchRequest,
        options: Option<SearchOptions>,
    ) -> Result<SearchReturn, OperationOutcomeError> {
        search::execute_search(
            &self.pool,
            self.parameter_resolver.clone(),
            &self.schema_registry,
            tenant,
            project,
            search_request,
            options.as_ref(),
        )
        .await
    }

    async fn index(
        &self,
        _fhir_version: SupportedFHIRVersions,
        resources: Vec<IndexResource>,
    ) -> Result<IndexOutcome, OperationOutcomeError> {
        indexing::index_resources(
            &self.pool,
            &self.parameter_resolver,
            &self.schema_registry,
            self.fp_engine.clone(),
            resources,
        )
        .await
    }

    async fn migrate(
        &self,
        _fhir_version: &SupportedFHIRVersions,
    ) -> Result<(), OperationOutcomeError> {
        migration::run_migration(&self.pool, &self.schema_registry).await
    }
}
