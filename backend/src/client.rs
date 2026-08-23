use crate::{CLIState, SECRETS_LOCATION, secrets::StoredTokens};
use haste_fhir_client::http::{FHIRHttpClient, FHIRHttpState};
use haste_fhir_model::r4::generated::terminology::IssueType;
use haste_fhir_operation_error::OperationOutcomeError;
use haste_server::auth_n::oidc::routes::discovery::WellKnownDiscoveryDocument;
use serde::Deserialize;
use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};
use tokio::sync::Mutex;

#[derive(Deserialize)]
pub(crate) struct TokenResponseBody {
    pub(crate) access_token: String,
    #[serde(default)]
    pub(crate) refresh_token: Option<String>,
    #[serde(default)]
    pub(crate) id_token: Option<String>,
    pub(crate) expires_in: i64,
}

pub(crate) fn unix_now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

/// Fetches (and caches on `CLIState`) the OIDC discovery document for the active profile.
pub(crate) async fn fetch_discovery_document(
    state: &Arc<Mutex<CLIState>>,
) -> Result<WellKnownDiscoveryDocument, OperationOutcomeError> {
    let mut current_state = state.lock().await;

    if let Some(well_known_doc) = &current_state.well_known_document {
        return Ok(well_known_doc.clone());
    }

    let Some(active_profile) = current_state.config.current_profile().cloned() else {
        return Err(OperationOutcomeError::error(
            IssueType::invalid(),
            "No active profile set. Please set an active profile using the config command."
                .to_string(),
        ));
    };

    let res = reqwest::get(&active_profile.oidc_discovery_uri)
        .await
        .map_err(|e| {
            OperationOutcomeError::error(
                IssueType::exception(),
                format!("Failed to fetch OIDC discovery document: {}", e),
            )
        })?;

    let well_known_document =
        serde_json::from_slice::<WellKnownDiscoveryDocument>(&res.bytes().await.map_err(|e| {
            OperationOutcomeError::error(
                IssueType::exception(),
                format!("Failed to read OIDC discovery document: {}", e),
            )
        })?)
        .map_err(|e| {
            OperationOutcomeError::error(
                IssueType::exception(),
                format!("Failed to parse OIDC discovery document: {}", e),
            )
        })?;

    current_state.well_known_document = Some(well_known_document.clone());

    Ok(well_known_document)
}

/// Exchanges a refresh token for a new access token, persisting the refreshed tokens to disk.
pub(crate) async fn refresh_access_token(
    state: &Arc<Mutex<CLIState>>,
    client_id: &str,
    profile_name: &str,
    refresh_token: &str,
) -> Result<String, OperationOutcomeError> {
    let well_known_document = fetch_discovery_document(state).await?;

    let params = [
        ("grant_type", "refresh_token"),
        ("client_id", client_id),
        ("refresh_token", refresh_token),
    ];

    let res = reqwest::Client::new()
        .post(&well_known_document.token_endpoint)
        .form(&params)
        .send()
        .await
        .map_err(|e| {
            OperationOutcomeError::error(
                IssueType::exception(),
                format!("Failed to refresh access token: {}", e),
            )
        })?;

    if !res.status().is_success() {
        return Err(OperationOutcomeError::error(
            IssueType::forbidden(),
            format!(
                "Failed to refresh access token: HTTP '{}'. Run `haste-health login` again.",
                res.status(),
            ),
        ));
    }

    let token_response: TokenResponseBody = res.json().await.map_err(|e| {
        OperationOutcomeError::error(
            IssueType::exception(),
            format!("Failed to parse refresh token response: {}", e),
        )
    })?;

    let mut current_state = state.lock().await;
    current_state.access_token = Some(token_response.access_token.clone());

    current_state.secrets.profile_mut(profile_name).tokens = Some(StoredTokens {
        access_token: token_response.access_token.clone(),
        refresh_token: token_response
            .refresh_token
            .or(Some(refresh_token.to_string())),
        id_token: token_response.id_token,
        expires_at: unix_now() + token_response.expires_in,
    });

    crate::secrets::write_secrets(&SECRETS_LOCATION, &current_state.secrets)?;

    Ok(token_response.access_token)
}

async fn config_to_fhir_http_state(
    state: Arc<Mutex<CLIState>>,
) -> Result<FHIRHttpState, OperationOutcomeError> {
    let current_state = state.lock().await;
    let Some(active_profile) = current_state.config.current_profile().cloned() else {
        return Err(OperationOutcomeError::error(
            IssueType::invalid(),
            "No active profile set. Please set an active profile using the config command."
                .to_string(),
        ));
    };

    let profile_name = active_profile.name.clone();
    let client_secret = current_state
        .secrets
        .profile(&profile_name)
        .and_then(|s| s.client_secret.clone());
    drop(current_state);

    let state = state.clone();
    let http_state = FHIRHttpState::new(
        &active_profile.r4_url.clone(),
        match active_profile.auth {
            crate::commands::config::ProfileAuth::Public {} => None,
            crate::commands::config::ProfileAuth::ClientCredentails { client_id } => {
                let Some(client_secret) = client_secret else {
                    return Err(OperationOutcomeError::error(
                        IssueType::invalid(),
                        format!(
                            "No client secret stored for profile '{}'. Recreate it with `haste-health config create-profile`.",
                            profile_name
                        ),
                    ));
                };

                Some(Arc::new(move || {
                    let state = state.clone();
                    let client_id = client_id.clone();
                    let client_secret = client_secret.clone();
                    Box::pin(async move {
                        {
                            let current_state = state.lock().await;
                            if let Some(token) = current_state.access_token.clone() {
                                return Ok(token);
                            }
                        }

                        let well_known_document = fetch_discovery_document(&state).await?;

                        // Post for JWT Token
                        let params = [
                            ("grant_type", "client_credentials"),
                            ("client_id", &client_id),
                            ("client_secret", &client_secret),
                            ("scope", "openid system/*.*"),
                        ];

                        let res: reqwest::Response = reqwest::Client::new()
                            .post(&well_known_document.token_endpoint)
                            .form(&params)
                            .send()
                            .await
                            .map_err(|e| {
                                OperationOutcomeError::error(
                                    IssueType::exception(),
                                    format!("Failed to fetch access token: {}", e),
                                )
                            })?;

                        if !res.status().is_success() {
                            return Err(OperationOutcomeError::error(
                                IssueType::forbidden(),
                                format!("Failed to fetch access token: HTTP '{}'", res.status(),),
                            ));
                        }

                        let token_response: serde_json::Value = res.json().await.map_err(|e| {
                            OperationOutcomeError::error(
                                IssueType::exception(),
                                format!("Failed to parse access token response: {}", e),
                            )
                        })?;

                        let access_token = token_response
                            .get("access_token")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| {
                                OperationOutcomeError::error(
                                    IssueType::exception(),
                                    "No access_token field in token response".to_string(),
                                )
                            })?
                            .to_string();

                        state.lock().await.access_token = Some(access_token.clone());

                        Ok(access_token)
                    })
                }))
            }
            crate::commands::config::ProfileAuth::AuthorizationCode {
                client_id,
                redirect_uri: _,
                scope: _,
            } => {
                Some(Arc::new(move || {
                    let state = state.clone();
                    let client_id = client_id.clone();
                    let profile_name = profile_name.clone();
                    Box::pin(async move {
                        if let Some(token) = state.lock().await.access_token.clone() {
                            return Ok(token);
                        }

                        let stored_tokens = {
                            let current_state = state.lock().await;
                            current_state
                                .secrets
                                .profile(&profile_name)
                                .and_then(|s| s.tokens.clone())
                        };

                        let Some(tokens) = stored_tokens else {
                            return Err(OperationOutcomeError::error(
                                IssueType::forbidden(),
                                "Not logged in. Run `haste-health login` first.".to_string(),
                            ));
                        };

                        // Small buffer so a token doesn't expire mid-request.
                        if tokens.expires_at > unix_now() + 30 {
                            state.lock().await.access_token = Some(tokens.access_token.clone());
                            return Ok(tokens.access_token);
                        }

                        let Some(refresh_token) = tokens.refresh_token else {
                            return Err(OperationOutcomeError::error(
                                IssueType::forbidden(),
                                "Login session expired. Run `haste-health login` again."
                                    .to_string(),
                            ));
                        };

                        refresh_access_token(&state, &client_id, &profile_name, &refresh_token)
                            .await
                    })
                }))
            }
        },
    )?;

    Ok(http_state)
}

pub(crate) async fn fhir_client(
    state: Arc<Mutex<CLIState>>,
) -> Result<Arc<FHIRHttpClient<()>>, OperationOutcomeError> {
    let http_state = config_to_fhir_http_state(state).await?;
    let fhir_client = Arc::new(FHIRHttpClient::<()>::new(http_state));

    Ok(fhir_client)
}
