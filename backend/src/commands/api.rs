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
use haste_server::auth_n::oidc::routes::discovery::WellKnownDiscoveryDocument;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Subcommand, Debug)]
pub enum ApiCommands {
    Create {
        #[arg(short, long)]
        data: Option<String>,
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
        data: Option<String>,
        #[arg(short, long)]
        file: Option<String>,
        resource_type: String,
        id: String,
    },
    Update {
        #[arg(short, long)]
        data: Option<String>,
        #[arg(short, long)]
        file: Option<String>,
        resource_type: String,
        id: String,
    },
    Transaction {
        #[arg(short, long)]
        data: Option<String>,
        #[arg(short, long)]
        parallel: Option<usize>,
        #[arg(short, long)]
        file: Option<String>,
        #[arg(short, long)]
        output: Option<bool>,
    },
    Batch {
        #[arg(short, long)]
        data: Option<String>,
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
        data: Option<String>,
        #[arg(short, long)]
        file: Option<String>,
        operation_name: String,
    },

    InvokeType {
        #[arg(short, long)]
        data: Option<String>,
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
        data: Option<String>,
        #[arg(short, long)]
        file: Option<String>,
        resource_type: String,
        id: String,
        operation_name: String,
    },
}

async fn derive_resource_data_arg_file_arg_or_stdin<Type: FHIRJSONDeserializer>(
    data_arg: &Option<String>,
    file_path: &Option<String>,
) -> Result<Type, OperationOutcomeError> {
    if let Some(data) = data_arg {
        haste_fhir_serialization_json::from_str::<Type>(data).map_err(|e| {
            OperationOutcomeError::error(
                IssueType::Exception(None),
                format!("Failed to parse transaction data: {}", e),
            )
        })
    } else if let Some(file_path) = file_path {
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
    let fhir_client = crate::client::fhir_client(state).await?;

    match command {
        ApiCommands::Create {
            data,
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

            let resource =
                derive_resource_data_arg_file_arg_or_stdin::<Resource>(data, file).await?;

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
            data,
            file,
        } => {
            let patches = if let Some(file) = file {
                let file_content = std::fs::read_to_string(file).map_err(|e| {
                    OperationOutcomeError::error(
                        IssueType::Exception(None),
                        format!("Failed to read transaction file: {}", e),
                    )
                })?;

                serde_json::from_str::<json_patch::Patch>(&file_content).map_err(|e| {
                    OperationOutcomeError::error(
                        IssueType::Invalid(None),
                        format!("Failed to parse patch JSON: {}", e),
                    )
                })?
            } else if let Some(data) = data {
                serde_json::from_str::<json_patch::Patch>(&data).map_err(|e| {
                    OperationOutcomeError::error(
                        IssueType::Invalid(None),
                        format!("Failed to parse patch JSON: {}", e),
                    )
                })?
            } else {
                return Err(OperationOutcomeError::error(
                    IssueType::Invalid(None),
                    "Either --data or --file must be provided for patch operation.".to_string(),
                ));
            };

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
                .patch((), resource_type, id.clone(), patches)
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
            data,
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

            let resource =
                derive_resource_data_arg_file_arg_or_stdin::<Resource>(data, file).await?;

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
            data,
            file,
            output,
            parallel,
        } => {
            let bundle = derive_resource_data_arg_file_arg_or_stdin::<Bundle>(data, file).await?;

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
        ApiCommands::Batch { data, file, output } => {
            let bundle = derive_resource_data_arg_file_arg_or_stdin::<Bundle>(data, file).await?;

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
            data,
        } => {
            let parameters = derive_resource_data_arg_file_arg_or_stdin::<
                haste_fhir_model::r4::generated::resources::Parameters,
            >(data, file)
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
            data,
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

            let parameters = derive_resource_data_arg_file_arg_or_stdin::<
                haste_fhir_model::r4::generated::resources::Parameters,
            >(data, file)
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
            data,
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

            let parameters = derive_resource_data_arg_file_arg_or_stdin::<
                haste_fhir_model::r4::generated::resources::Parameters,
            >(data, file)
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
