use crate::mcp::schemas::types::RequestId;
use axum::{http::StatusCode, response::IntoResponse};
use haste_fhir_model::r4::generated::resources::{OperationOutcome, OperationOutcomeIssue};
use haste_fhir_operation_error::OperationOutcomeError;

/// JSON-RPC 2.0 error codes (<https://www.jsonrpc.org/specification#error_object>).
const METHOD_NOT_FOUND: i32 = -32601;
const INVALID_PARAMS: i32 = -32602;
const INTERNAL_ERROR: i32 = -32603;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct MCPErrorDetail<T> {
    /// HTTP status of the failure. The response carries a JSON-RPC error code
    /// in its place (see [`jsonrpc_code_and_status`]).
    pub code: u16,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct MCPError<T> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<RequestId>,
    pub jsonrpc: String,
    pub error: MCPErrorDetail<T>,
}

impl<T> MCPError<T> {
    /// Attach the id of the request this error answers, unless it has one.
    pub fn with_id(mut self, id: Option<RequestId>) -> Self {
        if self.id.is_none() {
            self.id = id;
        }
        self
    }
}

impl From<OperationOutcomeError> for MCPError<serde_json::Value> {
    fn from(err: OperationOutcomeError) -> Self {
        let message = outcome_message(err.outcome());
        let status_code = err.status();
        let outcome = err.outcome();

        if let Ok(json_string) = serde_json::to_string(outcome)
            && let Ok(data) = serde_json::to_value(&json_string)
        {
            MCPError {
                id: None,
                jsonrpc: "2.0".to_string(),
                error: MCPErrorDetail {
                    code: status_code.as_u16(),
                    message,
                    data: Some(data),
                },
            }
        } else {
            MCPError {
                id: None,
                jsonrpc: "2.0".to_string(),
                error: MCPErrorDetail {
                    code: status_code.as_u16(),
                    message,
                    data: None,
                },
            }
        }
    }
}

/// The JSON-RPC error code and HTTP status to answer a failure with.
///
/// A JSON-RPC error is an ordinary response, so it goes back with HTTP 200 and
/// a JSON-RPC code. The exceptions are authentication, authorization and rate
/// limiting: MCP clients act on those HTTP statuses (sign in again, ask for more
/// scopes, back off), so they keep them, and the status doubles as the code,
/// which JSON-RPC allows outside its reserved range.
fn jsonrpc_code_and_status(status: StatusCode) -> (i32, StatusCode) {
    match status {
        StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN | StatusCode::TOO_MANY_REQUESTS => {
            (i32::from(status.as_u16()), status)
        }
        StatusCode::NOT_IMPLEMENTED => (METHOD_NOT_FOUND, StatusCode::OK),
        status if status.is_client_error() => (INVALID_PARAMS, StatusCode::OK),
        _ => (INTERNAL_ERROR, StatusCode::OK),
    }
}

/// An error response as sent. JSON-RPC codes are negative, so they do not fit
/// [`MCPErrorDetail::code`].
#[derive(serde::Serialize)]
struct ErrorResponse<'a, T> {
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<&'a RequestId>,
    jsonrpc: &'a str,
    error: ErrorResponseDetail<'a, T>,
}

#[derive(serde::Serialize)]
struct ErrorResponseDetail<'a, T> {
    code: i32,
    message: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<&'a T>,
}

/// One issue as a model should read it: severity and code, the diagnostics
/// (or the details text), and the element path when the server names one.
fn issue_line(issue: &OperationOutcomeIssue) -> String {
    let mut line = format!(
        "{} ({})",
        issue.severity.as_str().unwrap_or("error"),
        issue.code.as_str().unwrap_or("unknown")
    );

    let text = issue
        .diagnostics
        .as_ref()
        .and_then(|d| d.value.as_deref())
        .or_else(|| {
            issue
                .details
                .as_ref()
                .and_then(|d| d.text.as_ref())
                .and_then(|t| t.value.as_deref())
        });
    if let Some(text) = text {
        line.push_str(": ");
        line.push_str(text);
    }

    let paths: Vec<&str> = issue
        .expression
        .as_deref()
        .unwrap_or_default()
        .iter()
        .filter_map(|e| e.value.as_deref())
        .collect();
    if !paths.is_empty() {
        line.push_str(" at ");
        line.push_str(&paths.join(", "));
    }

    line
}

/// The error message for an outcome: every issue, not only the diagnostics
/// of each joined by commas, so a validation failure says which element.
fn outcome_message(outcome: &OperationOutcome) -> String {
    let lines: Vec<String> = outcome.issue.iter().map(issue_line).collect();

    if lines.is_empty() {
        "Operation failed without issue details".to_string()
    } else {
        lines.join("; ")
    }
}

impl IntoResponse for MCPError<serde_json::Value> {
    fn into_response(self) -> axum::response::Response {
        let (code, status) = jsonrpc_code_and_status(
            StatusCode::from_u16(self.error.code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
        );

        let body = serde_json::to_string(&ErrorResponse {
            id: self.id.as_ref(),
            jsonrpc: &self.jsonrpc,
            error: ErrorResponseDetail {
                code,
                message: &self.error.message,
                data: self.error.data.as_ref(),
            },
        })
        .unwrap_or_else(|_| {
            format!(
                "{{\"jsonrpc\":\"2.0\",\"error\":{{\"code\":{INTERNAL_ERROR},\"message\":\"Failed to serialize MCPError\"}}}}"
            )
        });

        let mut headers = axum::http::HeaderMap::new();
        headers.insert(
            axum::http::header::CONTENT_TYPE,
            "application/json".parse().unwrap(),
        );

        (status, headers, body).into_response()
    }
}
