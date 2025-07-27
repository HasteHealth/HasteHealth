use std::{pin::Pin, sync::Arc};

use crate::repository::{FHIRMethod, FHIRRepository, InsertResourceRow};
use fhir_client::{
    middleware::{Context, Middleware, Next},
    request::{FHIRRequest, FHIRResponse, Operation},
};
use fhir_model::r4::{sqlx::FHIRJsonRef, types::OperationOutcome};
use fhir_operation_error::OperationOutcomeError;

fn middleware_1<Repository: FHIRRepository + Send + Sync, CTX: Send + Sync>(
    state: Arc<(Repository)>,
    context: Context<CTX, FHIRRequest, FHIRResponse>,
    next: Option<
        Arc<Next<Arc<(Repository)>, CTX, FHIRRequest, FHIRResponse, OperationOutcomeError>>,
    >,
) -> Pin<
    Box<
        dyn Future<Output = Result<Context<CTX, FHIRRequest, FHIRResponse>, OperationOutcomeError>>
            + Send,
    >,
> {
    Box::pin(async move {
        if let FHIRRequest::Create(create_request) = &context.request {
            let response = state
                .insert(&InsertResourceRow {
                    tenant: path.tenant.to_string(),
                    project: path.project.to_string(),
                    author_id: "fake_author_id".to_string(),
                    fhir_version: path.fhir_version,
                    resource: FHIRJsonRef(&create_request.resource),
                    deleted: false,
                    request_method: "POST".to_string(),
                    author_type: "member".to_string(),
                    fhir_method: FHIRMethod::try_from(&context.request).unwrap(),
                })
                .await?;
            Ok((
                axum::http::StatusCode::CREATED,
                fhir_serialization_json::to_string(&response).unwrap(),
            )
                .into_response())
        } else {
            Ok((axum::http::StatusCode::OK, "Request successful".to_string()).into_response())
        }
    })
}

struct FHIRServerClient<Repository: FHIRRepository + Send + Sync, CTX: Send + Sync> {
    repository: Repository,
    middleware:
        Middleware<Arc<(Repository)>, CTX, FHIRRequest, FHIRResponse, OperationOutcomeError>,
}

impl<Repository: FHIRRepository + Send + Sync + 'static, CTX: Send + Sync + 'static>
    FHIRServerClient<Repository, CTX>
{
    pub fn new(repository: Repository, ctx: CTX) -> Self {
        let middleware = Middleware::new(vec![]);
        FHIRServerClient {
            repository,
            middleware,
        }
    }
}
