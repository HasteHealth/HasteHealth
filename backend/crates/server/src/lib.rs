pub mod auth_n;
mod extract;
pub mod fhir_client;
mod fhir_http;
pub mod load_artifacts;
mod mcp;
mod middleware;
mod openapi;
mod route_path;
pub mod server;
pub mod services;
mod static_assets;
pub mod tenants;
mod ui;

pub enum ServerEnvironmentVariables {
    AllowArtifactMutations,
    // Used for JWT
    CertificationDir,
    // Main repo config
    DataBaseURL,
    // Search variable config.
    ElasticSearchURL,
    ElasticSearchUsername,
    ElasticSearchPassword,
    // Main root where the FHIR Server is hosted.
    APIURI,
    // Where to redirect for hardcoded admin app.
    AdminAppRedirectURI,
    // Email
    SendGridAPIKey,
    EmailFromAddress,
    // Data Limits
    MaxRequestBodySize,
    RateLimitSubscriptions,
    RateLimitWindowInSeconds,
    RateLimitOperationPoints,
    IpSource,
}

impl From<ServerEnvironmentVariables> for String {
    fn from(value: ServerEnvironmentVariables) -> Self {
        match value {
            ServerEnvironmentVariables::CertificationDir => "CERTIFICATION_DIR".to_string(),
            ServerEnvironmentVariables::AllowArtifactMutations => {
                "ALLOW_ARTIFACT_MUTATIONS".to_string()
            }
            ServerEnvironmentVariables::DataBaseURL => "DATABASE_URL".to_string(),
            ServerEnvironmentVariables::ElasticSearchURL => "ELASTICSEARCH_URL".to_string(),
            ServerEnvironmentVariables::ElasticSearchUsername => {
                "ELASTICSEARCH_USERNAME".to_string()
            }
            ServerEnvironmentVariables::ElasticSearchPassword => {
                "ELASTICSEARCH_PASSWORD".to_string()
            }
            ServerEnvironmentVariables::APIURI => "API_URI".to_string(),
            ServerEnvironmentVariables::AdminAppRedirectURI => "ADMIN_APP_REDIRECT_URI".to_string(),
            ServerEnvironmentVariables::SendGridAPIKey => "SG_API_KEY".to_string(),
            ServerEnvironmentVariables::EmailFromAddress => "EMAIL_FROM".to_string(),
            ServerEnvironmentVariables::MaxRequestBodySize => "MAX_REQUEST_BODY_SIZE".to_string(),
            ServerEnvironmentVariables::RateLimitSubscriptions => {
                "RATE_LIMIT_SUBSCRIPTIONS".to_string()
            }
            ServerEnvironmentVariables::RateLimitWindowInSeconds => {
                "RATE_LIMIT_WINDOW_IN_SECONDS".to_string()
            }
            ServerEnvironmentVariables::RateLimitOperationPoints => {
                "RATE_LIMIT_OPERATION_POINTS".to_string()
            }
            ServerEnvironmentVariables::IpSource => "IP_SOURCE".to_string(),
        }
    }
}
