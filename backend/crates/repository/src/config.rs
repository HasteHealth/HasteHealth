//! Repository configuration and construction.
//!
//! Lives here rather than in each binary so the server and the worker connect
//! to the same store described the same way — a second copy of these types is
//! a second thing to keep in step, and the two drifting apart is how a worker
//! ends up indexing into somewhere the server never reads.

use std::sync::Arc;

use haste_fhir_model::r4::generated::terminology::IssueType;
use haste_fhir_operation_error::OperationOutcomeError;
use serde::{Deserialize, Serialize};

use crate::pg::PGConnection;

/// Where the FHIR server stores its resources.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "backend", rename_all = "snake_case")]
pub enum RepoConfig {
    Postgres(PostgresRepoConfig),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PostgresRepoConfig {
    pub database_url: String,
    pub max_connections: u32,
}

impl Default for PostgresRepoConfig {
    fn default() -> Self {
        Self {
            database_url: "postgresql://postgres:postgres@127.0.0.1/haste_health".to_string(),
            max_connections: 20,
        }
    }
}

impl Default for RepoConfig {
    fn default() -> Self {
        RepoConfig::Postgres(PostgresRepoConfig::default())
    }
}

/// Opens a connection pool for `config`.
///
/// # Errors
///
/// Returns an error if the pool cannot open its first connection — an
/// unreachable host, bad credentials, or a malformed `database_url`.
pub async fn create_repo(config: &RepoConfig) -> Result<Arc<PGConnection>, OperationOutcomeError> {
    match config {
        RepoConfig::Postgres(pg_config) => {
            let pool = sqlx::postgres::PgPoolOptions::new()
                .max_connections(pg_config.max_connections)
                .connect(&pg_config.database_url)
                .await
                .map_err(|e| {
                    OperationOutcomeError::fatal(
                        IssueType::exception(),
                        format!("Failed to connect to the repository database: {e}"),
                    )
                })?;

            Ok(Arc::new(PGConnection::pool(pool)))
        }
    }
}
