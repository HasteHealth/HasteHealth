use crate::{
    CLIState, SECRETS_LOCATION,
    client::{TokenResponseBody, fetch_discovery_document, unix_now},
    commands::config::ProfileAuth,
    secrets::StoredTokens,
};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use haste_fhir_model::r4::generated::terminology::IssueType;
use haste_fhir_operation_error::OperationOutcomeError;
use sha2::{Digest, Sha256};
use std::{
    io::{BufRead, BufReader, Write},
    net::TcpListener,
    sync::Arc,
};
use tokio::sync::Mutex;

fn random_url_safe_string(byte_len: usize) -> String {
    let bytes: Vec<u8> = (0..byte_len).map(|_| rand::random::<u8>()).collect();
    URL_SAFE_NO_PAD.encode(bytes)
}

fn code_challenge_s256(code_verifier: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(code_verifier.as_bytes());
    URL_SAFE_NO_PAD.encode(hasher.finalize())
}

fn open_in_browser(url: &str) {
    let result = if cfg!(target_os = "macos") {
        std::process::Command::new("open").arg(url).status()
    } else if cfg!(target_os = "windows") {
        std::process::Command::new("cmd")
            .args(["/C", "start", "", url])
            .status()
    } else {
        std::process::Command::new("xdg-open").arg(url).status()
    };

    if !result.is_ok_and(|s| s.success()) {
        println!("Could not open a browser automatically. Please open this URL manually:\n{url}");
    }
}

fn parse_redirect_port(redirect_uri: &str) -> Result<u16, OperationOutcomeError> {
    let url = reqwest::Url::parse(redirect_uri).map_err(|e| {
        OperationOutcomeError::error(
            IssueType::exception(),
            format!("Invalid redirect_uri in profile: {}", e),
        )
    })?;

    url.port().ok_or_else(|| {
        OperationOutcomeError::error(
            IssueType::exception(),
            "Profile's redirect_uri must include an explicit port, e.g. 'http://127.0.0.1:8976/callback'."
                .to_string(),
        )
    })
}

/// Blocks waiting for the browser to redirect back with `?code=&state=`, on a single
/// connection to the loopback listener. Returns (code, state).
fn wait_for_callback(
    port: u16,
    expected_path: &str,
) -> Result<(String, String), OperationOutcomeError> {
    let listener = TcpListener::bind(("127.0.0.1", port)).map_err(|e| {
        OperationOutcomeError::error(
            IssueType::exception(),
            format!(
                "Failed to bind local callback listener on port {}: {}",
                port, e
            ),
        )
    })?;

    let (stream, _) = listener.accept().map_err(|e| {
        OperationOutcomeError::error(
            IssueType::exception(),
            format!("Failed to accept local callback connection: {}", e),
        )
    })?;

    let mut reader = BufReader::new(stream.try_clone().map_err(|e| {
        OperationOutcomeError::error(
            IssueType::exception(),
            format!("Failed to read callback request: {}", e),
        )
    })?);

    let mut request_line = String::new();
    reader.read_line(&mut request_line).map_err(|e| {
        OperationOutcomeError::error(
            IssueType::exception(),
            format!("Failed to read callback request: {}", e),
        )
    })?;

    let path = request_line
        .split_whitespace()
        .nth(1)
        .unwrap_or("")
        .to_string();

    let mut stream = stream;
    let body = "<html><body><h3>Login complete.</h3><p>You can close this window and return to the terminal.</p></body></html>";
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    );
    let _ = stream.write_all(response.as_bytes());

    let full_url = format!("http://127.0.0.1{}", path);
    let parsed = reqwest::Url::parse(&full_url).map_err(|e| {
        OperationOutcomeError::error(
            IssueType::exception(),
            format!("Failed to parse callback URL: {}", e),
        )
    })?;

    if parsed.path() != expected_path {
        return Err(OperationOutcomeError::error(
            IssueType::exception(),
            format!("Received callback on unexpected path '{}'.", parsed.path()),
        ));
    }

    if let Some((_, error)) = parsed.query_pairs().find(|(k, _)| k == "error") {
        return Err(OperationOutcomeError::error(
            IssueType::security(),
            format!("Authorization failed: {}", error),
        ));
    }

    let code = parsed
        .query_pairs()
        .find(|(k, _)| k == "code")
        .map(|(_, v)| v.to_string())
        .ok_or_else(|| {
            OperationOutcomeError::error(
                IssueType::exception(),
                "No 'code' parameter present on callback.".to_string(),
            )
        })?;

    let state = parsed
        .query_pairs()
        .find(|(k, _)| k == "state")
        .map(|(_, v)| v.to_string())
        .ok_or_else(|| {
            OperationOutcomeError::error(
                IssueType::exception(),
                "No 'state' parameter present on callback.".to_string(),
            )
        })?;

    Ok((code, state))
}

/// Runs the browser-based authorization_code + PKCE login flow for the active profile
/// and caches the resulting tokens in the secrets file.
pub(crate) async fn login(state: Arc<Mutex<CLIState>>) -> Result<(), OperationOutcomeError> {
    let (client_id, redirect_uri, scope, profile_name) = {
        let current_state = state.lock().await;
        let Some(profile) = current_state.config.current_profile().cloned() else {
            return Err(OperationOutcomeError::error(
                IssueType::invalid(),
                "No active profile set. Please set an active profile using the config command."
                    .to_string(),
            ));
        };

        match profile.auth {
            ProfileAuth::AuthorizationCode {
                client_id,
                redirect_uri,
                scope,
            } => (client_id, redirect_uri, scope, profile.name),
            _ => {
                return Err(OperationOutcomeError::error(
                    IssueType::invalid(),
                    "The active profile is not configured for authorization-code login. Create one with `haste-health config create-profile --auth-mode authorization-code`."
                        .to_string(),
                ));
            }
        }
    };

    let well_known_document = fetch_discovery_document(&state).await?;

    let code_verifier = random_url_safe_string(64);
    let code_challenge = code_challenge_s256(&code_verifier);
    let oauth_state = random_url_safe_string(32);

    let port = parse_redirect_port(&redirect_uri)?;
    let expected_path = reqwest::Url::parse(&redirect_uri)
        .map(|u| u.path().to_string())
        .unwrap_or_else(|_| "/callback".to_string());

    let mut authorize_url = reqwest::Url::parse(&well_known_document.authorization_endpoint)
        .map_err(|e| {
            OperationOutcomeError::error(
                IssueType::exception(),
                format!(
                    "Invalid authorization_endpoint in discovery document: {}",
                    e
                ),
            )
        })?;
    authorize_url
        .query_pairs_mut()
        .append_pair("client_id", &client_id)
        .append_pair("response_type", "code")
        .append_pair("redirect_uri", &redirect_uri)
        .append_pair("scope", &scope)
        .append_pair("state", &oauth_state)
        .append_pair("code_challenge", &code_challenge)
        .append_pair("code_challenge_method", "S256");

    println!("Opening your browser to log in...");
    println!("If it doesn't open automatically, visit: {}", authorize_url);
    open_in_browser(authorize_url.as_str());

    let (code, returned_state) =
        tokio::task::spawn_blocking(move || wait_for_callback(port, &expected_path))
            .await
            .map_err(|e| {
                OperationOutcomeError::error(
                    IssueType::exception(),
                    format!("Login callback task failed: {}", e),
                )
            })??;

    if returned_state != oauth_state {
        return Err(OperationOutcomeError::error(
            IssueType::security(),
            "State mismatch on login callback; aborting.".to_string(),
        ));
    }

    let params = [
        ("grant_type", "authorization_code"),
        ("client_id", client_id.as_str()),
        ("code", code.as_str()),
        ("code_verifier", code_verifier.as_str()),
        ("redirect_uri", redirect_uri.as_str()),
    ];

    let res = reqwest::Client::new()
        .post(&well_known_document.token_endpoint)
        .form(&params)
        .send()
        .await
        .map_err(|e| {
            OperationOutcomeError::error(
                IssueType::exception(),
                format!("Failed to exchange authorization code: {}", e),
            )
        })?;

    if !res.status().is_success() {
        let status = res.status();
        let body = res.text().await.unwrap_or_default();
        return Err(OperationOutcomeError::error(
            IssueType::forbidden(),
            format!(
                "Failed to exchange authorization code: HTTP {} - {}",
                status, body
            ),
        ));
    }

    let token_response: TokenResponseBody = res.json().await.map_err(|e| {
        OperationOutcomeError::error(
            IssueType::exception(),
            format!("Failed to parse token response: {}", e),
        )
    })?;

    let mut current_state = state.lock().await;
    current_state.access_token = Some(token_response.access_token.clone());

    current_state.secrets.profile_mut(&profile_name).tokens = Some(StoredTokens {
        access_token: token_response.access_token,
        refresh_token: token_response.refresh_token,
        id_token: token_response.id_token,
        expires_at: unix_now() + token_response.expires_in,
    });

    crate::secrets::write_secrets(&SECRETS_LOCATION, &current_state.secrets)?;

    println!(
        "Login successful. Profile '{}' is now authenticated.",
        profile_name
    );

    Ok(())
}
