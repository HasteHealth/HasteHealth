use crate::mcp::schemas::types::RequestId;
use axum::response::IntoResponse;
use haste_fhir_model::r4::generated::resources::{OperationOutcome, OperationOutcomeIssue};
use haste_fhir_operation_error::OperationOutcomeError;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct MCPErrorDetail<T> {
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
        let code = self.error.code;
        let body = serde_json::to_string(&self).unwrap_or_else(|_| {
            "{\"jsonrpc\":\"2.0\",\"error\":{\"code\":500,\"message\":\"Failed to serialize MCPError\"}}".to_string()
        });

        let mut headers = axum::http::HeaderMap::new();
        headers.insert(
            axum::http::header::CONTENT_TYPE,
            "application/json".parse().unwrap(),
        );

        (
            axum::http::StatusCode::from_u16(code)
                .unwrap_or(axum::http::StatusCode::INTERNAL_SERVER_ERROR),
            headers,
            body,
        )
            .into_response()
    }
}
