use crate::CLIState;
use clap::Subcommand;
use jsonwebtoken::decode_header;
use oxidized_fhir_client::http::{self, FHIRHttpClient, FHIRHttpState};
use oxidized_fhir_model::r4::generated::terminology::IssueType;
use oxidized_fhir_operation_error::OperationOutcomeError;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Subcommand, Debug)]
pub enum ApiCommands {
    Transaction {},
}

async fn config_to_fhir_http_state(
    state: Arc<Mutex<CLIState>>,
) -> Result<FHIRHttpState, OperationOutcomeError> {
    let current_state = state.lock().await;
    let Some(active_profile) = current_state.config.current_profile() else {
        return Err(OperationOutcomeError::error(
            IssueType::Invalid(None),
            "No active profile set. Please set an active profile using the config command."
                .to_string(),
        ));
    };

    let state = state.clone();
    let http_state = FHIRHttpState::new(
        &active_profile.r4_url.clone(),
        Some(Arc::new(move || {
            let state = state.clone();
            Box::pin(async move {
                let current_state = state.lock().await;
                if let Some(token) = current_state.access_token.clone() {
                    Ok(token)
                } else {
                    let Some(active_profile) = current_state.config.current_profile() else {
                        return Err(OperationOutcomeError::error(
                            IssueType::Invalid(None),
                            "No active profile set. Please set an active profile using the config command."
                                .to_string(),
                        ));
                    };

                    Ok("".to_string())
                }
            })
        })),
    )?;

    Ok(http_state)
}

pub async fn api_commands(
    state: Arc<Mutex<CLIState>>,
    command: &ApiCommands,
) -> Result<(), OperationOutcomeError> {
    let http_state = config_to_fhir_http_state(state).await?;
    match command {
        ApiCommands::Transaction {} => {
            let fhir_client = FHIRHttpClient::<()>::new(http_state);

            todo!();
        }
    }
}
