use crate::{extract::path_tenant::TenantIdentifier, ui::pages::error::error_html};
use axum::{
    extract::Request,
    middleware::Next,
    response::{IntoResponse, Response},
};
use axum_extra::extract::Cached;
use maud::html;
use oxidized_fhir_operation_error::OperationOutcomeError;
use std::sync::Arc;

// Log operation outcome errors encountered during request processing.
pub async fn log_operationoutcome_errors(request: Request, next: Next) -> Response {
    let response = next.run(request).await;
    // If the response contains an AppError Extension, log it.
    if let Some(err) = response.extensions().get::<Arc<OperationOutcomeError>>() {
        tracing::error!(?err);
    }

    response
}

/// Middleware to handle OperationOutcomeErrors and return appropriate HTML or FHIR+JSON responses.
/// If the client accepts HTML, an error page is rendered.
/// Otherwise, a FHIR OperationOutcome JSON response is returned.
pub async fn operation_outcome_error_handle(
    Cached(TenantIdentifier { tenant }): Cached<TenantIdentifier>,
    request: Request,
    next: Next,
) -> Response {
    let content_type = request
        .headers()
        .get(axum::http::header::ACCEPT)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    let mut response = next.run(request).await;

    let error = response
        .extensions()
        .get::<Arc<OperationOutcomeError>>()
        .cloned();

    if let Some(err) = error {
        if content_type.contains("text/html") || content_type.contains("application/fhir+json") {
            let (parts, _) = response.into_parts();

            let outcome = err.outcome();
            let issue = outcome.issue.first();
            let body_html = error_html(
                &tenant,
                html! {
                    div class ="text-xl font-semibold text-red-600 mb-4" {
                       (issue.as_ref().map(|i| &i.code)
                            .and_then(|s| {let code_string: Option<String> =  s.as_ref().into(); code_string})
                            .unwrap_or_else(|| "UNKNOWN_ERROR".to_string()).to_ascii_uppercase())
                    }
                    div class= "text-sm text-red-500" {
                        (issue.as_ref().and_then(|i| i.diagnostics.as_ref())
                            .and_then(|d| d.value.as_ref())
                            .map(|s| s.as_str())
                            .unwrap_or("An unexpected error occurred."))
                    }
                },
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
