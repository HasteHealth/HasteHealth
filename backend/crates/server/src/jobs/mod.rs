//! The FHIR Asynchronous Request Pattern
//! (<https://hl7.org/fhir/R4/async.html>, and the Bulk Data IG's version of
//! it): a request answered with `202 Accepted` and a `Content-Location` to
//! poll, where the client reads `202` with `X-Progress` and `Retry-After`
//! while the work runs, then the result, or an error with an
//! `OperationOutcome`, and can `DELETE` the location to cancel.
//!
//! The work itself is an `async_jobs` row run by `haste-health worker`
//! (see `haste_worker::async_jobs`). This module is the HTTP side: the
//! status URL and the status and cancel endpoints.

use crate::{auth_n::middleware::jwt::User, route_path::api_v1_path, services::ServerState};
use axum::{
    Extension,
    extract::{Path, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};
use haste_fhir_model::r4::generated::{resources::OperationOutcome, terminology::IssueType};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_jwt::{ProjectId, TenantId, UserRole};
use haste_repository::{
    Repository,
    async_job::{AsyncJob, AsyncJobStatus},
};
use serde::Deserialize;
use sqlx::types::time::OffsetDateTime;
use std::sync::Arc;
use tower_sessions::cookie::time::{UtcOffset, format_description};
use url::Url;

/// The absolute URL the client polls for a job: sibling of the project's
/// FHIR root, so the same token authorizes it, but outside the FHIR routes
/// since neither its response codes nor its bodies are FHIR interactions.
pub fn status_url(
    api_uri: &str,
    tenant: &TenantId,
    project: &ProjectId,
    job_id: &str,
) -> Result<Url, OperationOutcomeError> {
    let invalid = |e: url::ParseError| {
        tracing::error!("Failed to derive the job status URL: {:?}", e);
        OperationOutcomeError::error(
            IssueType::invalid(),
            "Invalid API URL configured".to_string(),
        )
    };

    let mut path = api_v1_path(tenant, project);
    path.push("jobs");
    path.push(job_id);

    Url::parse(api_uri)
        .map_err(invalid)?
        .join(path.to_str().unwrap_or_default())
        .map_err(invalid)
}

#[derive(Deserialize)]
pub struct JobPath {
    tenant: TenantId,
    project: ProjectId,
    id: String,
}

/// A job is its submitter's: only they, or a tenant owner or admin, may see
/// or cancel it. Everyone else in the project is told it does not exist, so
/// that job ids leak nothing.
fn owned_by(job: &AsyncJob, user: &User) -> bool {
    job.submitted_by == user.claims.sub.as_ref()
        || matches!(user.claims.user_role, UserRole::Owner | UserRole::Admin)
}

fn not_found(id: &str) -> OperationOutcomeError {
    OperationOutcomeError::error(IssueType::not_found(), format!("Job '{id}' not found."))
}

/// Reads a job of the request's project that the caller may see.
async fn load_job<Repo: Repository + Send + Sync + 'static>(
    repo: &Repo,
    path: &JobPath,
    user: &User,
) -> Result<AsyncJob, OperationOutcomeError> {
    let job = repo
        .read_job(&path.tenant, &path.project, &path.id)
        .await?
        .filter(|job| owned_by(job, user))
        .ok_or_else(|| not_found(&path.id))?;

    Ok(job)
}

/// An HTTP date (IMF-fixdate), as `Expires` and friends want it.
fn http_date(at: OffsetDateTime) -> Option<HeaderValue> {
    let format = format_description::parse_borrowed::<2>(
        "[weekday repr:short], [day] [month repr:short] [year] [hour]:[minute]:[second] GMT",
    )
    .ok()?;

    at.to_offset(UtcOffset::UTC)
        .format(&format)
        .ok()?
        .parse()
        .ok()
}

/// `GET` of the status URL.
///
/// - queued or running: `202 Accepted`, `X-Progress`, `Retry-After`, no body;
/// - completed: `200 OK` and the result, in the content type its handler
///   chose, with `Expires` saying how long it stays available;
/// - failed: the `OperationOutcome` the handler produced, with the status
///   code it implies;
/// - cancelled, or unknown: `404 Not Found`.
pub async fn status_handler<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
>(
    Extension(user): Extension<Arc<User>>,
    Path(path): Path<JobPath>,
    State(state): State<Arc<ServerState<Repo, Search, Terminology>>>,
) -> Result<Response, OperationOutcomeError> {
    let job = load_job(state.repo.as_ref(), &path, &user).await?;

    let mut headers = HeaderMap::new();
    headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));

    match job.status {
        AsyncJobStatus::Accepted | AsyncJobStatus::InProgress => {
            let progress = job.progress.as_deref().unwrap_or(job.status.as_str());
            if let Ok(progress) = HeaderValue::from_str(progress) {
                headers.insert("x-progress", progress);
            }
            headers.insert(
                header::RETRY_AFTER,
                HeaderValue::from(state.config.jobs.retry_after_seconds),
            );

            Ok((StatusCode::ACCEPTED, headers).into_response())
        }
        AsyncJobStatus::Completed => {
            let content_type = job
                .output_content_type
                .as_deref()
                .and_then(|content_type| HeaderValue::from_str(content_type).ok())
                .unwrap_or_else(|| HeaderValue::from_static("application/json"));
            headers.insert(header::CONTENT_TYPE, content_type);
            if let Some(expires) = http_date(job.expires_at) {
                headers.insert(header::EXPIRES, expires);
            }

            let body = job
                .output
                .map(|output| output.0)
                .unwrap_or(serde_json::Value::Null);
            let body = serde_json::to_string(&body).map_err(|e| {
                OperationOutcomeError::fatal(
                    IssueType::exception(),
                    format!("Failed to serialize the job's output: {e}"),
                )
            })?;

            Ok((StatusCode::OK, headers, body).into_response())
        }
        AsyncJobStatus::Failed => {
            let outcome = job
                .error
                .and_then(|error| serde_json::from_value::<OperationOutcome>(error.0).ok())
                .ok_or_else(|| {
                    OperationOutcomeError::fatal(
                        IssueType::exception(),
                        "The job failed.".to_string(),
                    )
                })?;

            Err(OperationOutcomeError::new(None, outcome))
        }
        AsyncJobStatus::Cancelled => Err(not_found(&path.id)),
    }
}

/// `DELETE` of the status URL: cancels the job, or, once it has finished,
/// gives up its result. `202 Accepted` either way; `404` if there is no such
/// job or it was already cancelled.
pub async fn cancel_handler<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
>(
    Extension(user): Extension<Arc<User>>,
    Path(path): Path<JobPath>,
    State(state): State<Arc<ServerState<Repo, Search, Terminology>>>,
) -> Result<Response, OperationOutcomeError> {
    load_job(state.repo.as_ref(), &path, &user).await?;

    state
        .repo
        .cancel_job(&path.tenant, &path.project, &path.id)
        .await?
        .ok_or_else(|| not_found(&path.id))?;

    tracing::info!(job = %path.id, "Job cancelled by the client");

    Ok(StatusCode::ACCEPTED.into_response())
}
