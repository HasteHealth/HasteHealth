use crate::fhir_client::{
    ClientState, ServerCTX,
    middleware::{Backend, Middleware, Next, Output},
};
use haste_fhir_client::request::FHIRRequest;
use haste_fhir_model::r4::generated::terminology::IssueType;
use haste_fhir_operation_error::OperationOutcomeError;

use haste_repository::fhir::FHIRRepository;
use std::sync::Arc;
use tracing::info;

/// Runs each write in a transaction of its own, committed when the rest of
/// the pipeline succeeds and rolled back when it fails. Reads run as they
/// are, and so does everything when the caller already holds a transaction:
/// that one commits at the caller's site.
pub struct Transaction;

impl<B: Backend> Middleware<B> for Transaction {
    fn call<'a>(
        &'a self,
        state: &'a ClientState<B>,
        ctx: Arc<ServerCTX<B>>,
        request: FHIRRequest,
        next: Next<'a, B>,
    ) -> Output<'a> {
        Box::pin(async move {
            if state.repo.in_transaction() {
                return next.run(state, ctx, request).await;
            }
            match request {
                FHIRRequest::Create(_)
                | FHIRRequest::Delete(_)
                | FHIRRequest::Update(_)
                | FHIRRequest::Patch(_) => {}
                FHIRRequest::Read(_)
                | FHIRRequest::VersionRead(_)
                | FHIRRequest::Search(_)
                | FHIRRequest::History(_) => return next.run(state, ctx, request).await,
                _ => {
                    return Err(OperationOutcomeError::fatal(
                        IssueType::not_supported(),
                        "Request type not supported for transaction middleware.".to_string(),
                    ));
                }
            }

            let repo = Arc::new(state.repo.transaction(true).await?);
            let result = {
                let transaction_state = state.with_repo(repo.clone());
                next.run(&transaction_state, ctx, request).await
            };

            let repo = Arc::try_unwrap(repo).map_err(|_| {
                OperationOutcomeError::fatal(
                    IssueType::exception(),
                    "Failed to unwrap transaction client".to_string(),
                )
            })?;
            match result {
                Ok(response) => {
                    repo.commit().await?;
                    Ok(response)
                }
                Err(error) => {
                    info!("Rolling back transaction due to error");
                    repo.rollback().await?;
                    Err(error)
                }
            }
        })
    }
}
