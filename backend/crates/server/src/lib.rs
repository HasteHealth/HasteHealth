mod auth_n;
mod extract;
mod fhir_http;

pub mod fhir_client;
pub mod load_artifacts;
pub mod server;
pub mod services;
pub mod tenants;

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
    APIURL,
    // Where to redirect for hardcoded admin app.
    AdminAppRedirectURI,
    SendGridAPIKey,
}

impl From<ServerEnvironmentVariables> for String {
    fn from(value: ServerEnvironmentVariables) -> Self {
        match value {
            ServerEnvironmentVariables::SendGridAPIKey => "SENDGRID_API_KEY".to_string(),
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
            ServerEnvironmentVariables::APIURL => "API_URL".to_string(),
            ServerEnvironmentVariables::AdminAppRedirectURI => "ADMIN_APP_REDIRECT_URI".to_string(),
        }
    }
}
