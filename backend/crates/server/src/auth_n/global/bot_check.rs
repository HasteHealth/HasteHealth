//! Keeps bots off the email form: a honeypot field, and a Cloudflare
//! Turnstile token when configured. Rate limits are the caller's job.

use crate::config::SignupConfig;
use haste_fhir_model::r4::generated::terminology::IssueType;
use haste_fhir_operation_error::OperationOutcomeError;
use serde::Deserialize;
use std::{net::IpAddr, sync::LazyLock, time::Duration};

/// Hidden form field that only automated form fillers complete.
pub const HONEYPOT_FIELD: &str = "website";
/// Form field the Turnstile widget fills in.
pub const TURNSTILE_FIELD: &str = "cf-turnstile-response";

const TURNSTILE_VERIFY_URL: &str = "https://challenges.cloudflare.com/turnstile/v0/siteverify";

static HTTP: LazyLock<reqwest::Client> = LazyLock::new(|| {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .expect("reqwest client")
});

pub enum Verdict {
    Human,
    /// The reason, for the log.
    Bot(&'static str),
}

#[derive(Deserialize)]
struct TurnstileResponse {
    success: bool,
    #[serde(default, rename = "error-codes")]
    error_codes: Vec<String>,
}

fn unavailable(error: reqwest::Error) -> OperationOutcomeError {
    tracing::error!(%error, "Turnstile verification failed");
    OperationOutcomeError::fatal(
        IssueType::exception(),
        "The bot check is unavailable right now. Please try again shortly.".to_string(),
    )
}

/// Errors only when Turnstile cannot be reached: the form then refuses rather
/// than letting bots through.
pub async fn check(
    config: &SignupConfig,
    honeypot: Option<&str>,
    turnstile_token: Option<&str>,
    ip: Option<IpAddr>,
) -> Result<Verdict, OperationOutcomeError> {
    if honeypot.is_some_and(|value| !value.trim().is_empty()) {
        return Ok(Verdict::Bot("honeypot field filled"));
    }

    let Some(turnstile) = &config.turnstile else {
        return Ok(Verdict::Human);
    };

    let Some(token) = turnstile_token.filter(|token| !token.is_empty()) else {
        return Ok(Verdict::Bot("turnstile token missing"));
    };

    let remote_ip = ip.map(|ip| ip.to_string());
    let mut form = vec![
        ("secret", turnstile.secret_key.as_str()),
        ("response", token),
    ];
    form.extend(remote_ip.as_deref().map(|ip| ("remoteip", ip)));

    let response: TurnstileResponse = HTTP
        .post(TURNSTILE_VERIFY_URL)
        .form(&form)
        .send()
        .await
        .and_then(reqwest::Response::error_for_status)
        .map_err(unavailable)?
        .json()
        .await
        .map_err(unavailable)?;

    if response.success {
        Ok(Verdict::Human)
    } else {
        tracing::info!(error_codes = ?response.error_codes, "Turnstile rejected the token");
        Ok(Verdict::Bot("turnstile rejected the token"))
    }
}
