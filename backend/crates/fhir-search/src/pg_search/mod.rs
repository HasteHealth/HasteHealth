use std::sync::{Arc, LazyLock};

use haste_fhir_client::request::SearchRequest;
use haste_fhir_model::r4::generated::{resources::Resource, terminology::IssueType};
use haste_fhir_operation_error::{OperationOutcomeError, derive::OperationOutcomeError};
use haste_fhirpath::FPEngine;
use haste_jwt::{ProjectId, TenantId};
use haste_repository::types::{FHIRMethod, SupportedFHIRVersions};
use sqlx::{Pool, Postgres, postgres::PgPoolOptions};

use crate::search_parameter_cardinality;
use crate::{
    IndexOutcome, IndexResource, ParameterLevel, ResolvedParameter, SearchEngine, SearchOptions,
    SearchParameterResolve, SearchReturn,
    elastic_search::is_mapped_search_parameter_type,
    indexing_conversion::{self, InsertableIndex},
    memory::R4_SEARCH_PARAMETERS_INDEX,
    pg_search::schema::{SchemaRegistry, generate_schemas},
};

mod indexing;
pub mod keys;
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

/// [`SearchEngine`] backed by PostgreSQL. A thin handle: the work is done by
/// the free functions in [`search`], [`indexing`] and [`migration`].
#[derive(Clone)]
pub struct PgSearchEngine<SearchParameterResolver: SearchParameterResolve + 'static> {
    parameter_resolver: Arc<SearchParameterResolver>,
    fp_engine: Arc<FPEngine>,
    pool: Pool<Postgres>,
    /// Built from the same R4 parameters the migration uses, so the two agree.
    schema_registry: Arc<SchemaRegistry>,
}

/// Built once per process; it walks every HL7 `SearchParameter`.
static R4_SCHEMA_REGISTRY: LazyLock<Arc<SchemaRegistry>> = LazyLock::new(|| {
    Arc::new(generate_schemas(
        SupportedFHIRVersions::R4,
        &R4_SEARCH_PARAMETERS_INDEX.all_parameters(),
    ))
});

impl<Resolver: SearchParameterResolve + 'static> PgSearchEngine<Resolver> {
    pub fn new(
        parameter_resolver: Arc<Resolver>,
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

    /// Checks the search database responds, for startup waits.
    ///
    /// # Errors
    ///
    /// Returns an error if the pool cannot run a statement.
    pub async fn is_connected(&self) -> Result<(), OperationOutcomeError> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await
            .map(|_| ())
            .map_err(|e| {
                OperationOutcomeError::fatal(
                    IssueType::exception(),
                    format!("PG search database is not reachable: {e}"),
                )
            })
    }
}

/// Creates the search index's own connection pool.
///
/// # Errors
///
/// Returns an error if the first connection cannot be opened.
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

/// One resource's evaluated search values, split by storage.
#[derive(Default)]
pub(crate) struct ResourceSearchIndex {
    /// Singular system parameters, keyed by `code`: type table or anchor
    /// columns.
    pub system_entries: Vec<(String, InsertableIndex)>,
    /// Possibly repeating parameters, keyed by URL: shared-table rows.
    pub dynamic_entries: Vec<(String, InsertableIndex)>,
}

/// Evaluates each parameter's `FHIRPath` expression against `resource` and
/// splits the results by where they are stored.
pub(crate) async fn resource_to_search_index(
    fp_engine: Arc<FPEngine>,
    parameters: &[ResolvedParameter],
    resource: &Resource,
    resource_type: &str,
) -> Result<ResourceSearchIndex, OperationOutcomeError> {
    let mut index = ResourceSearchIndex::default();

    for (param, expression, url) in parameters.iter().filter_map(indexable) {
        let evaluated = fp_engine
            .evaluate(expression, vec![resource])
            .await
            .map_err(|e| {
                tracing::error!(
                    "Failed to evaluate FHIRPath expression: '{}' for resource.",
                    expression,
                );
                PgSearchError::from(e)
            })?;

        let insertable =
            indexing_conversion::to_insertable_index(param, &evaluated.iter().collect::<Vec<_>>())?;

        // Cardinality, not level, decides the destination: a column only
        // exists where a second value is impossible.
        match param.level {
            ParameterLevel::System
                if search_parameter_cardinality::is_single_valued(url, resource_type) =>
            {
                if let Some(code) = param.search_parameter.code.value.as_ref() {
                    index.system_entries.push((code.clone(), insertable));
                }
            }
            ParameterLevel::System | ParameterLevel::Project => {
                index.dynamic_entries.push((url.clone(), insertable));
            }
        }
    }

    Ok(index)
}

/// `(parameter, expression, url)` for a parameter that can be stored: it has
/// both, and a mapped type (not composite or special).
fn indexable(param: &ResolvedParameter) -> Option<(&ResolvedParameter, &String, &String)> {
    let search_parameter = &param.search_parameter;
    let expression = search_parameter.expression.as_ref()?.value.as_ref()?;
    let url = search_parameter.url.value.as_ref()?;

    is_mapped_search_parameter_type(&search_parameter.type_).then_some((param, expression, url))
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
