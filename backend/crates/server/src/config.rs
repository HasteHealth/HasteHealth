use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
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
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct FHIRConfig {
    /// Max delete limit for type-delete and system-delete operations.
    pub delete_limit: u32,
}

// Repo backend where the FHIR server stores its data/resources.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "backend", rename_all = "snake_case")]
pub enum RepoConfig {
    Postgres(PostgresConfig),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PostgresConfig {
    pub database_url: String,
    pub max_connections: u32,
}

// Search backend where the FHIR server stores its search indices.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "backend", rename_all = "snake_case")]
pub enum SearchConfig {
    Elasticsearch(ElasticsearchConfig),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ElasticsearchConfig {
    pub url: String,
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "backend", rename_all = "snake_case")]
pub enum EmailConfig {
    SendGrid {
        api_key: String,
        from_address: String,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct RateLimitsConfig {
    pub max_request_body_size: usize,
    pub rate_limit_subscriptions: bool, // see note
    pub rate_limit_window_seconds: u64,
    pub rate_limit_operation_points: u32,
    pub ip_source: IpSource,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum IpSource {
    #[default]
    ConnectInfo,
    XForwardedFor,
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
            rate_limits: RateLimitsConfig::default(),
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
            max_request_body_size: 1_048_576,
            rate_limit_subscriptions: true,
            rate_limit_window_seconds: 60,
            rate_limit_operation_points: 100,
            ip_source: IpSource::default(),
        }
    }
}
