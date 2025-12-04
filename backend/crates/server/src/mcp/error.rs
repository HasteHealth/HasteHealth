use crate::mcp::schemas::schema_2025_11_25::RequestId;
use axum::response::IntoResponse;
use haste_fhir_operation_error::OperationOutcomeError;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct MCPErrorDetail<T> {
    code: u16,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<T>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct MCPError<T> {
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<RequestId>,
    jsonrpc: String,
    error: MCPErrorDetail<T>,
}

impl From<OperationOutcomeError> for MCPError<serde_json::Value> {
    fn from(err: OperationOutcomeError) -> Self {
        let message = err.to_string();
        let status_code = err.status();
        let outcome = err.outcome();

        if let Ok(json_string) = haste_fhir_serialization_json::to_string(outcome)
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
