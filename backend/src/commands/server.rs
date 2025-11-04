use clap::Subcommand;
use oxidized_config::{Config, get_config};
use oxidized_fhir_client::FHIRClient;
use oxidized_fhir_model::r4::generated::{
    resources::{Resource, ResourceType, User},
    terminology::UserRole,
    types::FHIRString,
};
use oxidized_fhir_operation_error::OperationOutcomeError;
use oxidized_fhir_search::SearchEngine;
use oxidized_jwt::{ProjectId, TenantId};
use oxidized_repository::admin::Migrate;
use oxidized_server::{
    ServerEnvironmentVariables,
    auth_n::oidc::utilities::set_user_password,
    fhir_client::ServerCTX,
    load_artifacts, server, services,
    tenants::{SubscriptionTier, create_tenant},
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
    RepoSchema {},
    SearchSchema {},
    All,
}

#[derive(Subcommand, Debug)]
pub enum TenantCommands {
    Create {
        #[arg(short, long)]
        id: String,
        #[arg(short, long)]
        subscription_tier: Option<SubscriptionTier>,
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
        .migrate(&oxidized_repository::types::SupportedFHIRVersions::R4)
        .await?;
    Ok(())
}

pub async fn server(command: &ServerCommands) -> Result<(), OperationOutcomeError> {
    let config = get_config::<ServerEnvironmentVariables>("environment".into());

    match &command {
        ServerCommands::Start { port } => server::serve(port.unwrap_or(3000)).await,
        ServerCommands::Migrate { command } => match command {
            MigrationCommands::Artifacts {} => {
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
            MigrationCommands::RepoSchema {} => migrate_repo(config).await,
            MigrationCommands::SearchSchema {} => migrate_search(config).await,
            MigrationCommands::All => {
                migrate_repo(config.clone()).await?;
                migrate_search(config).await?;
                Ok(())
            }
        },
        ServerCommands::Tenant { command } => match command {
            TenantCommands::Create {
                id,
                subscription_tier,
            } => {
                let services = services::create_services(config).await?;
                create_tenant(
                    services,
                    Some(id.clone()),
                    id,
                    &subscription_tier.clone().unwrap_or(SubscriptionTier::Free),
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

                let ctx = Arc::new(ServerCTX::system(
                    tenant.clone(),
                    ProjectId::System,
                    services.fhir_client.clone(),
                ));

                let user = services
                    .fhir_client
                    .create(
                        ctx,
                        ResourceType::User,
                        Resource::User(User {
                            role: Box::new(UserRole::Admin(None)),
                            email: Box::new(FHIRString {
                                value: Some(email.clone()),
                                ..Default::default()
                            }),
                            ..Default::default()
                        }),
                    )
                    .await?;

                let user = match user {
                    Resource::User(user) => user,
                    _ => panic!("Created resource is not a User"),
                };

                let user_id = user.id.clone().unwrap();

                set_user_password(&*services.repo, &tenant, email, &user_id, password).await?;

                services.commit().await?;

                Ok(())
            }
        },
    }
}
