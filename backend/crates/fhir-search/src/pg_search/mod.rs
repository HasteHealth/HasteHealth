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

#[derive(Clone)]
pub struct PgSearchEngine<SearchParameterResolver: SearchParameterResolve + 'static> {
    parameter_resolver: Arc<SearchParameterResolver>,
    fp_engine: Arc<FPEngine>,
    pool: Pool<Postgres>,
    /// Table layouts for the HL7 base parameters, derived from the same static
    /// R4 set the migration builds the tables from, so the two can't drift.
    schema_registry: Arc<SchemaRegistry>,
}

/// Shared by every engine instance: deriving it walks every HL7
/// `SearchParameter`.
static R4_SCHEMA_REGISTRY: LazyLock<Arc<SchemaRegistry>> = LazyLock::new(|| {
    Arc::new(generate_schemas(
        SupportedFHIRVersions::R4,
        &R4_SEARCH_PARAMETERS_INDEX.all_parameters(),
    ))
});

impl<Resolver: SearchParameterResolve> PgSearchEngine<Resolver> {
    /// Whether the search database answers, for callers that wait on it at
    /// startup.
    ///
    /// # Errors
    ///
    /// Returns an error if the pool cannot serve a statement.
    pub async fn is_connected(&self) -> Result<(), OperationOutcomeError> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await
            .map(|_| ())
            .map_err(|e| {
                OperationOutcomeError::fatal(
                    haste_fhir_model::r4::generated::terminology::IssueType::exception(),
                    format!("PG search database is not reachable: {e}"),
                )
            })
    }
}

/// Creates a separate connection pool for the search index database.
///
/// # Errors
///
/// Returns an error if the pool cannot open its first connection.
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

/// One resource's evaluated search values, split the way the schema stores
/// them.
pub(crate) struct ResourceSearchIndex {
    /// Single-valued parameters, keyed by `code`: a column on the resource
    /// type's own table.
    pub system_entries: Vec<(String, InsertableIndex)>,
    /// Everything that may repeat, keyed by canonical URL: a row in the shared
    /// `{version}_param_{type}_idx` table, discriminated by a hash of the URL.
    pub dynamic_entries: Vec<(String, InsertableIndex)>,
}

/// Evaluates every applicable parameter's `FHIRPath` expression and splits the
/// results by where they are stored.
pub(crate) async fn resource_to_search_index(
    fp_engine: Arc<FPEngine>,
    parameters: &[ResolvedParameter],
    resource: &Resource,
    resource_type: &str,
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
            // An unmapped type (composite, special, ...) has nowhere to be
            // written.
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
                // A column exists only where a parameter cannot produce a
                // second value, so cardinality — not level — decides the
                // destination. Columns are named after the code, not the URL.
                ParameterLevel::System
                    if search_parameter_cardinality::is_single_valued(url, resource_type) =>
                {
                    if let Some(code) = param.search_parameter.code.value.as_ref() {
                        system_entries.push((code.clone(), insertable));
                    }
                }
                ParameterLevel::System | ParameterLevel::Project => {
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
