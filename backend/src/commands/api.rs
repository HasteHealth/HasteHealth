#![allow(unused)]
use crate::CLIState;
use clap::Subcommand;
use haste_fhir_client::{
    FHIRClient,
    http::{FHIRHttpClient, FHIRHttpState},
    url::ParsedParameters,
};
use haste_fhir_model::r4::generated::{
    resources::{Bundle, Resource, ResourceType},
    terminology::IssueType,
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_serialization_json::FHIRJSONDeserializer;
use haste_server::auth_n::oidc::routes::WellKnownDiscoveryDocument;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Subcommand, Debug)]
pub enum ApiCommands {
    Create {
        #[arg(short, long)]
        file: Option<String>,
        resource_type: String,
    },
    Read {
        resource_type: String,
        id: String,
    },

    VersionRead {
        resource_type: String,
        id: String,
        version_id: String,
    },

    Patch {
        #[arg(short, long)]
        file: String,
        resource_type: String,
        id: String,
    },
    Update {
        #[arg(short, long)]
        file: Option<String>,
        resource_type: String,
        id: String,
    },
    Transaction {
        #[arg(short, long)]
        parallel: Option<usize>,
        #[arg(short, long)]
        file: Option<String>,
        #[arg(short, long)]
        output: Option<bool>,
    },
    Batch {
        #[arg(short, long)]
        file: Option<String>,
        #[arg(short, long)]
        output: Option<bool>,
    },

    HistorySystem {
        parameters: Option<String>,
    },

    HistoryType {
        resource_type: String,
        parameters: Option<String>,
    },

    HistoryInstance {
        resource_type: String,
        id: String,
        parameters: Option<String>,
    },

    SearchType {
        resource_type: String,
        parameters: Option<String>,
    },

    SearchSystem {
        parameters: Option<String>,
    },

    InvokeSystem {
        #[arg(short, long)]
        file: Option<String>,
        operation_name: String,
    },

    InvokeType {
        #[arg(short, long)]
        file: Option<String>,
        resource_type: String,
        operation_name: String,
    },

    Capabilities {},

    DeleteInstance {
        resource_type: String,
        id: String,
    },

    DeleteType {
        resource_type: String,
        parameters: Option<String>,
    },

    DeleteSystem {
        parameters: Option<String>,
    },

    InvokeInstance {
        #[arg(short, long)]
        file: Option<String>,
        resource_type: String,
        id: String,
        operation_name: String,
    },
}

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
                            "No active profile set. Please set an active profile using the config command."
                                .to_string(),
                        ));
                            };

                            let well_known_document = if let Some(well_known_doc) =
                                &current_state.well_known_document
                            {
                                well_known_doc.clone()
                            } else {
                                let res = reqwest::get(&active_profile.oidc_discovery_uri).await;
                                let res = res.map_err(|e| {
                                    OperationOutcomeError::error(
                                        IssueType::Exception(None),
                                        format!("Failed to fetch OIDC discovery document: {}", e),
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
                                        format!("Failed to parse OIDC discovery document: {}", e),
                                    )
                                })?;

                                current_state.well_known_document =
                                    Some(well_known_document.clone());
                                well_known_document
                            };

                            // Post for JWT Token
                            let params = [
                                ("grant_type", "client_credentials"),
                                ("client_id", &client_id),
                                ("client_secret", &client_secret),
                                ("scope", "openid system/*.*"),
                            ];

                            let res = reqwest::Client::new()
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

async fn read_from_file_or_stin<Type: FHIRJSONDeserializer>(
    file_path: &Option<String>,
) -> Result<Type, OperationOutcomeError> {
    if let Some(file_path) = file_path {
        let file_content = std::fs::read_to_string(file_path).map_err(|e| {
            OperationOutcomeError::error(
                IssueType::Exception(None),
                format!("Failed to read transaction file: {}", e),
            )
        })?;

        haste_fhir_serialization_json::from_str::<Type>(&file_content).map_err(|e| {
            OperationOutcomeError::error(
                IssueType::Exception(None),
                format!("Failed to parse transaction file: {}", e),
            )
        })
    } else {
        // Read from stdin
        let mut buffer = String::new();

        std::io::stdin().read_line(&mut buffer).map_err(|e| {
            OperationOutcomeError::error(
                IssueType::Exception(None),
                format!("Failed to read from stdin: {}", e),
            )
        })?;

        haste_fhir_serialization_json::from_str::<Type>(&buffer).map_err(|e| {
            OperationOutcomeError::error(
                IssueType::Exception(None),
                format!("Failed to parse transaction from stdin: {}", e),
            )
        })
    }
}

pub async fn api_commands(
    state: Arc<Mutex<CLIState>>,
    command: &ApiCommands,
) -> Result<(), OperationOutcomeError> {
    let http_state = config_to_fhir_http_state(state).await?;
    let fhir_client = Arc::new(FHIRHttpClient::<()>::new(http_state));
    match command {
        ApiCommands::Create {
            resource_type,
            file,
        } => {
            let resource_type = ResourceType::try_from(resource_type.as_str()).map_err(|e| {
                OperationOutcomeError::error(
                    IssueType::Invalid(None),
                    format!(
                        "'{}' is not a valid FHIR resource type: {}",
                        resource_type, e
                    ),
                )
            })?;

            let resource = read_from_file_or_stin::<Resource>(file).await?;

            let result = fhir_client.create((), resource_type, resource).await?;

            println!(
                "{}",
                haste_fhir_serialization_json::to_string(&result)
                    .expect("Failed to serialize response")
            );

            Ok(())
        }
        ApiCommands::Read {
            resource_type: _,
            id: _,
        } => todo!(),
        ApiCommands::Patch {
            resource_type,
            id,
            file,
        } => {
            let file_content = std::fs::read_to_string(file).map_err(|e| {
                OperationOutcomeError::error(
                    IssueType::Exception(None),
                    format!("Failed to read transaction file: {}", e),
                )
            })?;

            let patch = serde_json::from_str::<json_patch::Patch>(&file_content).map_err(|e| {
                OperationOutcomeError::error(
                    IssueType::Invalid(None),
                    format!("Failed to parse patch JSON: {}", e),
                )
            })?;

            let resource_type = ResourceType::try_from(resource_type.as_str()).map_err(|e| {
                OperationOutcomeError::error(
                    IssueType::Invalid(None),
                    format!(
                        "'{}' is not a valid FHIR resource type: {}",
                        resource_type, e
                    ),
                )
            })?;

            let result = fhir_client
                .patch((), resource_type, id.clone(), patch)
                .await?;

            println!(
                "{}",
                haste_fhir_serialization_json::to_string(&result)
                    .expect("Failed to serialize response")
            );

            Ok(())
        }
        ApiCommands::Update {
            resource_type,
            id,
            file,
        } => {
            let resource_type = ResourceType::try_from(resource_type.as_str()).map_err(|e| {
                OperationOutcomeError::error(
                    IssueType::Invalid(None),
                    format!(
                        "'{}' is not a valid FHIR resource type: {}",
                        resource_type, e
                    ),
                )
            })?;

            let resource = read_from_file_or_stin::<Resource>(file).await?;

            let result = fhir_client
                .update((), resource_type, id.clone(), resource)
                .await?;

            println!(
                "{}",
                haste_fhir_serialization_json::to_string(&result)
                    .expect("Failed to serialize response")
            );
            Ok(())
        }
        ApiCommands::Transaction {
            file,
            output,
            parallel,
        } => {
            let bundle = read_from_file_or_stin::<Bundle>(file).await?;

            let parallel = parallel.unwrap_or(1);

            let mut futures = tokio::task::JoinSet::new();

            for _ in 0..parallel {
                let client = fhir_client.clone();
                let bundle = bundle.clone();
                let res = async move { client.transaction((), bundle).await };
                futures.spawn(res);
            }

            let res = futures.join_all().await;

            for bundle_result in res {
                let bundle = bundle_result?;
                if let Some(true) = output {
                    println!(
                        "{}",
                        haste_fhir_serialization_json::to_string(&bundle)
                            .expect("Failed to serialize response")
                    );
                }
            }

            Ok(())
        }
        ApiCommands::VersionRead {
            resource_type,
            id,
            version_id,
        } => {
            let resource_type = ResourceType::try_from(resource_type.as_str()).map_err(|e| {
                OperationOutcomeError::error(
                    IssueType::Invalid(None),
                    format!(
                        "'{}' is not a valid FHIR resource type: {}",
                        resource_type, e
                    ),
                )
            })?;

            let result = fhir_client
                .vread((), resource_type, id.clone(), version_id.clone())
                .await?;

            println!(
                "{}",
                haste_fhir_serialization_json::to_string(&result)
                    .expect("Failed to serialize response")
            );

            Ok(())
        }
        ApiCommands::Batch { file, output } => {
            let bundle = read_from_file_or_stin::<Bundle>(file).await?;

            let result = fhir_client.batch((), bundle).await?;

            if let Some(true) = output {
                println!(
                    "{}",
                    haste_fhir_serialization_json::to_string(&result)
                        .expect("Failed to serialize response")
                );
            }

            Ok(())
        }
        ApiCommands::HistorySystem { parameters } => {
            let result = fhir_client
                .history_system(
                    (),
                    ParsedParameters::try_from(parameters.clone().unwrap_or_default().as_str())?,
                )
                .await?;

            println!(
                "{}",
                haste_fhir_serialization_json::to_string(&result)
                    .expect("Failed to serialize response")
            );

            Ok(())
        }
        ApiCommands::HistoryType {
            resource_type,
            parameters,
        } => {
            let resource_type = ResourceType::try_from(resource_type.as_str()).map_err(|e| {
                OperationOutcomeError::error(
                    IssueType::Invalid(None),
                    format!(
                        "'{}' is not a valid FHIR resource type: {}",
                        resource_type, e
                    ),
                )
            })?;

            let result = fhir_client
                .history_type(
                    (),
                    resource_type,
                    ParsedParameters::try_from(parameters.clone().unwrap_or_default().as_str())?,
                )
                .await?;

            println!(
                "{}",
                haste_fhir_serialization_json::to_string(&result)
                    .expect("Failed to serialize response")
            );

            Ok(())
        }
        ApiCommands::HistoryInstance {
            resource_type,
            id,
            parameters,
        } => {
            let resource_type = ResourceType::try_from(resource_type.as_str()).map_err(|e| {
                OperationOutcomeError::error(
                    IssueType::Invalid(None),
                    format!(
                        "'{}' is not a valid FHIR resource type: {}",
                        resource_type, e
                    ),
                )
            })?;

            let result = fhir_client
                .history_instance(
                    (),
                    resource_type,
                    id.clone(),
                    ParsedParameters::try_from(parameters.clone().unwrap_or_default().as_str())?,
                )
                .await?;

            println!(
                "{}",
                haste_fhir_serialization_json::to_string(&result)
                    .expect("Failed to serialize response")
            );

            Ok(())
        }
        ApiCommands::SearchType {
            resource_type,
            parameters,
        } => {
            let resource_type = ResourceType::try_from(resource_type.as_str()).map_err(|e| {
                OperationOutcomeError::error(
                    IssueType::Invalid(None),
                    format!(
                        "'{}' is not a valid FHIR resource type: {}",
                        resource_type, e
                    ),
                )
            })?;

            let result = fhir_client
                .search_type(
                    (),
                    resource_type,
                    ParsedParameters::try_from(parameters.clone().unwrap_or_default().as_str())?,
                )
                .await?;

            println!(
                "{}",
                haste_fhir_serialization_json::to_string(&result)
                    .expect("Failed to serialize response")
            );

            Ok(())
        }
        ApiCommands::SearchSystem { parameters } => {
            let result = fhir_client
                .search_system(
                    (),
                    ParsedParameters::try_from(parameters.clone().unwrap_or_default().as_str())?,
                )
                .await?;

            println!(
                "{}",
                haste_fhir_serialization_json::to_string(&result)
                    .expect("Failed to serialize response")
            );

            Ok(())
        }
        ApiCommands::InvokeSystem {
            operation_name,
            file,
        } => {
            let parameters = read_from_file_or_stin::<
                haste_fhir_model::r4::generated::resources::Parameters,
            >(file)
            .await?;

            let result = fhir_client
                .invoke_system((), operation_name.clone(), parameters)
                .await?;

            println!(
                "{}",
                haste_fhir_serialization_json::to_string(&result)
                    .expect("Failed to serialize response")
            );

            Ok(())
        }
        ApiCommands::InvokeType {
            resource_type,
            operation_name,
            file,
        } => {
            let resource_type = ResourceType::try_from(resource_type.as_str()).map_err(|e| {
                OperationOutcomeError::error(
                    IssueType::Invalid(None),
                    format!(
                        "'{}' is not a valid FHIR resource type: {}",
                        resource_type, e
                    ),
                )
            })?;

            let parameters = read_from_file_or_stin::<
                haste_fhir_model::r4::generated::resources::Parameters,
            >(file)
            .await?;

            let result = fhir_client
                .invoke_type((), resource_type, operation_name.clone(), parameters)
                .await?;

            println!(
                "{}",
                haste_fhir_serialization_json::to_string(&result)
                    .expect("Failed to serialize response")
            );

            Ok(())
        }
        ApiCommands::InvokeInstance {
            resource_type,
            id,
            operation_name,
            file,
        } => {
            let resource_type = ResourceType::try_from(resource_type.as_str()).map_err(|e| {
                OperationOutcomeError::error(
                    IssueType::Invalid(None),
                    format!(
                        "'{}' is not a valid FHIR resource type: {}",
                        resource_type, e
                    ),
                )
            })?;

            let parameters = read_from_file_or_stin::<
                haste_fhir_model::r4::generated::resources::Parameters,
            >(file)
            .await?;

            let result = fhir_client
                .invoke_instance(
                    (),
                    resource_type,
                    id.clone(),
                    operation_name.clone(),
                    parameters,
                )
                .await?;

            println!(
                "{}",
                haste_fhir_serialization_json::to_string(&result)
                    .expect("Failed to serialize response")
            );

            Ok(())
        }
        ApiCommands::Capabilities {} => {
            let result = fhir_client.capabilities(()).await?;

            println!(
                "{}",
                haste_fhir_serialization_json::to_string(&result)
                    .expect("Failed to serialize response")
            );

            Ok(())
        }
        ApiCommands::DeleteInstance { resource_type, id } => {
            let resource_type = ResourceType::try_from(resource_type.as_str()).map_err(|e| {
                OperationOutcomeError::error(
                    IssueType::Invalid(None),
                    format!(
                        "'{}' is not a valid FHIR resource type: {}",
                        resource_type, e
                    ),
                )
            })?;

            fhir_client
                .delete_instance((), resource_type.clone(), id.clone())
                .await?;

            println!(
                "Resource of type '{}' with ID '{}' deleted.",
                resource_type.as_ref(),
                id
            );

            Ok(())
        }
        ApiCommands::DeleteType {
            resource_type,
            parameters,
        } => {
            let resource_type = ResourceType::try_from(resource_type.as_str()).map_err(|e| {
                OperationOutcomeError::error(
                    IssueType::Invalid(None),
                    format!(
                        "'{}' is not a valid FHIR resource type: {}",
                        resource_type, e
                    ),
                )
            })?;

            let parsed_parameters =
                ParsedParameters::try_from(parameters.clone().unwrap_or_default().as_str())?;

            fhir_client
                .delete_type((), resource_type.clone(), parsed_parameters)
                .await?;

            println!(
                "Resources of type '{}' deleted based on provided parameters.",
                resource_type.as_ref()
            );

            Ok(())
        }
        ApiCommands::DeleteSystem { parameters } => {
            let parsed_parameters =
                ParsedParameters::try_from(parameters.clone().unwrap_or_default().as_str())?;

            fhir_client.delete_system((), parsed_parameters).await?;

            println!("Resources deleted based on provided system-level parameters.");

            Ok(())
        }
    }
}
