use axum::{Router, ServiceExt, body::Body};
use clap::{Parser, Subcommand};
use oxidized_config::{Config, get_config};
use oxidized_fhir_client::FHIRClient;
use oxidized_fhir_model::r4::generated::{
    resources::{Resource, ResourceType, User},
    terminology::{IssueType, UserRole},
    types::FHIRString,
};
use oxidized_fhir_operation_error::OperationOutcomeError;
use oxidized_fhir_search::SearchEngine;
use oxidized_repository::{
    Repository,
    admin::TenantAuthAdmin,
    types::{
        ProjectId, TenantId,
        user::{CreateUser, UpdateUser},
    },
};
use oxidized_server::{
    ServerEnvironmentVariables,
    fhir_client::ServerCTX,
    load_artifacts, server,
    services::{self, get_pool},
    tenants::{SubscriptionTier, create_tenant},
};
use std::sync::Arc;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
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
enum MigrationCommands {
    Artifacts {},
    RepoSchema {},
    SearchSchema {},
    All,
}

#[derive(Subcommand, Debug)]
enum TenantCommands {
    Create {
        #[arg(short, long)]
        id: String,
        #[arg(short, long)]
        subscription_tier: Option<SubscriptionTier>,
    },
}

#[derive(Subcommand, Debug)]
enum UserCommands {
    Create {
        #[arg(short, long)]
        email: String,
        #[arg(short, long)]
        password: String,
        #[arg(short, long)]
        tenant: String,
    },
}

async fn set_user_password<Repo: Repository>(
    repo: &Repo,
    tenant: &TenantId,
    user_email: &str,
    user_id: &str,
    password: &str,
) -> Result<(), OperationOutcomeError> {
    // In a real implementation, you would hash the password here
    let password_strength = zxcvbn::zxcvbn(password, &[user_email]);

    if u8::from(password_strength.score()) < 3 {
        let feedback = password_strength
            .feedback()
            .map(|f| format!("{}", f))
            .unwrap_or_default();

        return Err(OperationOutcomeError::fatal(
            IssueType::Security(None),
            feedback,
        ));
    }

    TenantAuthAdmin::<CreateUser, _, _, _, String>::update(
        repo,
        &tenant,
        UpdateUser {
            id: user_id.to_string(),
            password: Some(password.to_string()),
            email: None,
            role: None,
            method: None,
            provider_id: None,
        },
    )
    .await?;

    Ok(())
}

async fn migrate_repo(
    config: &dyn Config<ServerEnvironmentVariables>,
) -> Result<(), OperationOutcomeError> {
    sqlx::migrate!("./migrations")
        .run(get_pool(config).await)
        .await
        .unwrap();
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

#[tokio::main]
async fn main() -> Result<(), OperationOutcomeError> {
    let cli = Cli::parse();

    let config = get_config::<ServerEnvironmentVariables>("environment".into());

    match &cli.command {
        Commands::Start { port } => {
            let server = server::server().await?;
            // run our app with hyper, listening globally on port 3000
            let addr = format!("0.0.0.0:{}", port.unwrap_or(3000));
            let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

            tracing::info!("Server started");
            axum::serve(
                listener,
                <tower_http::normalize_path::NormalizePath<Router> as ServiceExt<
                    axum::http::Request<Body>,
                >>::into_make_service(server),
            )
            .await
            .unwrap();

            Ok(())
        }
        Commands::Migrate { command } => match command {
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
            MigrationCommands::RepoSchema {} => migrate_repo(config.as_ref()).await,
            MigrationCommands::SearchSchema {} => migrate_search(config).await,
            MigrationCommands::All => {
                migrate_repo(config.as_ref()).await?;
                migrate_search(config).await?;
                Ok(())
            }
        },
        Commands::Tenant { command } => match command {
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
        Commands::User { command } => match command {
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

                let ctx = Arc::new(ServerCTX::system(tenant.clone(), ProjectId::System));

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
