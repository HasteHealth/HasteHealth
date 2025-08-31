use crate::{
    IndexResource, SearchEngine, SearchEntry, SearchOptions, SearchReturn,
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
use oxidized_fhir_model::r4::types::{Resource, ResourceType, SearchParameter};
use oxidized_fhir_operation_error::{OperationOutcomeError, derive::OperationOutcomeError};
use oxidized_fhirpath::FPEngine;
use oxidized_repository::types::{
    FHIRMethod, ProjectId, ResourceId, SupportedFHIRVersions, TenantId, VersionId,
};
use rayon::prelude::*;
use serde::Deserialize;
use std::{collections::HashMap, sync::Arc};

mod migration;
mod search;

#[derive(Deserialize, Debug)]
struct SearchEntryPrivate {
    pub id: Vec<ResourceId>,
    pub resource_type: Vec<ResourceType>,
    pub version_id: Vec<VersionId>,
}

#[derive(OperationOutcomeError, Debug)]
pub enum SearchError {
    #[fatal(
        code = "exception",
        diagnostic = "Failed to evaluate fhirpath expression."
    )]
    FHIRPathError(#[from] oxidized_fhirpath::FHIRPathError),
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
pub struct ElasticSearchEngine {
    fp_engine: Arc<FPEngine>,
    client: Elasticsearch,
}

impl ElasticSearchEngine {
    pub fn new(
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
            fp_engine,
            client: elasticsearch_client,
        })
    }
}

fn resource_to_elastic_index(
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
                .map_err(SearchError::from)?;

            let result_vec =
                indexing_conversion::to_insertable_index(param, result.iter().collect::<Vec<_>>())?;

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

impl SearchEngine for ElasticSearchEngine {
    async fn search<'a>(
        &self,
        fhir_version: &SupportedFHIRVersions,
        tenant: &TenantId,
        project: &ProjectId,
        search_request: super::SearchRequest<'a>,
        _options: Option<SearchOptions>,
    ) -> Result<SearchReturn, oxidized_fhir_operation_error::OperationOutcomeError> {
        let query = search::build_elastic_search_query(tenant, project, &search_request)?;

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
                })
                .collect(),
        })
    }

    async fn index<'a>(
        &self,
        _fhir_version: &SupportedFHIRVersions,
        tenant: &TenantId,
        resources: Vec<IndexResource<'a>>,
    ) -> Result<(), oxidized_fhir_operation_error::OperationOutcomeError> {
        // Iterator used to evaluate all of the search expressions for indexing.

        let bulk_ops: Vec<BulkOperation<HashMap<String, InsertableIndex>>> = resources
            .par_iter()
            .filter(|r| match r.fhir_method {
                FHIRMethod::Create | FHIRMethod::Update | FHIRMethod::Delete => true,
                _ => false,
            })
            .map(|r| match &r.fhir_method {
                FHIRMethod::Create | FHIRMethod::Update => {
                    let params =
                        oxidized_artifacts::search_parameters::get_search_parameters_for_resource(
                            &r.resource_type,
                        );

                    let mut elastic_index =
                        resource_to_elastic_index(self.fp_engine.clone(), &params, &r.resource)?;

                    elastic_index.insert(
                        "resource_type".to_string(),
                        InsertableIndex::Meta(r.resource_type.as_str().to_string()),
                    );

                    elastic_index.insert(
                        "id".to_string(),
                        InsertableIndex::Meta(r.id.as_ref().to_string()),
                    );

                    elastic_index.insert(
                        "version_id".to_string(),
                        InsertableIndex::Meta(r.version_id.to_string()),
                    );
                    elastic_index.insert(
                        "project".to_string(),
                        InsertableIndex::Meta(r.project.as_ref().to_string()),
                    );
                    elastic_index.insert(
                        "tenant".to_string(),
                        InsertableIndex::Meta(tenant.as_ref().to_string()),
                    );

                    Ok(BulkOperation::index(elastic_index)
                        .id(r.id.as_ref())
                        .index(get_index_name(_fhir_version)?)
                        .into())
                }
                FHIRMethod::Delete => Ok(BulkOperation::delete(r.id.as_ref())
                    .index(get_index_name(_fhir_version)?)
                    .into()),
                method => Err(SearchError::UnsupportedFHIRMethod((*method).clone()).into()),
            })
            .collect::<Result<Vec<_>, OperationOutcomeError>>()?;

        if !bulk_ops.is_empty() {
            let res = self
                .client
                .bulk(BulkParts::Index(get_index_name(_fhir_version)?))
                .body(bulk_ops)
                .send()
                .await
                .map_err(SearchError::from)?;

            if !res.status_code().is_success() {
                let status_code = res.status_code().as_u16();
                tracing::error!(
                    "Failed to index resources for tenant: '{}'. Response: '{:?}', body: '{}'",
                    tenant.as_ref(),
                    status_code,
                    res.text().await.unwrap()
                );
                return Err(SearchError::Fatal(status_code).into());
            }
        }

        Ok(())
    }

    async fn migrate(
        &self,
        _fhir_version: &SupportedFHIRVersions,
    ) -> Result<(), oxidized_fhir_operation_error::OperationOutcomeError> {
        migration::create_mapping(&self.client, get_index_name(_fhir_version)?).await?;
        Ok(())
    }
}
