//! Request plumbing shared by the storage and token calls.

use std::error::Error;

use reqwest::{Response, StatusCode};
use serde::{Deserialize, de::DeserializeOwned};

use crate::error::StorageError;

/// Longest raw error body quoted in an error message.
const MAX_ERROR_BODY: usize = 512;

/// Wraps a request failure, keeping the chain of causes (DNS, TLS, timeout)
/// that the top-level message leaves out.
pub(super) fn transport(error: impl Error) -> StorageError {
    let mut message = error.to_string();
    let mut cause = error.source();
    while let Some(next) = cause {
        message.push_str(": ");
        message.push_str(&next.to_string());
        cause = next.source();
    }
    StorageError::Transport(message)
}

/// Passes a successful response through and turns any other status into the
/// matching error. `subject` names what a 404 refers to.
pub(super) async fn check(response: Response, subject: &str) -> Result<Response, StorageError> {
    let status = response.status();
    if status.is_success() {
        return Ok(response);
    }

    let message = error_message(&response.text().await.unwrap_or_default());
    Err(match status {
        StatusCode::UNAUTHORIZED => StorageError::Unauthorized(message),
        StatusCode::FORBIDDEN => StorageError::Forbidden(message),
        StatusCode::NOT_FOUND => StorageError::NotFound(subject.to_string()),
        other => StorageError::Backend(other.as_u16(), message),
    })
}

/// Reads and deserializes a JSON response body.
pub(super) async fn parse_json<T: DeserializeOwned>(response: Response) -> Result<T, StorageError> {
    let status = response.status().as_u16();
    let body = response.bytes().await.map_err(transport)?;
    serde_json::from_slice(&body).map_err(|error| {
        StorageError::Backend(status, format!("unexpected response body: {error}"))
    })
}

#[derive(Deserialize)]
struct ErrorBody {
    error: ErrorDetail,
}

#[derive(Deserialize)]
struct ErrorDetail {
    message: String,
}

/// The `error.message` of a Google API error body, or the raw body, truncated,
/// when it is not one.
fn error_message(body: &str) -> String {
    match serde_json::from_str::<ErrorBody>(body) {
        Ok(parsed) => parsed.error.message,
        Err(_) => body.chars().take(MAX_ERROR_BODY).collect(),
    }
}
