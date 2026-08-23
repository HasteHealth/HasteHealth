use crate::{
    IndexFailure, IndexOutcome, IndexResource, ParameterLevel, ResolvedParameter, SearchEngine,
    SearchOptions, SearchParameterResolve, SearchReturn,
    indexing_conversion::{self, InsertableIndex},
};
use elasticsearch::{
    BulkOperation, BulkParts, Elasticsearch,
    auth::Credentials,
    cert::CertificateValidation,
    http::{
        Url,
        transport::{BuildError, SingleNodeConnectionPool, TransportBuilder},
    },
};
use haste_fhir_client::request::SearchRequest;
use haste_fhir_model::r4::generated::{
    resources::{Resource, ResourceType},
    terminology::IssueType,
};
use haste_fhir_operation_error::{OperationOutcomeError, derive::OperationOutcomeError};
use haste_fhirpath::FPEngine;
use haste_jwt::{ProjectId, ResourceId, TenantId, VersionId};
use haste_repository::types::{FHIRMethod, SupportedFHIRVersions};
use serde::Deserialize;
use std::{collections::HashMap, sync::Arc};

mod migration;
mod search;
pub mod search_parameter_resolver;

#[derive(Deserialize, Debug)]
struct SearchEntryPrivate {
    pub id: Vec<ResourceId>,
    pub resource_type: Vec<ResourceType>,
    pub version_id: Vec<VersionId>,
    pub project: Vec<ProjectId>,
}

static DYNAMIC_PARAMETER_INDEX_FIELD: &str = "dynamic_parameters";

/// Elasticsearch treats a literal `.` in a mapped field name as an object
/// path separator: e.g. mapping the field
/// `http://hl7.org/fhir/SearchParameter/Patient-name` silently produces
/// nested objects (`http://hl7` -> `org/fhir/SearchParameter/Patient-name`),
/// even though `/` is left alone. Search parameter canonical URLs almost
/// always contain a dotted domain, so every active (top-level) search
/// parameter needs its dots stripped to stay a genuinely flat field
/// consistently in the index mapping, in the documents written to it, and in
/// queries built against it.
fn flatten_parameter_field_name(url: &str) -> String {
    url.replace('.', "_")
}

#[derive(OperationOutcomeError, Debug)]
pub enum SearchError {
    #[fatal(
        code = "exception",
        diagnostic = "Failed to evaluate fhirpath expression."
    )]
    FHIRPathError(#[from] haste_fhirpath::FHIRPathError),
    #[fatal(
        code = "exception",
        diagnostic = "Search does not support the fhir method: '{arg0:?}'"
    )]
    UnsupportedFHIRMethod(FHIRMethod),
    #[fatal(
        code = "exception",
        diagnostic = "Failed to index resources server responded with status code: '{arg0}'"
    )]
    Fatal(u16),
    #[fatal(
        code = "exception",
        diagnostic = "Elasticsearch server failed to index: '{arg0}'"
    )]
    ElasticsearchError(#[from] elasticsearch::Error),
    #[fatal(
        code = "exception",
        diagnostic = "Elasticsearch server responded with an error: '{arg0}'"
    )]
    ElasticSearchResponseError(u16),
    NotConnected,
}

#[derive(OperationOutcomeError, Debug)]
pub enum SearchConfigError {
    #[fatal(code = "exception", diagnostic = "Failed to parse URL: '{arg0}'.")]
    UrlParseError(String),
    #[fatal(
        code = "exception",
        diagnostic = "Elasticsearch client creation failed."
    )]
    ElasticSearchConfigError(#[from] BuildError),
    #[fatal(
        code = "exception",
        diagnostic = "Unsupported FHIR version for index: '{arg0}'"
    )]
    UnsupportedIndex(SupportedFHIRVersions),
}

#[derive(Clone)]
pub struct ElasticSearchEngine<SearchParameterResolver: SearchParameterResolve + 'static> {
    parameter_resolver: Arc<SearchParameterResolver>,
    fp_engine: Arc<FPEngine>,
    client: Arc<Elasticsearch>,
    /// Whether `migrate` is allowed to rebuild the index (briefly making it
    /// unavailable) to drop columns for search parameters that no longer
    /// exist. When `false`, `migrate` only logs which parameters would be
    /// dropped.
    prune_removed_search_parameters: bool,
}

/// Creates an Elasticsearch client using the provided URL and credentials.
///
/// # Errors
///
/// Returns [`SearchConfigError::UrlParseError`] if url is not a valid URL.
///
/// Returns a [`SearchConfigError`] if the Elasticsearch transport cannot be
/// constructed.
pub fn create_es_client(
    url: &str,
    username: String,
    password: String,
) -> Result<Arc<Elasticsearch>, SearchConfigError> {
    let url = Url::parse(url).map_err(|_e| SearchConfigError::UrlParseError(url.to_string()))?;
    let conn_pool = SingleNodeConnectionPool::new(url);
    let transport = TransportBuilder::new(conn_pool)
        .cert_validation(CertificateValidation::None)
        .auth(Credentials::Basic(username, password))
        .build()?;

    let elasticsearch_client = Elasticsearch::new(transport);

    Ok(Arc::new(elasticsearch_client))
}

type Tasks = tokio::task::JoinHandle<(
    IndexResource,
    Result<BulkOperation<HashMap<String, InsertableIndex>>, OperationOutcomeError>,
)>;

struct CollectedOperations {
    bulk_ops: Vec<BulkOperation<HashMap<String, InsertableIndex>>>,
    /// Parallel to `bulk_ops` (same order) - the resource each bulk op was
    /// built from, kept so a failed Elasticsearch bulk item can be attributed
    /// back to the resource that produced it.
    sent_resources: Vec<IndexResource>,
    failed: Vec<IndexFailure>,
}

/// See <https://www.elastic.co/docs/api/doc/elasticsearch/operation/operation-bulk>
#[derive(Deserialize, Debug)]
struct BulkResponse {
    /// The length of time, in milliseconds, it took to process the bulk request.
    #[allow(dead_code)]
    took: u64,
    /// The result of each operation in the bulk request, in the order they were submitted.
    items: Vec<serde_json::Value>,
    /// If true, one or more of the operations in the bulk request did not complete successfully.
    #[allow(dead_code)]
    errors: bool,
}

/// Processes the Elasticsearch bulk response, attributing each per-item result
/// back to the resource that produced it. Elasticsearch indexes bulk items
/// independently, so one bad document does not prevent the rest of the batch
/// from succeeding - only a request-level failure (unreachable cluster, malformed response) surfaces as `Err` here.
fn process_bulk_response(
    bulk_response: &BulkResponse,
    sent_resources: Vec<IndexResource>,
) -> Result<IndexOutcome, OperationOutcomeError> {
    if bulk_response.items.len() != sent_resources.len() {
        return Err(OperationOutcomeError::fatal(
            IssueType::exception(),
            format!(
                "Elasticsearch bulk response item count '{}' did not match request count '{}'.",
                bulk_response.items.len(),
                sent_resources.len()
            ),
        ));
    }

    let mut succeeded = 0;
    let mut failed = Vec::new();

    // Per documentation: The result of each operation in the bulk request, in the order they were submitted.
    // See https://www.elastic.co/docs/api/doc/elasticsearch/operation/operation-bulk.

    for (item, resource) in bulk_response.items.iter().zip(sent_resources) {
        // Each item is a single-key object, e.g. `{"index": {...}}` or `{"delete": {...}}`.
        if let Some(op_result) = item.as_object().and_then(|o| o.values().next()) {
            match op_result["status"].as_u64() {
                Some(status) if (200..300).contains(&status) => succeeded += 1,
                status => {
                    let reason = op_result["error"]["reason"]
                        .as_str()
                        .unwrap_or("unknown error");
                    failed.push(IndexFailure {
                        error: OperationOutcomeError::fatal(
                            IssueType::exception(),
                            format!("Elasticsearch indexing failed (status {status:?}): {reason}"),
                        ),
                        resource,
                    });
                }
            }
        } else {
            failed.push(IndexFailure {
                error: OperationOutcomeError::fatal(
                    IssueType::exception(),
                    format!("Unexpected Elasticsearch bulk item shape: '{item}'"),
                ),
                resource,
            });
        }
    }

    if !failed.is_empty() {
        tracing::error!(
            "Elasticsearch bulk index reported {} failed item(s) out of {}.",
            failed.len(),
            bulk_response.items.len()
        );
    }

    Ok(IndexOutcome { succeeded, failed })
}

impl<SearchParameterResolver: SearchParameterResolve + 'static>
    ElasticSearchEngine<SearchParameterResolver>
{
    pub fn new(
        parameter_resolver: Arc<SearchParameterResolver>,
        fp_engine: Arc<FPEngine>,
        es_client: Arc<Elasticsearch>,
        prune_removed_search_parameters: bool,
    ) -> Self {
        ElasticSearchEngine {
            parameter_resolver,
            fp_engine,
            client: es_client,
            prune_removed_search_parameters,
        }
    }

    /// Checks whether the Elasticsearch client is connected.
    ///
    /// # Errors
    ///
    /// Returns [`SearchError::NotConnected`] if Elasticsearch responds with a
    /// non-success status code.
    ///
    /// Returns a [`SearchError`] if the ping request fails.
    pub async fn is_connected(&self) -> Result<(), SearchError> {
        let res = self.client.ping().send().await.map_err(SearchError::from)?;

        if res.status_code().is_success() {
            Ok(())
        } else {
            Err(SearchError::NotConnected)
        }
    }

    /// Sends one Elasticsearch `_bulk` request and attributes each per-item
    /// result back to the resource that produced it. Elasticsearch indexes
    /// bulk items independently, so one bad document does not prevent the
    /// rest of the batch from succeeding - only a request-level failure
    /// (unreachable cluster, malformed response) surfaces as `Err` here.
    async fn send_bulk_operations(
        &self,
        search_index_name: &'static str,
        bulk_ops: Vec<BulkOperation<HashMap<String, InsertableIndex>>>,
        sent_resources: Vec<IndexResource>,
    ) -> Result<IndexOutcome, OperationOutcomeError> {
        if bulk_ops.is_empty() {
            return Ok(IndexOutcome {
                succeeded: 0,
                failed: Vec::new(),
            });
        }

        let res = self
            .client
            .bulk(BulkParts::Index(search_index_name))
            .body(bulk_ops)
            .send()
            .await
            .map_err(SearchError::from)?;

        let response_body = res.json::<BulkResponse>().await.map_err(|_e| {
            OperationOutcomeError::fatal(
                IssueType::exception(),
                "Failed to parse response body.".to_string(),
            )
        })?;

        process_bulk_response(&response_body, sent_resources)
    }

    fn spawn_index_tasks(
        &self,
        resources: Vec<IndexResource>,
        search_index_name: &'static str,
    ) -> Vec<Tasks> {
        resources
            .into_iter()
            .map(|r| {
                let engine = self.fp_engine.clone();
                let parameter_resolver = self.parameter_resolver.clone();

                tokio::spawn(async move {
                    Self::build_bulk_operation(engine, parameter_resolver, r, search_index_name)
                        .await
                })
            })
            .collect()
    }

    /// Awaits every bulk-op-building task, separating resources that built
    /// successfully from those that failed. A single resource's failure
    /// (bad `FHIRPath` expression, unsupported method, etc.) is captured with
    /// its identity rather than aborting the rest of the batch. A `JoinError`
    /// (the task itself panicked) has no resource to attribute the failure
    /// to, so it aborts the whole call.
    async fn collect_bulk_operations(
        &self,
        tasks: Vec<Tasks>,
    ) -> Result<CollectedOperations, OperationOutcomeError> {
        tracing::trace!("Awaiting {} indexing tasks.", tasks.len());

        let mut bulk_ops = Vec::with_capacity(tasks.len());
        let mut sent_resources = Vec::with_capacity(tasks.len());
        let mut failed = Vec::new();

        for task in tasks {
            let (resource, result) = task
                .await
                .map_err(|e| OperationOutcomeError::fatal(IssueType::exception(), e.to_string()))?;

            match result {
                Ok(bulk_op) => {
                    sent_resources.push(resource);
                    bulk_ops.push(bulk_op);
                }
                Err(error) => failed.push(IndexFailure { resource, error }),
            }
        }

        Ok(CollectedOperations {
            bulk_ops,
            sent_resources,
            failed,
        })
    }

    async fn build_bulk_operation<ParameterResolver: SearchParameterResolve>(
        engine: Arc<FPEngine>,
        parameter_resolver: Arc<ParameterResolver>,
        resource: IndexResource,
        search_index_name: &'static str,
    ) -> (
        IndexResource,
        Result<BulkOperation<HashMap<String, InsertableIndex>>, OperationOutcomeError>,
    ) {
        let result = match &resource.fhir_method {
            FHIRMethod::Create | FHIRMethod::Update => {
                Self::build_index_operation(
                    engine,
                    parameter_resolver,
                    &resource,
                    search_index_name,
                )
                .await
            }

            FHIRMethod::Delete => Ok(BulkOperation::delete(unique_index_id(
                &resource.tenant,
                &resource.project,
                &resource.resource_type,
                &resource.id,
            ))
            .index(search_index_name)
            .into()),

            method @ FHIRMethod::Read => Err(OperationOutcomeError::from(
                SearchError::UnsupportedFHIRMethod((*method).clone()),
            )),
        };

        (resource, result)
    }

    async fn build_index_operation<ParameterResolver: SearchParameterResolve>(
        engine: Arc<FPEngine>,
        parameter_resolver: Arc<ParameterResolver>,
        resource: &IndexResource,
        search_index_name: &'static str,
    ) -> Result<BulkOperation<HashMap<String, InsertableIndex>>, OperationOutcomeError> {
        // Id is not sufficient because different Resourcetypes may have the same id.
        // Additionally should be namespaced by tenant and project to avoid conflicts across tenants and projects.
        let index_id = unique_index_id(
            &resource.tenant,
            &resource.project,
            &resource.resource_type,
            &resource.id,
        );

        let params = parameter_resolver
            .by_resource_type(&resource.tenant, &resource.project, &resource.resource_type)
            .await?;

        let mut elastic_index =
            resource_to_elastic_index(engine, &params, &resource.resource).await?;

        Self::add_index_metadata(&mut elastic_index, resource);

        Ok(BulkOperation::index(elastic_index)
            .id(index_id)
            .index(search_index_name)
            .into())
    }

    fn add_index_metadata(
        elastic_index: &mut HashMap<String, InsertableIndex>,
        resource: &IndexResource,
    ) {
        elastic_index.insert(
            "resource_type".to_string(),
            InsertableIndex::Meta(resource.resource_type.as_ref().to_string()),
        );

        elastic_index.insert(
            "id".to_string(),
            InsertableIndex::Meta(resource.id.as_ref().to_string()),
        );

        elastic_index.insert(
            "version_id".to_string(),
            InsertableIndex::Meta(resource.version_id.as_ref().to_string()),
        );

        elastic_index.insert(
            "project".to_string(),
            InsertableIndex::Meta(resource.project.as_ref().to_string()),
        );

        elastic_index.insert(
            "tenant".to_string(),
            InsertableIndex::Meta(resource.tenant.as_ref().to_string()),
        );
    }
}

async fn resource_to_elastic_index(
    fp_engine: Arc<FPEngine>,
    parameters: &[ResolvedParameter],
    resource: &Resource,
) -> Result<HashMap<String, InsertableIndex>, OperationOutcomeError> {
    let mut map = HashMap::new();
    let mut dynamic_parameters = HashMap::new();
    for param in parameters {
        if let Some(expression) = param
            .search_parameter
            .expression
            .as_ref()
            .and_then(|e| e.value.as_ref())
            && let Some(url) = param.search_parameter.url.value.as_ref()
        {
            let result = fp_engine
                .evaluate(expression, vec![resource])
                .await
                .map_err(SearchError::from);

            if let Err(err) = result {
                tracing::error!(
                    "Failed to evaluate FHIRPath expression: '{}' for resource.",
                    expression,
                );

                return Err(err.into());
            }

            let result_vec = indexing_conversion::to_insertable_index(
                param,
                &result?.iter().collect::<Vec<_>>(),
            )?;

            match param.level {
                ParameterLevel::System => {
                    map.insert(flatten_parameter_field_name(url), result_vec);
                }
                // Project Parameters are indexed using a single JS Object which gets indexed to
                ParameterLevel::Project => {
                    dynamic_parameters.insert(url.clone(), result_vec);
                }
            }
        }
    }

    // Various project level parameters. These are indexed under a single field in elasticsearch as a nested type with url and indexed value..
    map.insert(
        DYNAMIC_PARAMETER_INDEX_FIELD.to_string(),
        InsertableIndex::DynamicParameters(dynamic_parameters),
    );

    Ok(map)
}

#[allow(dead_code)]
static R4_FHIR_INDEX_V1: &str = "r4_search_index";
static R4_FHIR_INDEX_V2: &str = "r4_search_index_v2";

#[must_use]
pub const fn get_index_name() -> &'static str {
    R4_FHIR_INDEX_V2
}

#[derive(serde::Deserialize, Debug)]
struct ElasticSearchHitResult {
    _index: String,
    _id: String,
    _score: Option<f64>,
    fields: SearchEntryPrivate,
}

#[derive(serde::Deserialize, Debug)]
struct ElasticSearchHitTotalMeta {
    value: i64,
    // relation: String,
}

#[derive(serde::Deserialize, Debug)]
struct ElasticSearchHit {
    total: Option<ElasticSearchHitTotalMeta>,
    hits: Vec<ElasticSearchHitResult>,
}

#[derive(serde::Deserialize, Debug)]
struct ElasticSearchResponse {
    hits: ElasticSearchHit,
}

fn unique_index_id(
    tenant: &TenantId,
    project: &ProjectId,
    resource_type: &ResourceType,
    id: &ResourceId,
) -> String {
    let unique_index_id = format!(
        "{}/{}/{}/{}",
        tenant.as_ref(),
        project.as_ref(),
        resource_type.as_ref(),
        id.as_ref()
    );

    unique_index_id
}

impl<SearchParameterResolver: SearchParameterResolve> SearchEngine
    for ElasticSearchEngine<SearchParameterResolver>
{
    async fn search(
        &self,
        _fhir_version: &SupportedFHIRVersions,
        tenant: &TenantId,
        project: &ProjectId,
        search_request: &SearchRequest,
        options: Option<SearchOptions>,
    ) -> Result<SearchReturn, haste_fhir_operation_error::OperationOutcomeError> {
        search::execute_search(
            self.client.clone(),
            self.parameter_resolver.clone(),
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
        let resources_total = resources.len();
        let search_index_name = get_index_name();

        tracing::trace!(
            "Indexing {} resources into index: '{}'",
            resources_total,
            search_index_name
        );

        let tasks = self.spawn_index_tasks(resources, search_index_name);
        let CollectedOperations {
            bulk_ops,
            sent_resources,
            mut failed,
        } = self.collect_bulk_operations(tasks).await?;

        tracing::trace!(
            "Bulk indexing {} resources into index: '{}'",
            bulk_ops.len(),
            search_index_name
        );

        let mut outcome = self
            .send_bulk_operations(search_index_name, bulk_ops, sent_resources)
            .await?;
        outcome.failed.append(&mut failed);

        Ok(outcome)
    }

    async fn migrate(
        &self,
        _fhir_version: &SupportedFHIRVersions,
    ) -> Result<(), haste_fhir_operation_error::OperationOutcomeError> {
        migration::create_mapping(
            self.parameter_resolver.clone(),
            &self.client,
            get_index_name(),
            self.prune_removed_search_parameters,
        )
        .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flatten_parameter_field_name_strips_dots_only() {
        assert_eq!(
            flatten_parameter_field_name("http://hl7.org/fhir/SearchParameter/Patient-name"),
            "http://hl7_org/fhir/SearchParameter/Patient-name"
        );
    }

    #[test]
    fn flatten_parameter_field_name_handles_multiple_dots() {
        assert_eq!(
            flatten_parameter_field_name("https://sub.acme.io/v1.2/x"),
            "https://sub_acme_io/v1_2/x"
        );
    }
}
