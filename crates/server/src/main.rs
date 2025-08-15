use std::sync::Arc;

use clap::{Parser, Subcommand};
use oxidized_fhir_operation_error::OperationOutcomeError;
use oxidized_server::server;

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
        username: String,
        #[arg(short, long)]
        password: String,
        #[arg(short, long)]
        tenant: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), OperationOutcomeError> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Start { port } => {
            let server = server().await?;
            // run our app with hyper, listening globally on port 3000
            let addr = format!("0.0.0.0:{}", port.unwrap_or(3000));
            let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

            tracing::info!("Server started");
            axum::serve(listener, server).await.unwrap();

            Ok(())
        }
        Commands::Tenant { command } => match command {
            TenantCommands::Create {
                id,
                subscription_tier,
            } => {
                println!(
                    "Creating tenant with id: {}, subscription tier: {:?}",
                    id, subscription_tier
                );

                todo!();
                // Ok(())
            }
        },
        Commands::User { command } => match command {
            UserCommands::Create {
                username,
                password,
                tenant,
            } => {
                println!(
                    "Creating user with username: {}, password: {}, tenant: {}",
                    username, password, tenant
                );

                todo!();
                // Ok(())
            }
        },
    }
}
