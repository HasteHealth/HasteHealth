use crate::{ui::components::TenantContext, ui::pages::error::error_html};
use axum::{
    extract::{OriginalUri, Request},
    http::{HeaderName, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use axum_extra::extract::Cached;
use haste_fhir_operation_error::OperationOutcomeError;
use maud::html;
use std::sync::Arc;

/// The header that carries the request id the server assigns (or the client sent).
pub const REQUEST_ID_HEADER: HeaderName = HeaderName::from_static("x-request-id");

/// Logs the error a response carries: at error level when the server failed
/// (5xx), at warn for a feature it does not implement (501) and at debug
/// when it rejected the request. The error's Debug output has the issues,
/// origin, context trail, source chain and backtrace.
pub async fn log_operationoutcome_errors(request: Request, next: Next) -> Response {
    // The request is consumed before we know whether it failed. These are
    // reference-count bumps, not copies. The original URI still has the
    // `/w/{tenant}` prefix the nested router strips.
    let method = request.method().clone();
    let uri = request
        .extensions()
        .get::<OriginalUri>()
        .map_or_else(|| request.uri().clone(), |original| original.0.clone());
    let request_id = request.headers().get(&REQUEST_ID_HEADER).cloned();

    let response = next.run(request).await;

    if let Some(error) = response.extensions().get::<Arc<OperationOutcomeError>>() {
        let request_id = request_id
            .as_ref()
            .and_then(|id| id.to_str().ok())
            .unwrap_or("");
        let status = response.status();
        if status == StatusCode::NOT_IMPLEMENTED {
            tracing::warn!(%method, %uri, request_id, origin = %error.origin(), ?error, "request unsupported");
        } else if status.is_server_error() {
            tracing::error!(%method, %uri, request_id, origin = %error.origin(), ?error, "request failed");
        } else {
            tracing::debug!(%method, %uri, request_id, origin = %error.origin(), ?error, "request rejected");
        }
    }

    response
}

/// Answers an `OperationOutcomeError` as an HTML error page when the client
/// accepts HTML, otherwise as the FHIR `OperationOutcome` JSON.
pub async fn operation_outcome_error_handle(
    Cached(TenantContext { tenant, branding }): Cached<TenantContext>,
    request: Request,
    next: Next,
) -> Response {
    let accepts_html = request
        .headers()
        .get(axum::http::header::ACCEPT)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|accept| accept.contains("text/html"));

    let mut response = next.run(request).await;

    let error = response
        .extensions()
        .get::<Arc<OperationOutcomeError>>()
        .cloned();

    if let Some(err) = error {
        if accepts_html {
            let (parts, _) = response.into_parts();

            let outcome = err.outcome();
            let issue = outcome.issue.first();
            let body_html = error_html(
                &tenant,
                &html! {
                    div class ="text-xl font-semibold text-red-600 mb-4" {
                       (issue.and_then(|i| i.code.as_str())
                            .unwrap_or("UNKNOWN_ERROR").to_ascii_uppercase())
                    }
                    div class= "text-sm text-red-500" {
                        (issue.and_then(|i| i.diagnostics.as_ref())
                            .and_then(|d| d.value.as_deref())
                            .unwrap_or("An unexpected error occurred."))
                    }
                },
                Some(&branding),
            );
            let mut html_response = body_html.into_response();
            let status_mut = html_response.status_mut();
            *status_mut = parts.status;
            let headers_mut = html_response.headers_mut();
            *headers_mut = parts.headers;

            headers_mut.insert(
                axum::http::header::CONTENT_TYPE,
                axum::http::HeaderValue::from_static("text/html; charset=utf-8"),
            );

            html_response
        } else {
            response.headers_mut().insert(
                axum::http::header::CONTENT_TYPE,
                axum::http::HeaderValue::from_static("application/fhir+json"),
            );
            response
        }
    } else {
        response
    }
}
