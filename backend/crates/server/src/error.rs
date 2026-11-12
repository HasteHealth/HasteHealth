//! Server-wide failures that wrap a foreign error. A `#[from]` field keeps
//! that error as the source, so `map_err(ServerError::from)` is all a call
//! site needs and the log still shows what went wrong.

use haste_fhir_operation_error::derive::OperationOutcomeError;

#[derive(OperationOutcomeError, Debug)]
pub enum ServerError {
    /// A response or stored value the server built did not serialize.
    #[fatal(code = "exception", diagnostic = "Failed to serialize the response.")]
    Serialize(#[from] serde_json::Error),
    /// The configured API URL does not parse, or a path cannot be joined to it.
    #[fatal(code = "exception", diagnostic = "Invalid API URL configured.")]
    InvalidApiUrl(#[from] url::ParseError),
    /// The configured API URL is not a valid `http::Uri` either.
    #[fatal(code = "exception", diagnostic = "Invalid API URL configured.")]
    InvalidApiUri(#[from] axum::http::uri::InvalidUri),
}
