use crate::SearchEngine;
use elasticsearch::{
    Elasticsearch,
    auth::Credentials,
    cert::CertificateValidation,
    http::{
        Url,
        transport::{SingleNodeConnectionPool, TransportBuilder},
    },
};
use oxidized_fhir_repository::{ProjectId, SupportedFHIRVersions, TenantId};
use oxidized_fhirpath::FPEngine;
use std::sync::Arc;

pub mod migration;

pub struct ElasticSearchEngine {
    _fp_engine: Arc<FPEngine>,
    _client: Elasticsearch,
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
            _fp_engine: fp_engine,
            _client: elasticsearch_client,
        }
    }
}

impl SearchEngine for ElasticSearchEngine {
    async fn search(
        &self,
        _tenant: TenantId,
        _project: ProjectId,
        _search_request: super::SearchRequest,
    ) -> Result<Vec<String>, oxidized_fhir_operation_error::OperationOutcomeError> {
        todo!()
    }

    async fn index(
        &self,
        _tenant: TenantId,
        _project: ProjectId,
        _resource: Vec<oxidized_fhir_model::r4::types::Resource>,
    ) -> Result<(), oxidized_fhir_operation_error::OperationOutcomeError> {
        todo!()
    }

    async fn remove_index(
        &self,
        _tenant: TenantId,
        _project: ProjectId,
        _remove_indices: Vec<super::RemoveIndex>,
    ) -> Result<(), oxidized_fhir_operation_error::OperationOutcomeError> {
        todo!()
    }

    async fn migrate(
        &self,
        _fhir_version: SupportedFHIRVersions,
        index_name: &str,
    ) -> Result<(), oxidized_fhir_operation_error::OperationOutcomeError> {
        migration::create_mapping(&self._client, index_name).await?;
        // Here you would typically call the migration function to set up the index
        // For example:
        // migration::create_mapping(&self._client).await?;
        Ok(())
    }
}
