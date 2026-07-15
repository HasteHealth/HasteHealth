use derivative::Derivative;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct ServerConfig {
    pub allow_artifact_mutations: bool,
    /// Used for JWT signing/verification.
    pub certification_dir: PathBuf,
    /// Main root where the FHIR server is hosted.
    pub api_uri: String,
    /// Where to redirect for the hardcoded admin app.
    pub admin_app_redirect_uri: String,

    pub fhir: FHIRConfig,
    pub repo: RepoConfig,
    pub search: SearchConfig,
    pub email: Option<EmailConfig>,
    pub rate_limits: RateLimitsConfig,
    pub max_request_body_size: usize,
    pub monitoring: MonitoringConfig,
    pub security: SecurityConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct SecurityConfig {
    pub mfa: MFAConfig,
    pub encryption: SecretProviderConfig,
    pub aes_key: Option<String>,
    pub certification_key: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct MFAConfig {
    pub max_credentials_per_user: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct MonitoringConfig {
    pub audit_enabled: bool,
    pub ip_source: IpSource,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SecretProviderConfig {
    Environment { prefix: Option<String> },
    GCP { project_id: String },
    AWS { region: String },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct FHIRConfig {
    /// Max delete limit for type-delete and system-delete operations.
    pub delete_limit: usize,
}

// Repo backend where the FHIR server stores its data/resources.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "backend", rename_all = "snake_case")]
pub enum RepoConfig {
    Postgres(PostgresConfig),
}

#[derive(Derivative, Clone, Deserialize, Serialize)]
#[derivative(Debug)]
pub struct PostgresConfig {
    #[derivative(Debug = "ignore")]
    pub database_url: String,
    pub max_connections: u32,
}

// Search backend where the FHIR server stores its search indices.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "backend", rename_all = "snake_case")]
pub enum SearchConfig {
    Elasticsearch(ElasticsearchConfig),
}

#[derive(Derivative, Clone, Deserialize, Serialize)]
#[derivative(Debug)]
pub struct ElasticsearchConfig {
    pub url: String,
    #[derivative(Debug = "ignore")]
    pub username: String,
    #[derivative(Debug = "ignore")]
    pub password: String,
}

#[derive(Derivative, Clone, Deserialize, Serialize)]
#[derivative(Debug)]
#[serde(tag = "backend", rename_all = "snake_case")]
pub enum EmailConfig {
    SendGrid {
        #[derivative(Debug = "ignore")]
        api_key: String,
        #[derivative(Debug = "ignore")]
        from_address: String,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct RateLimitsConfig {
    pub rate_limit_subscription_tiers: Option<[usize; 4]>,
    pub rate_limit_window_seconds: u64,
    pub rate_limit_operation_points: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum IpSource {
    #[default]
    ConnectInfo,
    CfConnectingIp,
    XRealIp,
}

impl Default for FHIRConfig {
    fn default() -> Self {
        Self { delete_limit: 100 }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            allow_artifact_mutations: false,
            certification_dir: PathBuf::from("certifications"),
            api_uri: "http://localhost:3000".into(),
            admin_app_redirect_uri: "http://*.localhost:3001".into(),
            fhir: FHIRConfig::default(),
            repo: RepoConfig::default(),
            search: SearchConfig::default(),
            email: None,
            max_request_body_size: 4 * 1024 * 1024,
            rate_limits: RateLimitsConfig::default(),
            monitoring: MonitoringConfig::default(),
            security: SecurityConfig::default(),
        }
    }
}
impl Default for RepoConfig {
    fn default() -> Self {
        RepoConfig::Postgres(PostgresConfig::default())
    }
}
impl Default for PostgresConfig {
    fn default() -> Self {
        Self {
            database_url: "postgresql://postgres:postgres@localhost:5432/haste_health".into(),
            max_connections: 10,
        }
    }
}
impl Default for SearchConfig {
    fn default() -> Self {
        SearchConfig::Elasticsearch(ElasticsearchConfig::default())
    }
}
impl Default for ElasticsearchConfig {
    fn default() -> Self {
        Self {
            url: "http://localhost:9200".into(),
            username: "elastic".into(),
            password: "elastic".into(),
        }
    }
}

impl Default for RateLimitsConfig {
    fn default() -> Self {
        Self {
            rate_limit_subscription_tiers: None,
            rate_limit_window_seconds: 60 * 60 * 24, // 1 day in seconds
            rate_limit_operation_points: 100,
        }
    }
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            audit_enabled: false,
            ip_source: IpSource::default(),
        }
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            mfa: MFAConfig::default(),
            encryption: SecretProviderConfig::default(),
            aes_key: None,
            certification_key: None,
        }
    }
}

impl Default for MFAConfig {
    fn default() -> Self {
        Self {
            max_credentials_per_user: 1,
        }
    }
}

impl Default for SecretProviderConfig {
    fn default() -> Self {
        Self::Environment {
            prefix: Some("HASTE_SECRET_".to_string()),
        }
    }
}
