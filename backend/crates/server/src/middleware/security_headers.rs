//! Security headers on every response. Mostly helmet's defaults.

use axum::{
    http::{
        HeaderMap, HeaderName, HeaderValue,
        header::{
            CONTENT_SECURITY_POLICY, REFERRER_POLICY, STRICT_TRANSPORT_SECURITY,
            X_CONTENT_TYPE_OPTIONS, X_DNS_PREFETCH_CONTROL, X_FRAME_OPTIONS,
        },
    },
    response::Response,
};
use tower::util::MapResponseLayer;

/// Origin of the Cloudflare Turnstile widget's script and frame.
pub const TURNSTILE_ORIGIN: &str = "https://challenges.cloudflare.com";

const STATIC_HEADERS: [(HeaderName, HeaderValue); 10] = [
    (
        HeaderName::from_static("cross-origin-opener-policy"),
        HeaderValue::from_static("same-origin"),
    ),
    (
        HeaderName::from_static("cross-origin-resource-policy"),
        HeaderValue::from_static("same-origin"),
    ),
    (
        HeaderName::from_static("origin-agent-cluster"),
        HeaderValue::from_static("?1"),
    ),
    (REFERRER_POLICY, HeaderValue::from_static("no-referrer")),
    (
        STRICT_TRANSPORT_SECURITY,
        HeaderValue::from_static("max-age=31536000; includeSubDomains"),
    ),
    (X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static("nosniff")),
    (X_DNS_PREFETCH_CONTROL, HeaderValue::from_static("off")),
    (
        HeaderName::from_static("x-download-options"),
        HeaderValue::from_static("noopen"),
    ),
    (X_FRAME_OPTIONS, HeaderValue::from_static("SAMEORIGIN")),
    (
        HeaderName::from_static("x-permitted-cross-domain-policies"),
        HeaderValue::from_static("none"),
    ),
];

/// `turnstile` lets the sign-up form load the Turnstile widget's script and frame.
fn content_security_policy(turnstile: bool) -> HeaderValue {
    let (script_src, frame_src) = if turnstile {
        (
            format!("'self' {TURNSTILE_ORIGIN}"),
            format!(";frame-src 'self' {TURNSTILE_ORIGIN}"),
        )
    } else {
        ("'self'".to_string(), String::new())
    };

    format!(
        "default-src 'self';base-uri 'self';font-src 'self' https: data:;frame-ancestors 'self'{frame_src};img-src 'self' data:;object-src 'none';script-src {script_src};script-src-attr 'none';style-src 'self' https: 'unsafe-inline';upgrade-insecure-requests"
    )
    .parse()
    .expect("content security policy is a valid header value")
}

/// Layer that sets the security headers on every response.
pub fn security_headers(
    turnstile: bool,
) -> MapResponseLayer<impl Fn(Response) -> Response + Clone> {
    let mut headers = HeaderMap::from_iter(STATIC_HEADERS);
    headers.insert(CONTENT_SECURITY_POLICY, content_security_policy(turnstile));

    MapResponseLayer::new(move |mut response: Response| {
        response.headers_mut().extend(headers.clone());
        response
    })
}
