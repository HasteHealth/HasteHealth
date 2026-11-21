use crate::indexing_lock::IndexLockProvider;
use haste_fhir_operation_error::{OperationOutcomeError, derive::OperationOutcomeError};
use haste_jwt::{ProjectId, TenantId};
use haste_repository::pg::PGConnection;
use haste_repository::types::SearchIndexBackend;
use sqlx::{Acquire, Postgres, QueryBuilder};

#[derive(OperationOutcomeError, Debug)]
pub enum TenantLockIndexError {
    #[fatal(code = "exception", diagnostic = "SQL error occurred {arg0}")]
    SQLError(#[from] sqlx::Error),
    #[fatal(
        code = "exception",
        diagnostic = "Locking must be done in a transaction."
    )]
    InvalidConnection,
}

#[derive(sqlx::FromRow, Debug)]
pub struct ProjectLockIndex {
    #[allow(dead_code)]
    pub tenant: TenantId,
    #[allow(dead_code)]
    pub project: ProjectId,
    pub index_sequence_position: i64,
}

impl IndexLockProvider<(TenantId, ProjectId), ProjectLockIndex> for PGConnection {
    async fn get_available_locks(
        &self,
        backend: SearchIndexBackend,
        project_ids: Vec<&(TenantId, ProjectId)>,
    ) -> Result<Vec<ProjectLockIndex>, OperationOutcomeError> {
        match self {
            PGConnection::Transaction(tx, _, _) => {
                let mut tx = tx.lock().await;
                let conn = (&mut (*tx))
                    .acquire()
                    .await
                    .map_err(TenantLockIndexError::from)?;

                let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
                    "SELECT tenant, project, index_sequence_position FROM search_index_locks WHERE backend = ",
                );
                query_builder.push_bind(backend);
                query_builder.push(" AND (tenant, project) IN ( ");

                let mut separated = query_builder.separated(", ");
                for (tenant_id, project_id) in &project_ids {
                    separated.push_unseparated("(");
                    separated.push_bind_unseparated(tenant_id.as_ref());
                    separated.push_unseparated(", ");
                    separated.push_bind_unseparated(project_id.as_ref());
                    separated.push_unseparated(")");
                }

                query_builder.push(") FOR NO KEY UPDATE SKIP LOCKED");

                let query = query_builder.build_query_as();
                let res = query
                    .fetch_all(conn)
                    .await
                    .map_err(TenantLockIndexError::from)?;

                Ok(res)
            }
            PGConnection::Pool(..) => Err(TenantLockIndexError::InvalidConnection.into()),
        }
    }

    async fn update_lock(
        &self,
        backend: SearchIndexBackend,
        project_id: &(TenantId, ProjectId),
        model: ProjectLockIndex,
    ) -> Result<(), OperationOutcomeError> {
        match self {
            PGConnection::Transaction(tx, _, _) => {
                let mut tx = tx.lock().await;
                let conn = (&mut (*tx))
                    .acquire()
                    .await
                    .map_err(TenantLockIndexError::from)?;
                let (tenant_id, proj_id) = project_id;
                sqlx::query!(
                    "UPDATE search_index_locks SET index_sequence_position = $1 WHERE tenant = $2 AND project = $3 AND backend = $4",
                    model.index_sequence_position,
                    tenant_id.as_ref(),
                    proj_id.as_ref(),
                    backend as SearchIndexBackend,
                )
                .execute(conn)
                .await
                .map_err(TenantLockIndexError::from)?;

                Ok(())
            }
            PGConnection::Pool(..) => Err(TenantLockIndexError::InvalidConnection.into()),
        }
    }
}
