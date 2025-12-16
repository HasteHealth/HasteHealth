use clap::Subcommand;
use haste_config::{Config, get_config};
use haste_fhir_model::r4::generated::terminology::UserRole;
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_search::SearchEngine;
use haste_jwt::TenantId;
use haste_repository::admin::Migrate;
use haste_server::{
    ServerEnvironmentVariables, load_artifacts, server, services,
    tenants::{SubscriptionTier, create_tenant, create_user},
};
use std::sync::Arc;

#[derive(Subcommand, Debug)]
pub enum ServerCommands {
    Start {
        #[arg(short, long)]
        port: Option<u16>,
    },

    Tenant {
        #[command(subcommand)]
        command: TenantCommands,
    },

    User {
        #[command(subcommand)]
        command: UserCommands,
    },

    Migrate {
        #[command(subcommand)]
        command: MigrationCommands,
    },
}

#[derive(Subcommand, Debug)]
pub enum MigrationCommands {
    Artifacts {},
    Repo {},
    Search {},
    All,
}

#[derive(Subcommand, Debug)]
pub enum TenantCommands {
    Create {
        #[arg(short, long)]
        id: String,
        #[arg(short, long)]
        subscription_tier: Option<SubscriptionTier>,
        #[arg(long)]
        owner_email: String,
        #[arg(long)]
        owner_password: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum UserCommands {
    Create {
        #[arg(short, long)]
        email: String,
        #[arg(short, long)]
        password: String,
        #[arg(short, long)]
        tenant: String,
    },
}

async fn migrate_repo(
    config: Arc<dyn Config<ServerEnvironmentVariables>>,
) -> Result<(), OperationOutcomeError> {
    let services = services::create_services(config).await?;
    services.repo.migrate().await?;
    Ok(())
}

async fn migrate_search(
    config: Arc<dyn Config<ServerEnvironmentVariables>>,
) -> Result<(), OperationOutcomeError> {
    let services = services::create_services(config).await?;
    services
        .search
        .migrate(&haste_repository::types::SupportedFHIRVersions::R4)
        .await?;
    Ok(())
}

async fn migrate_artifacts(
    config: Arc<dyn Config<ServerEnvironmentVariables>>,
) -> Result<(), OperationOutcomeError> {
    let initial = config
        .get(ServerEnvironmentVariables::AllowArtifactMutations)
        .unwrap_or("false".to_string());
    config.set(
        ServerEnvironmentVariables::AllowArtifactMutations,
        "true".to_string(),
    )?;
    load_artifacts::load_artifacts(config.clone()).await?;
    config.set(ServerEnvironmentVariables::AllowArtifactMutations, initial)?;
    Ok(())
}

pub async fn server(command: &ServerCommands) -> Result<(), OperationOutcomeError> {
    let config = get_config::<ServerEnvironmentVariables>("environment".into());

    match &command {
        ServerCommands::Start { port } => server::serve(port.unwrap_or(3000)).await,
        ServerCommands::Migrate { command } => match command {
            MigrationCommands::Artifacts {} => migrate_artifacts(config).await,
            MigrationCommands::Repo {} => migrate_repo(config).await,
            MigrationCommands::Search {} => migrate_search(config).await,
            MigrationCommands::All => {
                migrate_repo(config.clone()).await?;
                migrate_search(config.clone()).await?;
                migrate_artifacts(config).await?;
                Ok(())
            }
        },
        ServerCommands::Tenant { command } => match command {
            TenantCommands::Create {
                id,
                subscription_tier,
                owner_email,
                owner_password,
            } => {
                let services = services::create_services(config).await?;
                create_tenant(
                    services.as_ref(),
                    Some(id.clone()),
                    id,
                    &subscription_tier.clone().unwrap_or(SubscriptionTier::Free),
                    owner_email,
                    Some(owner_password),
                )
                .await?;

                Ok(())
            }
        },
        ServerCommands::User { command } => match command {
            UserCommands::Create {
                email,
                password,
                tenant,
            } => {
                let services = services::create_services(config)
                    .await?
                    .transaction()
                    .await?;

                let tenant = TenantId::new(tenant.clone());
                create_user(
                    &services,
                    &tenant,
                    email,
                    Some(password),
                    UserRole::Admin(None),
                )
                .await?;

                services.commit().await?;

                Ok(())
            }
        },
    }
}
