use std::sync::Arc;

use elasticsearch::{
    Elasticsearch,
    auth::Credentials,
    cert::CertificateValidation,
    http::{
        Url,
        transport::{SingleNodeConnectionPool, TransportBuilder},
    },
};
use oxidized_fhirpath::FPEngine;

use crate::search::SearchEngine;

struct ElasticSearchEngine {
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
            fp_engine: fp_engine,
            client: elasticsearch_client,
        }
    }
}

impl SearchEngine for ElasticSearchEngine {
    fn search(
        &self,
        tenant: crate::repository::TenantId,
        project: crate::repository::ProjectId,
        search_request: super::SearchRequest,
    ) -> Result<Vec<String>, oxidized_fhir_operation_error::OperationOutcomeError> {
        todo!()
    }

    fn index(
        &self,
        tenant: crate::repository::TenantId,
        project: crate::repository::ProjectId,
        resource: Vec<oxidized_fhir_model::r4::types::Resource>,
    ) -> Result<(), oxidized_fhir_operation_error::OperationOutcomeError> {
        todo!()
    }

    fn remove_index(
        &self,
        tenant: crate::repository::TenantId,
        project: crate::repository::ProjectId,
        remove_indices: Vec<super::RemoveIndex>,
    ) -> Result<(), oxidized_fhir_operation_error::OperationOutcomeError> {
        todo!()
    }
}
