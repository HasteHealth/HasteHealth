use crate::{
    IndexResource, SearchEngine, SearchEntry, SearchOptions, SearchParameterResolve, SearchReturn,
    SuccessfullyIndexedCount,
    indexing_conversion::{self, InsertableIndex},
};
use elasticsearch::{
    BulkOperation, BulkParts, Elasticsearch, SearchParts,
    auth::Credentials,
    cert::CertificateValidation,
    http::{
        Url,
        transport::{BuildError, SingleNodeConnectionPool, TransportBuilder},
    },
};
use haste_fhir_client::request::SearchRequest;
use haste_fhir_model::r4::generated::{
    resources::{Resource, ResourceType, SearchParameter},
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

#[derive(Deserialize, Debug)]
struct SearchEntryPrivate {
    pub id: Vec<ResourceId>,
    pub resource_type: Vec<ResourceType>,
    pub version_id: Vec<VersionId>,
    pub project: Vec<ProjectId>,
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

impl<SearchParameterResolver: SearchParameterResolve + 'static>
    ElasticSearchEngine<SearchParameterResolver>
{
    pub fn new(
        parameter_resolver: Arc<SearchParameterResolver>,
        fp_engine: Arc<FPEngine>,
        url: &str,
        username: String,
        password: String,
    ) -> Result<Self, SearchConfigError> {
        let url =
            Url::parse(url).map_err(|_e| SearchConfigError::UrlParseError(url.to_string()))?;
        let conn_pool = SingleNodeConnectionPool::new(url);
        let transport = TransportBuilder::new(conn_pool)
            .cert_validation(CertificateValidation::None)
            .auth(Credentials::Basic(username, password))
            .build()?;

        let elasticsearch_client = Elasticsearch::new(transport);
        Ok(ElasticSearchEngine {
            parameter_resolver,
            fp_engine,
            client: Arc::new(elasticsearch_client),
        })
    }

    pub async fn is_connected(&self) -> Result<(), SearchError> {
        let res = self.client.ping().send().await.map_err(SearchError::from)?;

        if res.status_code().is_success() {
            Ok(())
        } else {
            Err(SearchError::NotConnected)
        }
    }
}

async fn resource_to_elastic_index(
    fp_engine: Arc<FPEngine>,
    parameters: &Vec<Arc<SearchParameter>>,
    resource: &Resource,
) -> Result<HashMap<String, InsertableIndex>, OperationOutcomeError> {
    let mut map = HashMap::new();
    for param in parameters.iter() {
        if let Some(expression) = param.expression.as_ref().and_then(|e| e.value.as_ref())
            && let Some(url) = param.url.value.as_ref()
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

                return Err(SearchError::from(err).into());
            }

            let result_vec = indexing_conversion::to_insertable_index(
                param,
                result?.iter().collect::<Vec<_>>(),
            )?;

            map.insert(url.clone(), result_vec);
        }
    }

    Ok(map)
}

static R4_FHIR_INDEX: &str = "r4_search_index";

pub fn get_index_name(
    fhir_version: &SupportedFHIRVersions,
) -> Result<&'static str, SearchConfigError> {
    match fhir_version {
        SupportedFHIRVersions::R4 => Ok(R4_FHIR_INDEX),
        // _ => Err(SearchConfigError::UnsupportedIndex(fhir_version.clone())),
    }
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

impl<SearchParameterResolver: SearchParameterResolve + 'static> SearchEngine
    for ElasticSearchEngine<SearchParameterResolver>
{
    async fn search(
        &self,
        fhir_version: &SupportedFHIRVersions,
        tenant: &TenantId,
        projects: &[&ProjectId],
        search_request: &SearchRequest,
        options: Option<SearchOptions>,
    ) -> Result<SearchReturn, haste_fhir_operation_error::OperationOutcomeError> {
        let query = search::build_elastic_search_query(
            self.parameter_resolver.clone(),
            tenant,
            projects,
            &search_request,
            &options,
        )
        .await?;

        let search_response = self
            .client
            .search(SearchParts::Index(&[get_index_name(&fhir_version)?]))
            .body(query)
            .send()
            .await
            .map_err(SearchError::from)?;

        if !search_response.status_code().is_success() {
            return Err(SearchError::ElasticSearchResponseError(
                search_response.status_code().as_u16(),
            )
            .into());
        }

        let search_results = search_response
            .json::<ElasticSearchResponse>()
            .await
            .map_err(SearchError::from)?;

        Ok(SearchReturn {
            total: search_results.hits.total.as_ref().map(|t| t.value),
            entries: search_results
                .hits
                .hits
                .into_iter()
                .map(|mut hit| SearchEntry {
                    id: hit.fields.id.pop().unwrap(),
                    resource_type: hit.fields.resource_type.pop().unwrap(),
                    version_id: hit.fields.version_id.pop().unwrap(),
                    project: hit.fields.project.pop().unwrap(),
                })
                .collect(),
        })
    }

    fn index(
        &self,
        fhir_version: SupportedFHIRVersions,
        resources: Vec<IndexResource>,
    ) -> impl Future<Output = Result<SuccessfullyIndexedCount, OperationOutcomeError>> + Send + Sync
    {
        async move {
            // Iterator used to evaluate all of the search expressions for indexing.

            let mut tasks = Vec::with_capacity(resources.len());
            let resources_total = resources.len();
            let search_index_name = get_index_name(&fhir_version)?;

            for r in resources.into_iter().filter(|r| match r.fhir_method {
                FHIRMethod::Create | FHIRMethod::Update | FHIRMethod::Delete => true,
                _ => false,
            }) {
                let engine = self.fp_engine.clone();
                let parameter_resolver = self.parameter_resolver.clone();
                tasks.push(tokio::spawn(async move {
                    match &r.fhir_method {
                        FHIRMethod::Create | FHIRMethod::Update => {
                            // Id is not sufficient because different Resourcetypes may have the same id.
                            let index_id =
                                unique_index_id(&r.tenant, &r.project, &r.resource_type, &r.id);
                            let params =
                                parameter_resolver.by_resource_type(&r.resource_type).await;

                            let mut elastic_index =
                                resource_to_elastic_index(engine, &params, &r.resource).await?;

                            elastic_index.insert(
                                "resource_type".to_string(),
                                InsertableIndex::Meta(r.resource_type.as_ref().to_string()),
                            );

                            elastic_index.insert(
                                "id".to_string(),
                                InsertableIndex::Meta(r.id.as_ref().to_string()),
                            );

                            elastic_index.insert(
                                "version_id".to_string(),
                                InsertableIndex::Meta(r.version_id.as_ref().to_string()),
                            );
                            elastic_index.insert(
                                "project".to_string(),
                                InsertableIndex::Meta(r.project.as_ref().to_string()),
                            );
                            elastic_index.insert(
                                "tenant".to_string(),
                                InsertableIndex::Meta(r.tenant.as_ref().to_string()),
                            );
                            Ok(BulkOperation::index(elastic_index)
                                .id(index_id)
                                .index(search_index_name)
                                .into())
                        }
                        FHIRMethod::Delete => Ok(BulkOperation::delete(unique_index_id(
                            &r.tenant,
                            &r.project,
                            &r.resource_type,
                            &r.id,
                        ))
                        .index(search_index_name)
                        .into()),
                        method => Err(SearchError::UnsupportedFHIRMethod((*method).clone()))
                            .map_err(OperationOutcomeError::from),
                    }
                }));
            }

            let client = self.client.clone();

            let mut bulk_ops: Vec<BulkOperation<HashMap<String, InsertableIndex>>> =
                Vec::with_capacity(resources_total);

            for task in tasks {
                let res = task.await.map_err(|e| {
                    OperationOutcomeError::fatal(IssueType::Exception(None), e.to_string())
                })??;
                bulk_ops.push(res);
            }

            if !bulk_ops.is_empty() {
                let res = client
                    .bulk(BulkParts::Index(search_index_name))
                    .body(bulk_ops)
                    .send()
                    .await
                    .map_err(SearchError::from)?;

                let response_body = res.json::<serde_json::Value>().await.map_err(|_e| {
                    OperationOutcomeError::fatal(
                        IssueType::Exception(None),
                        "Failed to parse response body.".to_string(),
                    )
                })?;

                if response_body["errors"].as_bool().unwrap() == true {
                    tracing::error!("Failed to index resources. Response: '{:?}'", response_body);
                    return Err(SearchError::Fatal(500).into());
                }
                Ok(SuccessfullyIndexedCount(
                    response_body["items"].as_array().unwrap().len(),
                ))
            } else {
                Ok(SuccessfullyIndexedCount(0))
            }
        }
    }

    async fn migrate(
        &self,
        _fhir_version: &SupportedFHIRVersions,
    ) -> Result<(), haste_fhir_operation_error::OperationOutcomeError> {
        migration::create_mapping(
            self.parameter_resolver.clone(),
            &self.client,
            get_index_name(_fhir_version)?,
        )
        .await?;
        Ok(())
    }
}
