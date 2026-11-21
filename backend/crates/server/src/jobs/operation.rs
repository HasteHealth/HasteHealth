//! The `operation` kind of job: an operation invoked with
//! `Prefer: respond-async`, run later by the worker exactly as the request
//! would have run it (the same client, middleware, scopes and access
//! policies, from the submitter's stored claims), with the resource it
//! returns served as the job's result.

use crate::error::ServerError;
use crate::{
    auth_n::middleware::jwt::User, config::ServerConfig, fhir_client::ServerCTX, jobs::status_url,
    services::ServerState,
};
use haste_fhir_client::{
    FHIRClient,
    request::{
        FHIRAcceptedResponse, FHIRInvokeInstanceRequest, FHIRInvokeSystemRequest,
        FHIRInvokeTypeRequest, FHIRRequest, FHIRResponse, InvocationRequest, InvokeResponse,
        Operation,
    },
};
use haste_fhir_model::r4::generated::{
    resources::{Parameters, ResourceType},
    terminology::IssueType,
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_repository::{Repository, async_job::CreateAsyncJob};
use haste_worker::async_jobs::{HandlerFuture, JobContext, JobHandler, JobOutput};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub const KIND: &str = "operation";

/// Re-running an operation is not safe in general (a custom one may write),
/// so an operation job gets one attempt: if its worker dies mid-way, the job
/// fails rather than runs again.
const MAX_ATTEMPTS: i32 = 1;

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
#[serde(rename_all = "lowercase")]
enum Level {
    Instance,
    Type,
    System,
}

/// An invocation as stored in the job's `parameters`: enough to rebuild the
/// `InvocationRequest` in the worker.
#[derive(Serialize, Deserialize, Debug)]
pub struct StoredInvocation {
    level: Level,
    #[serde(skip_serializing_if = "Option::is_none")]
    resource_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<String>,
    operation: String,
    parameters: Parameters,
}

impl From<&InvocationRequest> for StoredInvocation {
    fn from(request: &InvocationRequest) -> Self {
        match request {
            InvocationRequest::Instance(request) => StoredInvocation {
                level: Level::Instance,
                resource_type: Some(request.resource_type.as_ref().to_string()),
                id: Some(request.id.clone()),
                operation: request.operation.name().to_string(),
                parameters: request.parameters.clone(),
            },
            InvocationRequest::Type(request) => StoredInvocation {
                level: Level::Type,
                resource_type: Some(request.resource_type.as_ref().to_string()),
                id: None,
                operation: request.operation.name().to_string(),
                parameters: request.parameters.clone(),
            },
            InvocationRequest::System(request) => StoredInvocation {
                level: Level::System,
                resource_type: None,
                id: None,
                operation: request.operation.name().to_string(),
                parameters: request.parameters.clone(),
            },
        }
    }
}

impl TryFrom<StoredInvocation> for InvocationRequest {
    type Error = OperationOutcomeError;

    fn try_from(stored: StoredInvocation) -> Result<Self, Self::Error> {
        let resource_type = |name: Option<String>| {
            ResourceType::try_from(name.unwrap_or_default()).map_err(|_| {
                OperationOutcomeError::error(
                    IssueType::invalid(),
                    "The stored invocation names an unknown resource type.".to_string(),
                )
            })
        };
        let operation = Operation::new(&stored.operation);

        Ok(match stored.level {
            Level::Instance => InvocationRequest::Instance(FHIRInvokeInstanceRequest {
                operation,
                resource_type: resource_type(stored.resource_type)?,
                id: stored.id.unwrap_or_default(),
                parameters: stored.parameters,
            }),
            Level::Type => InvocationRequest::Type(FHIRInvokeTypeRequest {
                operation,
                resource_type: resource_type(stored.resource_type)?,
                parameters: stored.parameters,
            }),
            Level::System => InvocationRequest::System(FHIRInvokeSystemRequest {
                operation,
                parameters: stored.parameters,
            }),
        })
    }
}

/// The invocation as a FHIR location, for the job's `request`.
fn describe(request: &InvocationRequest) -> String {
    match request {
        InvocationRequest::Instance(request) => format!(
            "{}/{}/${}",
            request.resource_type.as_ref(),
            request.id,
            request.operation.name()
        ),
        InvocationRequest::Type(request) => {
            format!(
                "{}/${}",
                request.resource_type.as_ref(),
                request.operation.name()
            )
        }
        InvocationRequest::System(request) => format!("${}", request.operation.name()),
    }
}

/// Queues `request` as a job for the caller and answers `202 Accepted` with
/// its status URL. Called once the operation is known to exist and the
/// request has passed every check the synchronous form would have.
pub async fn submit<
    Repo: Repository + Send + Sync + 'static,
    Client: FHIRClient<Arc<ServerCTX<Client>>, OperationOutcomeError> + 'static,
>(
    repo: &Repo,
    config: &ServerConfig,
    ctx: &ServerCTX<Client>,
    request: &InvocationRequest,
) -> Result<FHIRResponse, OperationOutcomeError> {
    let live = repo.count_live_jobs(&ctx.tenant, &ctx.project).await?;
    if live >= config.jobs.max_live_per_project {
        return Err(OperationOutcomeError::error(
            IssueType::throttled(),
            format!(
                "Project '{}' already has {live} queued or running job(s); wait for one to \
                 finish before starting another.",
                ctx.project.as_ref()
            ),
        ));
    }

    let parameters =
        serde_json::to_value(StoredInvocation::from(request)).map_err(ServerError::from)?;

    let job = repo
        .create_job(
            &ctx.tenant,
            &ctx.project,
            CreateAsyncJob {
                kind: KIND.to_string(),
                request: describe(request),
                parameters,
                claims: ctx.user.claims.clone(),
                retention: config.jobs.retention(),
                max_attempts: MAX_ATTEMPTS,
            },
        )
        .await?;

    tracing::info!(
        job = %job.id,
        request = %job.request,
        "Queued an operation as an asynchronous job"
    );

    let content_location = status_url(&config.api_uri, &ctx.tenant, &ctx.project, &job.id)?;

    Ok(FHIRResponse::Accepted(FHIRAcceptedResponse {
        content_location: content_location.to_string(),
    }))
}

/// Runs `operation` jobs through the server's FHIR client.
pub struct OperationJobHandler<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
> {
    state: Arc<ServerState<Repo, Search, Terminology>>,
}

impl<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
> OperationJobHandler<Repo, Search, Terminology>
{
    #[must_use]
    pub fn new(state: Arc<ServerState<Repo, Search, Terminology>>) -> Self {
        Self { state }
    }
}

impl<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
> JobHandler for OperationJobHandler<Repo, Search, Terminology>
{
    fn kind(&self) -> &'static str {
        KIND
    }

    fn run<'a>(&'a self, context: &'a JobContext) -> HandlerFuture<'a, JobOutput> {
        Box::pin(async move {
            let job = &context.job;
            let stored: StoredInvocation = serde_json::from_value(job.parameters.0.clone())
                .map_err(|e| {
                    OperationOutcomeError::fatal(
                        IssueType::exception(),
                        format!("The job's stored invocation could not be read: {e}"),
                    )
                })?;
            let request = InvocationRequest::try_from(stored)?;

            // The submitter's verified claims, so the operation runs with
            // their scopes and access policies. No token: it was checked at
            // kick-off, and nothing downstream reads it.
            let claims = job.claims.0.clone();
            let ctx = Arc::new(
                ServerCTX::new(
                    job.tenant.clone(),
                    job.project.clone(),
                    claims.fhir_version.clone(),
                    Arc::new(User {
                        token: None,
                        claims,
                    }),
                    self.state.fhir_client.clone(),
                    self.state.rate_limit.clone(),
                )
                .with_tracing_id(Some(format!("job-{}", job.id))),
            );

            let response = self
                .state
                .fhir_client
                .request(ctx, FHIRRequest::Invocation(request))
                .await?;

            match response {
                FHIRResponse::Invoke(InvokeResponse::Instance(response)) => {
                    Ok(JobOutput::Resource(Box::new(response.resource)))
                }
                FHIRResponse::Invoke(InvokeResponse::Type(response)) => {
                    Ok(JobOutput::Resource(Box::new(response.resource)))
                }
                FHIRResponse::Invoke(InvokeResponse::System(response)) => {
                    Ok(JobOutput::Resource(Box::new(response.resource)))
                }
                _ => Err(OperationOutcomeError::fatal(
                    IssueType::exception(),
                    "The operation did not return an operation response.".to_string(),
                )),
            }
        })
    }
}
