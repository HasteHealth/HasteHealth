use std::sync::Arc;

use clap::{Subcommand, ValueEnum};
use figment::{
    Figment,
    providers::{Env, Format as _, Toml},
};
use haste_fhir_model::r4::generated::terminology::IssueType;
use haste_fhir_operation_error::OperationOutcomeError;
use haste_jwt::claims::SubscriptionTier;
use haste_server::{config::ServerConfig, server};

#[derive(Clone, Debug, ValueEnum)]
pub(crate) enum UserSubscriptionChoice {
    Free,
    Professional,
    Team,
    Unlimited,
}

impl From<UserSubscriptionChoice> for SubscriptionTier {
    fn from(choice: UserSubscriptionChoice) -> Self {
        match choice {
            UserSubscriptionChoice::Free => SubscriptionTier::Free,
            UserSubscriptionChoice::Professional => SubscriptionTier::Professional,
            UserSubscriptionChoice::Team => SubscriptionTier::Team,
            UserSubscriptionChoice::Unlimited => SubscriptionTier::Unlimited,
        }
    }
}

#[derive(Subcommand, Debug)]
pub(crate) enum ServerCommands {
    Start {
        #[arg(short, long)]
        port: Option<u16>,
    },
}

pub(crate) async fn server(command: &ServerCommands) -> Result<(), OperationOutcomeError> {
    let config: ServerConfig = Figment::new()
        .merge(Toml::file("haste.toml"))
        .merge(Env::prefixed("HASTE_"))
        .extract()
        .map_err(|e| OperationOutcomeError::error(IssueType::exception(), e.to_string()))?;

    match &command {
        ServerCommands::Start { port } => {
            server::serve(Arc::new(config), port.unwrap_or(3000)).await
        }
    }
}
