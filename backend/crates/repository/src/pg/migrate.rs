use oxidized_fhir_model::r4::generated::terminology::IssueType;
use oxidized_fhir_operation_error::OperationOutcomeError;
use sqlx;

use crate::admin::Migrate;

impl Migrate for super::PGConnection {
    async fn migrate(&self) -> Result<(), OperationOutcomeError> {
        match self {
            super::PGConnection::Pool(pool, _) => {
                sqlx::migrate!("./pg-migrations")
                    .run(pool)
                    .await
                    .map_err(|e| {
                        OperationOutcomeError::fatal(
                            IssueType::Exception(None),
                            format!("Failed to migrate repository schema: {}", e),
                        )
                    })?;
                Ok(())
            }
            super::PGConnection::Transaction(_, _) => {
                return Err(OperationOutcomeError::fatal(
                    IssueType::Exception(None),
                    "Cannot run migrations in a transaction.".to_string(),
                ));
            }
        }
    }
}
