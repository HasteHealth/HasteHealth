// Custom OIDC error types
// Based on https://datatracker.ietf.org/doc/html/rfc6749#section-4.1.2.1 and https://openid.net/specs/openid-connect-core-1_0.html and 3.1.2.6

use axum::response::{IntoResponse, Redirect};

#[derive(serde::Serialize, Debug)]
pub enum OIDCErrorCode {
    /**
     * The request is missing a required parameter, includes aninvalid parameter value,
     * includes a parameter more than once, or is otherwise malformed.
     */
    #[serde(rename = "invalid_request")]
    InvalidRequest,
    /**
     * The client is not authorized to request an authorization
     * code using this method.
     */
    #[serde(rename = "unauthorized_client")]
    UnauthorizedClient,
    /**
     * The authorization server does not support obtaining an
     * authorization code using this method.
     */
    #[serde(rename = "unsupported_response_type")]
    UnsupportedResponseType,
    /**
     * The requested scope is invalid, unknown, or malformed.
     */
    #[serde(rename = "invalid_scope")]
    InvalidScope,
    /**
     * The authorization server encountered an unexpected
     * condition that prevented it from fulfilling the request.
     * (This error code is needed because a 500 Internal Server
     * Error HTTP status code cannot be returned to the client
     * via an HTTP redirect.)
     */
    #[serde(rename = "server_error")]
    ServerError,
    /**
     * The authorization server is currently unable to handle
     * the request due to a temporary overloading or maintenance
     * of the server.  (This error code is needed because a 503
     * Service Unavailable HTTP status code cannot be returned
     * to the client via an HTTP redirect.)
     */
    #[serde(rename = "temporarily_unavailable")]
    TemporarilyUnavailable,
    /**
     * Client authentication failed (e.g., unknown client, no
     * client authentication included, or unsupported
     * authentication method).  The authorization server MAY
     * return an HTTP 401 (Unauthorized) status code to indicate
     * which HTTP authentication schemes are supported.  If the
     * client attempted to authenticate via the "Authorization"
     * request header field, the authorization server MUST
     * respond with an HTTP 401 (Unauthorized) status code and
     * include the "WWW-Authenticate" response header field
     * matching the authentication scheme used by the client.
     */
    #[serde(rename = "invalid_client")]
    InvalidClient,
    /**
     *  The provided authorization grant (e.g., authorization
     * code, resource owner credentials) or refresh token is
     * invalid, expired, revoked, does not match the redirection
     * URI used in the authorization request, or was issued to
     * another client.
     */
    #[serde(rename = "invalid_grant")]
    InvalidGrant,
}

impl From<&OIDCErrorCode> for &str {
    fn from(code: &OIDCErrorCode) -> Self {
        match code {
            OIDCErrorCode::InvalidRequest => "invalid_request",
            OIDCErrorCode::UnauthorizedClient => "unauthorized_client",
            OIDCErrorCode::UnsupportedResponseType => "unsupported_response_type",
            OIDCErrorCode::InvalidScope => "invalid_scope",
            OIDCErrorCode::ServerError => "server_error",
            OIDCErrorCode::TemporarilyUnavailable => "temporarily_unavailable",
            OIDCErrorCode::InvalidClient => "invalid_client",
            OIDCErrorCode::InvalidGrant => "invalid_grant",
        }
    }
}

#[allow(dead_code)]
#[derive(serde::Serialize, Debug)]
pub struct OIDCError {
    pub code: OIDCErrorCode,
    pub description: Option<String>,
    #[serde(skip_serializing)]
    pub redirect_uri: Option<String>,
}

impl IntoResponse for OIDCError {
    fn into_response(self) -> axum::response::Response {
        let error_code: &str = (&self.code).into();

        if let Some(error_uri) = self.redirect_uri {
            let mut redirect_uri = error_uri + "?error=" + error_code;

            if let Some(description) = self.description {
                redirect_uri = redirect_uri + "&error_description=" + &description;
            }

            Redirect::to(&redirect_uri).into_response()
        } else {
            let json_body = serde_json::to_string(&self).unwrap_or_default();
            (axum::http::StatusCode::BAD_REQUEST, json_body).into_response()
        }
    }
}
