use crate::CLIState;
use haste_fhir_client::http::{FHIRHttpClient, FHIRHttpState};
use haste_fhir_model::r4::generated::terminology::IssueType;
use haste_fhir_operation_error::OperationOutcomeError;
use haste_server::auth_n::oidc::routes::discovery::WellKnownDiscoveryDocument;
use std::{borrow::Cow, sync::Arc};
use tokio::sync::Mutex;

async fn config_to_fhir_http_state(
    state: Arc<Mutex<CLIState>>,
) -> Result<FHIRHttpState, OperationOutcomeError> {
    let current_state = state.lock().await;
    let Some(active_profile) = current_state.config.current_profile().cloned() else {
        return Err(OperationOutcomeError::error(
            IssueType::Invalid(None),
            "No active profile set. Please set an active profile using the config command."
                .to_string(),
        ));
    };

    let state = state.clone();
    let http_state = FHIRHttpState::new(
        &active_profile.r4_url.clone(),
        match active_profile.auth {
            crate::commands::config::ProfileAuth::Public {} => None,
            crate::commands::config::ProfileAuth::ClientCredentails {
                client_id,
                client_secret,
            } => {
                Some(Arc::new(move || {
                    let state = state.clone();
                    let client_id = client_id.clone();
                    let client_secret = client_secret.clone();
                    Box::pin(async move {
                        let mut current_state = state.lock().await;
                        if let Some(token) = current_state.access_token.clone() {
                            Ok(token)
                        } else {
                            let Some(active_profile) = current_state.config.current_profile()
                            else {
                                return Err(OperationOutcomeError::error(
                            IssueType::Invalid(None),
                            "No active profile set. Please set an active profile using the config command.".to_string(),
                                ));
                            };

                            let well_known_document: Cow<WellKnownDiscoveryDocument> =
                                if let Some(well_known_doc) = &current_state.well_known_document {
                                    Cow::Borrowed(well_known_doc)
                                } else {
                                    let res =
                                        reqwest::get(&active_profile.oidc_discovery_uri).await;
                                    let res = res.map_err(|e| {
                                        OperationOutcomeError::error(
                                            IssueType::Exception(None),
                                            format!(
                                                "Failed to fetch OIDC discovery document: {}",
                                                e
                                            ),
                                        )
                                    })?;

                                    let well_known_document = serde_json::from_slice::<
                                        WellKnownDiscoveryDocument,
                                    >(
                                        &res.bytes().await.map_err(|e| {
                                            OperationOutcomeError::error(
                                                IssueType::Exception(None),
                                                format!(
                                                    "Failed to read OIDC discovery document: {}",
                                                    e
                                                ),
                                            )
                                        })?,
                                    )
                                    .map_err(|e| {
                                        OperationOutcomeError::error(
                                            IssueType::Exception(None),
                                            format!(
                                                "Failed to parse OIDC discovery document: {}",
                                                e
                                            ),
                                        )
                                    })?;

                                    current_state.well_known_document =
                                        Some(well_known_document.clone());

                                    Cow::Owned(well_known_document)
                                };

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
                                        IssueType::Exception(None),
                                        format!("Failed to fetch access token: {}", e),
                                    )
                                })?;

                            if !res.status().is_success() {
                                return Err(OperationOutcomeError::error(
                                    IssueType::Forbidden(None),
                                    format!(
                                        "Failed to fetch access token: HTTP '{}'",
                                        res.status(),
                                    ),
                                ));
                            }

                            let token_response: serde_json::Value =
                                res.json().await.map_err(|e| {
                                    OperationOutcomeError::error(
                                        IssueType::Exception(None),
                                        format!("Failed to parse access token response: {}", e),
                                    )
                                })?;

                            let access_token = token_response
                                .get("access_token")
                                .and_then(|v| v.as_str())
                                .ok_or_else(|| {
                                    OperationOutcomeError::error(
                                        IssueType::Exception(None),
                                        "No access_token field in token response".to_string(),
                                    )
                                })?
                                .to_string();

                            current_state.access_token = Some(access_token.clone());

                            Ok(access_token)
                        }
                    })
                }))
            }
        },
    )?;

    Ok(http_state)
}

pub async fn fhir_client(
    state: Arc<Mutex<CLIState>>,
) -> Result<Arc<FHIRHttpClient<()>>, OperationOutcomeError> {
    let http_state = config_to_fhir_http_state(state).await?;
    let fhir_client = Arc::new(FHIRHttpClient::<()>::new(http_state));

    Ok(fhir_client)
}
