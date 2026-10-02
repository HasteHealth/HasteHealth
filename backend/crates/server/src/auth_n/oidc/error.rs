// Custom OIDC error types
// Based on https://datatracker.ietf.org/doc/html/rfc6749#section-4.1.2.1 and https://openid.net/specs/openid-connect-core-1_0.html and 3.1.2.6

use axum::{
    http::{HeaderValue, StatusCode, header},
    response::{IntoResponse, Redirect},
};
use url::Url;

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

    #[serde(rename = "access_denied")]
    AccessDenied,
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
            OIDCErrorCode::AccessDenied => "access_denied",
        }
    }
}

#[allow(dead_code)]
#[derive(serde::Serialize, Debug)]
pub struct OIDCError {
    pub code: OIDCErrorCode,
    pub description: Option<String>,
    /// Where the user agent is sent with the error, per RFC 6749 §4.1.2.1.
    ///
    /// Only a redirect URI that has already been validated against the client
    /// application's registered redirect URIs may be set here. The response
    /// redirects to it without further checks, so an unvalidated value turns
    /// the authorization server into an open redirector. RFC 6749 §4.1.2.1
    /// says a request with an invalid redirect URI must not redirect at all,
    /// and the token endpoint (§5.2) never redirects: both leave this `None`.
    #[serde(skip_serializing)]
    pub redirect_uri: Option<String>,
}

impl OIDCError {
    pub fn new(
        code: OIDCErrorCode,
        description: Option<String>,
        redirect_uri: Option<String>,
    ) -> Self {
        OIDCError {
            code,
            description,
            redirect_uri,
        }
    }

    /// The URL the user agent is redirected to, with `error` and
    /// `error_description` added as query parameters.
    ///
    /// `None` when the error carries no redirect URI, or when that URI does
    /// not parse as an absolute URL: a malformed URI cannot be redirected to.
    /// The parameters are appended to any query the URI already has, and are
    /// percent-encoded, so neither the description nor an existing query can
    /// break the URL.
    fn error_redirect_url(&self) -> Option<Url> {
        let mut url = Url::parse(self.redirect_uri.as_deref()?).ok()?;
        let error_code: &str = (&self.code).into();

        {
            let mut query = url.query_pairs_mut();
            query.append_pair("error", error_code);

            if let Some(description) = self.description.as_deref() {
                query.append_pair("error_description", description);
            }
        }

        Some(url)
    }
}

impl IntoResponse for OIDCError {
    fn into_response(self) -> axum::response::Response {
        if let Some(redirect_url) = self.error_redirect_url() {
            Redirect::to(redirect_url.as_str()).into_response()
        } else {
            // RFC 6749 §5.2: a JSON body with the error code and description.
            let json_body = serde_json::to_string(&self).unwrap_or_default();

            let status = match self.code {
                OIDCErrorCode::InvalidRequest
                | OIDCErrorCode::InvalidGrant
                | OIDCErrorCode::InvalidClient
                | OIDCErrorCode::InvalidScope => StatusCode::BAD_REQUEST,
                OIDCErrorCode::UnauthorizedClient => StatusCode::UNAUTHORIZED,
                OIDCErrorCode::UnsupportedResponseType => StatusCode::UNPROCESSABLE_ENTITY,
                OIDCErrorCode::ServerError => StatusCode::INTERNAL_SERVER_ERROR,
                OIDCErrorCode::TemporarilyUnavailable => StatusCode::SERVICE_UNAVAILABLE,
                OIDCErrorCode::AccessDenied => StatusCode::FORBIDDEN,
            };

            (
                status,
                [(
                    header::CONTENT_TYPE,
                    HeaderValue::from_static("application/json"),
                )],
                json_body,
            )
                .into_response()
        }
    }
}
