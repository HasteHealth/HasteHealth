use haste_fhir_client::request::SearchRequest;
use haste_fhir_operation_error::OperationOutcomeError;
use haste_jwt::{ProjectId, TenantId};
use sqlx::{Pool, Postgres};

use super::{R4_SCHEMA_REGISTRY, search};
use crate::{
    SearchOptions, SearchReturn,
    memory::R4_SEARCH_PARAMETERS_INDEX,
    project_parameters::{IndexSearch, ProjectParameterResolver},
};

/// Resolves the base parameters plus each project's own, found in the
/// Postgres search index.
pub type PgSearchParameterResolver<Repo> = ProjectParameterResolver<PgIndexSearch, Repo>;

/// The Postgres search index, searched with the base parameters.
#[derive(Clone)]
pub struct PgIndexSearch {
    pool: Pool<Postgres>,
}

impl PgIndexSearch {
    #[must_use]
    pub fn new(pool: Pool<Postgres>) -> Self {
        PgIndexSearch { pool }
    }
}

impl IndexSearch for PgIndexSearch {
    async fn search(
        &self,
        tenant: &TenantId,
        project: &ProjectId,
        request: &SearchRequest,
        options: &SearchOptions,
    ) -> Result<SearchReturn, OperationOutcomeError> {
        search::execute_search(
            &self.pool,
            R4_SEARCH_PARAMETERS_INDEX.clone(),
            &R4_SCHEMA_REGISTRY,
            tenant,
            project,
            request,
            Some(options),
        )
        .await
    }
}
