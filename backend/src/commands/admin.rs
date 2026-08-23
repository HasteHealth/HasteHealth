use clap::{Subcommand, ValueEnum};
use figment::{
    Figment,
    providers::{Env, Format, Toml},
};
use haste_fhir_client::FHIRClient;
use haste_fhir_model::r4::generated::{
    resources::{
        AccessPolicyV2, AccessPolicyV2Target, Bundle, BundleEntry, BundleEntryRequest,
        ClientApplication, Resource,
    },
    terminology::{
        AccessPolicyv2Engine, BundleType, ClientapplicationGrantType,
        ClientapplicationResponseTypes, HttpVerb, IssueType, UserRole,
    },
    types::{FHIRString, FHIRUri, Reference},
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_search::SearchEngine;
use haste_jwt::{ProjectId, TenantId, claims::SubscriptionTier};
use haste_repository::admin::Migrate;
use haste_server::{
    config::ServerConfig,
    fhir_client::ServerCTX,
    load_artifacts::{self, reset_artifacts},
    services,
    tenants::{create_tenant, create_user},
};
use std::sync::Arc;

/// Subscription tier to assign a newly created tenant.
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

/// How a newly created OIDC client authenticates.
#[derive(Clone, Debug, ValueEnum, PartialEq, Eq)]
pub(crate) enum ClientGrantTypeChoice {
    /// A confidential (server-to-server) client authenticated with a client secret.
    ClientCredentials,
    /// A public client (no secret) a human logs into via the browser (authorization_code + PKCE).
    AuthorizationCode,
}

/// Manage OIDC ClientApplication resources.
#[derive(Subcommand, Debug)]
pub(crate) enum ClientCommands {
    /// Create a ClientApplication and, for client-credentials clients, an AccessPolicyV2
    /// granting it full access.
    Create {
        /// OIDC client ID to create.
        #[arg(short, long)]
        id: String,
        /// Required for --grant-type client-credentials. Ignored (and unset, making the
        /// client public) for --grant-type authorization-code.
        #[arg(short, long)]
        secret: Option<String>,
        /// Tenant to create the client in.
        #[arg(short, long)]
        tenant: String,
        /// Project to create the client in.
        #[arg(short, long)]
        project: String,
        /// OAuth grant type the client uses to authenticate.
        #[arg(long, value_enum, default_value = "client-credentials")]
        grant_type: ClientGrantTypeChoice,
        /// Loopback redirect URI(s) to allow, e.g. http://127.0.0.1:8976/callback.
        /// Required for --grant-type authorization-code.
        #[arg(long)]
        redirect_uri: Vec<String>,
        /// OAuth scope to grant the client. Defaults depend on --grant-type.
        #[arg(long)]
        scope: Option<String>,
    },
}

/// Server-side administrative operations (tenants, users, clients, migrations).
#[derive(Subcommand, Debug)]
pub(crate) enum AdminCommands {
    /// Manage tenants.
    Tenant {
        #[command(subcommand)]
        command: TenantCommands,
    },

    /// Manage users.
    User {
        #[command(subcommand)]
        command: UserCommands,
    },

    /// Manage OIDC ClientApplication resources.
    Client {
        #[command(subcommand)]
        command: ClientCommands,
    },

    /// Run database/search/artifact migrations.
    Migrate {
        #[command(subcommand)]
        command: MigrationCommands,
    },
}

/// Run database/search/artifact migrations.
#[derive(Subcommand, Debug)]
pub(crate) enum MigrationCommands {
    /// Load the built-in FHIR artifacts (StructureDefinitions, ValueSets, etc).
    Artifacts {},
    /// Reload the built-in FHIR artifacts from scratch, discarding local edits to them.
    ResetArtifacts {},
    /// Run pending repository (Postgres) migrations.
    Repo {},
    /// Run pending search index (ElasticSearch) migrations.
    Search {},
    /// Run all of the above: repo, then search, then artifacts.
    All,
}

/// Manage tenants.
#[derive(Subcommand, Debug)]
pub(crate) enum TenantCommands {
    /// Create a tenant and its owner user.
    Create {
        /// Tenant ID to create.
        #[arg(short, long)]
        id: String,
        /// Subscription tier to assign. Defaults to Free.
        #[arg(short, long)]
        subscription_tier: Option<UserSubscriptionChoice>,
        /// Email address for the tenant's owner user.
        #[arg(long)]
        owner_email: String,
        /// Password for the tenant's owner user.
        #[arg(long)]
        owner_password: String,
    },
}

/// Manage users.
#[derive(Subcommand, Debug)]
pub(crate) enum UserCommands {
    /// Create an admin user within a tenant.
    Create {
        /// Email address for the new user.
        #[arg(short, long)]
        email: String,
        /// Password for the new user.
        #[arg(short, long)]
        password: String,
        /// Tenant to create the user in.
        #[arg(short, long)]
        tenant: String,
    },
}

async fn migrate_repo(config: Arc<ServerConfig>) -> Result<(), OperationOutcomeError> {
    let services = services::create_services(config).await?;
    services.repo.migrate().await?;
    Ok(())
}

async fn migrate_search(config: Arc<ServerConfig>) -> Result<(), OperationOutcomeError> {
    let services = services::create_services(config).await?;
    services
        .search
        .migrate(&haste_repository::types::SupportedFHIRVersions::R4)
        .await?;
    Ok(())
}

async fn migrate_artifacts(config: Arc<ServerConfig>) -> Result<(), OperationOutcomeError> {
    let mut config = (*config).clone();
    config.allow_artifact_mutations = true;

    load_artifacts::load_artifacts(Arc::new(config)).await?;

    Ok(())
}

/// Runs the `admin` command group.
pub(crate) async fn admin(command: &AdminCommands) -> Result<(), OperationOutcomeError> {
    let config: Arc<ServerConfig> = Arc::new(
        Figment::new()
            .merge(Toml::file("haste.toml"))
            .merge(Env::prefixed("HASTE_"))
            .extract()
            .map_err(|e| OperationOutcomeError::error(IssueType::exception(), e.to_string()))?,
    );

    match &command {
        AdminCommands::Migrate { command } => match command {
            MigrationCommands::Artifacts {} => migrate_artifacts(config).await,
            MigrationCommands::ResetArtifacts {} => reset_artifacts(config).await,
            MigrationCommands::Repo {} => migrate_repo(config).await,
            MigrationCommands::Search {} => migrate_search(config).await,
            MigrationCommands::All => {
                migrate_repo(config.clone()).await?;
                migrate_search(config.clone()).await?;
                migrate_artifacts(config).await?;
                Ok(())
            }
        },
        AdminCommands::Tenant { command } => match command {
            TenantCommands::Create {
                id,
                subscription_tier,
                owner_email,
                owner_password,
            } => {
                let services = services::create_services(config).await?;
                let result = create_tenant(
                    services.as_ref(),
                    Some(id.clone()),
                    id,
                    &SubscriptionTier::from(
                        subscription_tier
                            .clone()
                            .unwrap_or(UserSubscriptionChoice::Free),
                    ),
                    haste_fhir_model::r4::generated::resources::User {
                        role: UserRole::owner(),
                        email: Some(Box::new(
                            haste_fhir_model::r4::generated::types::FHIRString {
                                value: Some(owner_email.clone()),
                                ..Default::default()
                            },
                        )),
                        ..Default::default()
                    },
                    Some(owner_password),
                )
                .await;

                if let Err(operation_outcome_error) = result.as_ref()
                    && let Some(issue) = operation_outcome_error.outcome().issue.first()
                    && issue.code == IssueType::duplicate()
                {
                    println!("Tenant with ID '{}' already exists.", id);
                    return Ok(());
                }

                result?;

                Ok(())
            }
        },
        AdminCommands::User { command } => match command {
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
                    haste_fhir_model::r4::generated::resources::User {
                        role: UserRole::admin(),
                        email: Some(Box::new(
                            haste_fhir_model::r4::generated::types::FHIRString {
                                value: Some(email.clone()),
                                ..Default::default()
                            },
                        )),
                        ..Default::default()
                    },
                    Some(password),
                )
                .await?;

                services.commit().await?;

                Ok(())
            }
        },
        AdminCommands::Client { command } => match command {
            ClientCommands::Create {
                tenant,
                project,
                id,
                secret,
                grant_type,
                redirect_uri,
                scope,
            } => {
                let client_app = match grant_type {
                    ClientGrantTypeChoice::ClientCredentials => {
                        let Some(secret) = secret else {
                            return Err(OperationOutcomeError::error(
                                IssueType::invalid(),
                                "--secret is required for --grant-type client-credentials"
                                    .to_string(),
                            ));
                        };

                        ClientApplication {
                            id: Some(id.clone()),
                            secret: Some(Box::new(FHIRString {
                                value: Some(secret.clone()),
                                ..Default::default()
                            })),
                            scope: Some(Box::new(FHIRString {
                                value: Some(
                                    scope.clone().unwrap_or("openid system/*.*".to_string()),
                                ),
                                ..Default::default()
                            })),
                            name: Box::new(FHIRString {
                                value: Some("CLI".to_string()),
                                ..Default::default()
                            }),
                            grantType: vec![ClientapplicationGrantType::client_credentials()],
                            responseTypes: ClientapplicationResponseTypes::token(),
                            ..Default::default()
                        }
                    }
                    ClientGrantTypeChoice::AuthorizationCode => {
                        if redirect_uri.is_empty() {
                            return Err(OperationOutcomeError::error(
                                IssueType::invalid(),
                                "At least one --redirect-uri is required for --grant-type authorization-code"
                                    .to_string(),
                            ));
                        }

                        ClientApplication {
                            id: Some(id.clone()),
                            secret: None,
                            scope: Some(Box::new(FHIRString {
                                value: Some(scope.clone().unwrap_or(
                                    "openid profile fhirUser offline_access user/*.*".to_string(),
                                )),
                                ..Default::default()
                            })),
                            name: Box::new(FHIRString {
                                value: Some("CLI".to_string()),
                                ..Default::default()
                            }),
                            grantType: vec![
                                ClientapplicationGrantType::authorization_code(),
                                ClientapplicationGrantType::refresh_token(),
                            ],
                            responseTypes: ClientapplicationResponseTypes::code(),
                            redirectUri: Some(
                                redirect_uri
                                    .iter()
                                    .map(|uri| FHIRString {
                                        value: Some(uri.clone()),
                                        ..Default::default()
                                    })
                                    .collect(),
                            ),
                            ..Default::default()
                        }
                    }
                };

                let services = services::create_services(config).await?;

                let ctx = Arc::new(ServerCTX::system(
                    TenantId::new(tenant.clone()),
                    ProjectId::new(project.clone()),
                    services.fhir_client.clone(),
                    services.rate_limit.clone(),
                ));

                let mut entries = Vec::with_capacity(2);

                // Authorization-code clients are used by humans and rely on whatever
                // access policy is attached to the authenticating user, so only
                // client-credentials clients get an access policy of their own.
                if *grant_type == ClientGrantTypeChoice::ClientCredentials {
                    entries.push(BundleEntry {
                        fullUrl: Some(Box::new(FHIRUri {
                            value: Some("access-policy".to_string()),
                            ..Default::default()
                        })),
                        request: Some(BundleEntryRequest {
                            method: HttpVerb::post(),
                            url: Box::new(FHIRUri {
                                value: Some("AccessPolicyV2".to_string()),
                                ..Default::default()
                            }),
                            ..Default::default()
                        }),
                        resource: Some(Box::new(Resource::AccessPolicyV2(AccessPolicyV2 {
                            name: Box::new(FHIRString {
                                value: Some("ADMIN".to_string()),
                                ..Default::default()
                            }),
                            engine: AccessPolicyv2Engine::full_access(),
                            target: Some(vec![AccessPolicyV2Target {
                                link: Box::new(Reference {
                                    reference: Some(Box::new(FHIRString {
                                        value: Some("client-app".to_string()),
                                        ..Default::default()
                                    })),
                                    ..Default::default()
                                }),
                            }]),
                            ..Default::default()
                        }))),
                        ..Default::default()
                    });
                }

                entries.push(BundleEntry {
                    fullUrl: Some(Box::new(FHIRUri {
                        value: Some("client-app".to_string()),
                        ..Default::default()
                    })),
                    request: Some(BundleEntryRequest {
                        method: HttpVerb::put(),
                        url: Box::new(FHIRUri {
                            value: Some(format!("ClientApplication/{}", id)),
                            ..Default::default()
                        }),
                        ..Default::default()
                    }),
                    resource: Some(Box::new(Resource::ClientApplication(client_app))),
                    ..Default::default()
                });

                let transaction_bundle = Bundle {
                    type_: BundleType::transaction(),
                    entry: Some(entries),
                    ..Default::default()
                };

                services
                    .fhir_client
                    .transaction(ctx, transaction_bundle)
                    .await?;

                Ok(())
            }
        },
    }
}
