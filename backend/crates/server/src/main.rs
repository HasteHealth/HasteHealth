use axum::{Router, ServiceExt, body::Body};
use clap::{Parser, Subcommand};
use oxidized_config::{Config, get_config};
use oxidized_fhir_operation_error::OperationOutcomeError;
use oxidized_fhir_search::SearchEngine;
use oxidized_repository::{
    admin::TenantAuthAdmin,
    types::{
        TenantId,
        tenant::CreateTenant,
        user::{AuthMethod, CreateUser, UserRole},
    },
};
use oxidized_server::{
    load_artifacts, server,
    services::{self, get_pool},
};

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
        subscription_tier: Option<String>,
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

async fn migrate_repo(config: &dyn Config) -> Result<(), OperationOutcomeError> {
    sqlx::migrate!("./migrations")
        .run(get_pool(config).await)
        .await
        .unwrap();
    Ok(())
}

async fn migrate_search(config: Box<dyn Config>) -> Result<(), OperationOutcomeError> {
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

    let config = get_config("environment".into());

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
                load_artifacts::load_artifacts().await?;
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
                services
                    .repo
                    .create(
                        &TenantId::System,
                        CreateTenant {
                            id: Some(TenantId::new(id.clone())),
                            subscription_tier: subscription_tier.clone(),
                        },
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
                let services = services::create_services(config).await?;
                services
                    .repo
                    .create(
                        &TenantId::new(tenant.clone()),
                        CreateUser {
                            role: UserRole::Admin,
                            email: email.clone(),
                            password: Some(password.clone()),
                            provider_id: None,
                            method: AuthMethod::EmailPassword,
                        },
                    )
                    .await?;

                Ok(())
            }
        },
    }
}
