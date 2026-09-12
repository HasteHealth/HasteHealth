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
            self.parameter_resolver.as_ref(),
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

/// End-to-end index-then-search coverage against a live PostgreSQL instance.
///
/// Ignored by default since it needs a database; run with
/// `PG_SEARCH_TEST_URL=postgres://... cargo test -p haste-fhir-search --lib
/// pg_search::live -- --ignored --nocapture`.
#[cfg(test)]
mod live {
    use super::*;
    use crate::memory::R4_SEARCH_PARAMETERS_INDEX;
    use haste_fhir_client::{
        request::{FHIRSearchTypeRequest, SearchRequest},
        url::{Parameter, ParsedParameter, ParsedParameters},
    };
    use haste_fhir_model::r4::generated::resources::ResourceType;
    use haste_jwt::{ResourceId, VersionId};

    fn pool_url() -> String {
        std::env::var("PG_SEARCH_TEST_URL").expect("set PG_SEARCH_TEST_URL")
    }

    /// Builds a Patient from FHIR JSON, the same shape the server receives.
    fn patient(id: &str, family: &str, given: &str, identifier: &str) -> Resource {
        let json = serde_json::json!({
            "resourceType": "Patient",
            "id": id,
            "name": [{ "family": family, "given": [given] }],
            "identifier": [{ "system": "http://example.com/mrn", "value": identifier }],
            "gender": "female",
            "birthDate": "1980-05-17",
            "generalPractitioner": [{ "reference": "Practitioner/prac-1" }]
        });

        serde_json::from_value(json).expect("valid Patient JSON")
    }

    fn index_resource(id: &str, resource: Resource) -> IndexResource {
        IndexResource {
            id: ResourceId::new(id.to_string()),
            version_id: VersionId::new(format!("{id}-v1")),
            tenant: TenantId::new("t-live".to_string()),
            project: ProjectId::new("p-live".to_string()),
            fhir_method: FHIRMethod::Create,
            resource_type: ResourceType::Patient,
            resource,
        }
    }

    fn type_search(params: Vec<Parameter>) -> SearchRequest {
        SearchRequest::Type(FHIRSearchTypeRequest {
            resource_type: ResourceType::Patient,
            parameters: ParsedParameters::new(
                params.into_iter().map(ParsedParameter::Resource).collect(),
            ),
        })
    }

    #[tokio::test]
    #[ignore = "requires a live PostgreSQL instance"]
    async fn index_then_search_round_trip() {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(4)
            .connect(&pool_url())
            .await
            .expect("connect");

        let registry = Arc::new(generate_schemas(
            &R4_SEARCH_PARAMETERS_INDEX.all_parameters(),
        ));
        migration::run_migration(&pool, &registry)
            .await
            .expect("migrate");

        let resolver = R4_SEARCH_PARAMETERS_INDEX.clone();
        let fp_engine = Arc::new(FPEngine::new());

        let outcome = indexing::index_resources(
            &pool,
            resolver.as_ref(),
            &registry,
            fp_engine,
            vec![
                index_resource("live-1", patient("live-1", "Smith", "Jane", "MRN-1")),
                index_resource("live-2", patient("live-2", "Jones", "Bob", "MRN-2")),
            ],
        )
        .await
        .expect("index");

        for failure in &outcome.failed {
            eprintln!("index failure: {:?}", failure.error);
        }
        assert_eq!(outcome.succeeded, 2, "both resources should index");

        let tenant = TenantId::new("t-live".to_string());
        let project = ProjectId::new("p-live".to_string());

        let search = async |params: Vec<Parameter>| {
            search::execute_search(
                &pool,
                resolver.clone(),
                &registry,
                &tenant,
                &project,
                &type_search(params),
                None,
            )
            .await
            .expect("search")
        };

        // String prefix on a dedicated column.
        let by_family = search(vec![Parameter {
            name: "family".to_string(),
            value: vec!["Smi".to_string()],
            modifier: None,
            chains: None,
        }])
        .await;
        assert_eq!(by_family.entries.len(), 1, "family=Smi matches one patient");
        assert_eq!(by_family.entries[0].id.as_ref(), "live-1");

        // Token system|code across parallel columns.
        let by_identifier = search(vec![Parameter {
            name: "identifier".to_string(),
            value: vec!["http://example.com/mrn|MRN-2".to_string()],
            modifier: None,
            chains: None,
        }])
        .await;
        assert_eq!(by_identifier.entries.len(), 1);
        assert_eq!(by_identifier.entries[0].id.as_ref(), "live-2");

        // A token that matches neither resource.
        let no_match = search(vec![Parameter {
            name: "identifier".to_string(),
            value: vec!["http://example.com/mrn|NOPE".to_string()],
            modifier: None,
            chains: None,
        }])
        .await;
        assert!(no_match.entries.is_empty());

        // Date range over parallel start/end columns.
        let by_birthdate = search(vec![Parameter {
            name: "birthdate".to_string(),
            value: vec!["1980-05-17".to_string()],
            modifier: None,
            chains: None,
        }])
        .await;
        assert_eq!(by_birthdate.entries.len(), 2, "both share a birthdate");

        // Two clauses must AND together.
        let combined = search(vec![
            Parameter {
                name: "family".to_string(),
                value: vec!["Jones".to_string()],
                modifier: None,
                chains: None,
            },
            Parameter {
                name: "gender".to_string(),
                value: vec!["female".to_string()],
                modifier: None,
                chains: None,
            },
        ])
        .await;
        assert_eq!(combined.entries.len(), 1);
        assert_eq!(combined.entries[0].id.as_ref(), "live-2");

        // No clauses at all still returns the rows, which is what proves the
        // per-resource-type row is written even for a resource with no
        // indexable values.
        let all = search(vec![]).await;
        assert_eq!(all.entries.len(), 2);

        // Re-indexing replaces rather than duplicating.
        indexing::index_resources(
            &pool,
            resolver.as_ref(),
            &registry,
            Arc::new(FPEngine::new()),
            vec![index_resource(
                "live-1",
                patient("live-1", "Smithson", "Jane", "MRN-1"),
            )],
        )
        .await
        .expect("reindex");

        let after_reindex = search(vec![]).await;
        assert_eq!(after_reindex.entries.len(), 2, "reindex must not duplicate");

        let renamed = search(vec![Parameter {
            name: "family".to_string(),
            value: vec!["Smithson".to_string()],
            modifier: None,
            chains: None,
        }])
        .await;
        assert_eq!(renamed.entries.len(), 1, "updated value is searchable");

        // A reference resolves through the dedicated parallel columns.
        let by_gp = search(vec![Parameter {
            name: "general-practitioner".to_string(),
            value: vec!["Practitioner/prac-1".to_string()],
            modifier: None,
            chains: None,
        }])
        .await;
        assert_eq!(by_gp.entries.len(), 2, "both patients share a practitioner");

        // System-level references are also mirrored into the shared reference
        // table, so reverse lookups have somewhere to scan.
        let mirrored: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM search_dynamic_reference \
             WHERE tenant = 't-live' AND target_resource_type = 'Practitioner' \
             AND target_id = 'prac-1'",
        )
        .fetch_one(&pool)
        .await
        .expect("count mirrored references");
        assert!(mirrored > 0, "system-level references must be mirrored");
        println!("mirrored reference rows: {mirrored}");

        // Clean up so the test can be re-run.
        sqlx::query("DELETE FROM search_resource WHERE tenant = 't-live'")
            .execute(&pool)
            .await
            .expect("cleanup");
    }
}
