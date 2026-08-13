use crate::{
    IndexResource, ParameterLevel, ResolvedParameter, SearchEngine, SearchOptions,
    SearchParameterResolve, SearchReturn, SuccessfullyIndexedCount,
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
        diagnostic = "Elasticsearch server failed to index."
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

type Tasks = tokio::task::JoinHandle<
    Result<BulkOperation<HashMap<String, InsertableIndex>>, OperationOutcomeError>,
>;

impl<SearchParameterResolver: SearchParameterResolve + 'static>
    ElasticSearchEngine<SearchParameterResolver>
{
    pub fn new(
        parameter_resolver: Arc<SearchParameterResolver>,
        fp_engine: Arc<FPEngine>,
        es_client: Arc<Elasticsearch>,
    ) -> Self {
        ElasticSearchEngine {
            parameter_resolver,
            fp_engine,
            client: es_client,
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

    async fn send_bulk_operations(
        &self,
        search_index_name: &'static str,
        bulk_ops: Vec<BulkOperation<HashMap<String, InsertableIndex>>>,
    ) -> Result<SuccessfullyIndexedCount, OperationOutcomeError> {
        if bulk_ops.is_empty() {
            return Ok(SuccessfullyIndexedCount(0));
        }

        let res = self
            .client
            .bulk(BulkParts::Index(search_index_name))
            .body(bulk_ops)
            .send()
            .await
            .map_err(SearchError::from)?;

        let response_body = res.json::<serde_json::Value>().await.map_err(|_e| {
            OperationOutcomeError::fatal(
                IssueType::exception(),
                "Failed to parse response body.".to_string(),
            )
        })?;

        if response_body["errors"].as_bool().unwrap() {
            tracing::error!("Failed to index resources. Response: '{:?}'", response_body);
            return Err(SearchError::Fatal(500).into());
        }

        Ok(SuccessfullyIndexedCount(
            response_body["items"].as_array().unwrap().len(),
        ))
    }

    fn spawn_index_tasks(
        &self,
        resources: Vec<IndexResource>,
        search_index_name: &'static str,
    ) -> Vec<Tasks> {
        resources
            .into_iter()
            .filter(|r| {
                matches!(
                    r.fhir_method,
                    FHIRMethod::Create | FHIRMethod::Update | FHIRMethod::Delete
                )
            })
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

    async fn collect_bulk_operations(
        &self,
        tasks: Vec<Tasks>,
        resources_total: usize,
    ) -> Result<Vec<BulkOperation<HashMap<String, InsertableIndex>>>, OperationOutcomeError> {
        tracing::trace!("Awaiting {} indexing tasks.", tasks.len());

        let mut bulk_ops = Vec::with_capacity(resources_total);

        for task in tasks {
            let res = task.await.map_err(|e| {
                OperationOutcomeError::fatal(IssueType::exception(), e.to_string())
            })??;

            bulk_ops.push(res);
        }

        Ok(bulk_ops)
    }

    async fn build_bulk_operation<ParameterResolver: SearchParameterResolve>(
        engine: Arc<FPEngine>,
        parameter_resolver: Arc<ParameterResolver>,
        resource: IndexResource,
        search_index_name: &'static str,
    ) -> Result<BulkOperation<HashMap<String, InsertableIndex>>, OperationOutcomeError> {
        match &resource.fhir_method {
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
        }
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
                    map.insert(url.clone(), result_vec);
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

static R4_FHIR_INDEX: &str = "r4_search_index";

#[must_use]
pub const fn get_index_name() -> &'static str {
    R4_FHIR_INDEX
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
    ) -> Result<SuccessfullyIndexedCount, OperationOutcomeError> {
        // Iterator used to evaluate all of the search expressions for indexing.

        let resources_total = resources.len();
        let search_index_name = get_index_name();

        tracing::trace!(
            "Indexing {} resources into index: '{}'",
            resources_total,
            search_index_name
        );

        let tasks = self.spawn_index_tasks(resources, search_index_name);
        let bulk_ops = self.collect_bulk_operations(tasks, resources_total).await?;

        tracing::trace!(
            "Bulk indexing {} resources into index: '{}'",
            bulk_ops.len(),
            search_index_name
        );

        self.send_bulk_operations(search_index_name, bulk_ops).await
    }

    async fn migrate(
        &self,
        _fhir_version: &SupportedFHIRVersions,
    ) -> Result<(), haste_fhir_operation_error::OperationOutcomeError> {
        migration::create_mapping(
            self.parameter_resolver.clone(),
            &self.client,
            get_index_name(),
        )
        .await?;
        Ok(())
    }
}
