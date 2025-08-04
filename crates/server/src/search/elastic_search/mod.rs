use elasticsearch::{
    Elasticsearch,
    auth::Credentials,
    cert::CertificateValidation,
    http::{
        Url,
        transport::{SingleNodeConnectionPool, TransportBuilder},
    },
};

use crate::search::SearchEngine;

struct ElasticSearchEngine(Elasticsearch);

impl ElasticSearchEngine {
    pub fn new(url: &str, username: String, password: String) -> Self {
        let url = Url::parse(url).unwrap();
        let conn_pool = SingleNodeConnectionPool::new(url);
        let transport = TransportBuilder::new(conn_pool)
            .cert_validation(CertificateValidation::None)
            .auth(Credentials::Basic(username, password))
            .build()
            .unwrap();
        let elasticsearch_client = Elasticsearch::new(transport);
        ElasticSearchEngine(elasticsearch_client)
    }
}

impl SearchEngine for ElasticSearchEngine {
    fn search(
        tenant: crate::repository::TenantId,
        project: crate::repository::ProjectId,
        search_request: super::SearchRequest,
    ) -> Result<Vec<String>, oxidized_fhir_operation_error::OperationOutcomeError> {
        todo!()
    }

    fn index(
        tenant: crate::repository::TenantId,
        project: crate::repository::ProjectId,
        resource: Vec<oxidized_fhir_model::r4::types::Resource>,
    ) -> Result<(), oxidized_fhir_operation_error::OperationOutcomeError> {
        todo!()
    }

    fn remove_index(
        tenant: crate::repository::TenantId,
        project: crate::repository::ProjectId,
        remove_indices: Vec<super::RemoveIndex>,
    ) -> Result<(), oxidized_fhir_operation_error::OperationOutcomeError> {
        todo!()
    }
}
