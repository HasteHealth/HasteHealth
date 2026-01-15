use clap::{Subcommand, ValueEnum};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_jwt::claims::SubscriptionTier;
use haste_server::server;

#[derive(Clone, Debug, ValueEnum)]
pub enum UserSubscriptionChoice {
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
pub enum ServerCommands {
    Start {
        #[arg(short, long)]
        port: Option<u16>,
    },
}

pub async fn server(command: &ServerCommands) -> Result<(), OperationOutcomeError> {
    match &command {
        ServerCommands::Start { port } => server::serve(port.unwrap_or(3000)).await,
    }
}
