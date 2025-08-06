use crate::{
    IndexResource, SearchEngine,
    indexing_conversion::{self, InsertableIndex},
};
use elasticsearch::{
    BulkOperation, BulkParts, Elasticsearch,
    auth::Credentials,
    cert::CertificateValidation,
    http::{
        Url,
        transport::{SingleNodeConnectionPool, TransportBuilder},
    },
};
use oxidized_fhir_model::r4::types::{Resource, SearchParameter};
use oxidized_fhir_operation_error::{OperationOutcomeError, derive::OperationOutcomeError};
use oxidized_fhir_repository::{FHIRMethod, ProjectId, SupportedFHIRVersions, TenantId};
use oxidized_fhirpath::FPEngine;
use rayon::prelude::*;
use std::{collections::HashMap, sync::Arc};

pub mod migration;

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
}

pub struct ElasticSearchEngine {
    fp_engine: Arc<FPEngine>,
    client: Elasticsearch,
}

impl ElasticSearchEngine {
    pub fn new(fp_engine: Arc<FPEngine>, url: &str, username: String, password: String) -> Self {
        let url = Url::parse(url).unwrap();
        let conn_pool = SingleNodeConnectionPool::new(url);
        let transport = TransportBuilder::new(conn_pool)
            .cert_validation(CertificateValidation::None)
            .auth(Credentials::Basic(username, password))
            .build()
            .unwrap();
        let elasticsearch_client = Elasticsearch::new(transport);
        ElasticSearchEngine {
            fp_engine,
            client: elasticsearch_client,
        }
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

pub fn get_index_name(fhir_version: SupportedFHIRVersions) -> &'static str {
    match fhir_version {
        SupportedFHIRVersions::R4 => R4_FHIR_INDEX,
        _ => panic!("Unsupported FHIR version for index name"),
    }
}

impl SearchEngine for ElasticSearchEngine {
    async fn search(
        &self,
        _fhir_version: &SupportedFHIRVersions,
        _tenant: TenantId,
        _project: ProjectId,
        _search_request: super::SearchRequest,
    ) -> Result<Vec<String>, oxidized_fhir_operation_error::OperationOutcomeError> {
        todo!()
    }

    async fn index<'a>(
        &self,
        fhir_version: &SupportedFHIRVersions,
        tenant_id: TenantId,
        _project: ProjectId,
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
                        resource_to_elastic_index(self.fp_engine.clone(), &params, &r.resource.0)?;

                    elastic_index.insert(
                        "resource_type".to_string(),
                        InsertableIndex::String(vec![r.resource_type.as_str().to_string()]),
                    );
                    elastic_index.insert(
                        "version_id".to_string(),
                        InsertableIndex::String(vec![r.version_id.as_ref().to_string()]),
                    );
                    elastic_index.insert(
                        "project".to_string(),
                        InsertableIndex::String(vec![r.project.as_ref().to_string()]),
                    );
                    elastic_index.insert(
                        "tenant".to_string(),
                        InsertableIndex::String(vec![r.tenant.as_ref().to_string()]),
                    );

                    Ok(BulkOperation::index(elastic_index)
                        .id(&r.id)
                        .index(R4_FHIR_INDEX)
                        .into())
                }
                FHIRMethod::Delete => Ok(BulkOperation::delete(&r.id).index(R4_FHIR_INDEX).into()),
                method => Err(SearchError::UnsupportedFHIRMethod(method.clone()).into()),
            })
            .collect::<Result<Vec<_>, OperationOutcomeError>>()?;

        if !bulk_ops.is_empty() {
            let res = self
                .client
                .bulk(BulkParts::Index(R4_FHIR_INDEX))
                .body(bulk_ops)
                .send()
                .await?;

            if !res.status_code().is_success() {
                tracing::error!(
                    "Failed to index resources for tenant: '{}'. Response: '{:?}', body: '{}'",
                    tenant_id,
                    res.status_code(),
                    res.text().await.unwrap()
                );
                return Err(SearchError::Fatal(res.status_code().as_u16()).into());
            }
        }

        Ok(())
    }

    async fn migrate(
        &self,
        _fhir_version: &SupportedFHIRVersions,
    ) -> Result<(), oxidized_fhir_operation_error::OperationOutcomeError> {
        migration::create_mapping(&self.client, R4_FHIR_INDEX).await?;
        Ok(())
    }
}
