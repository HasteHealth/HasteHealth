use axum::extract::OriginalUri;
use axum_extra::routing::TypedPath;
use oxidized_fhir_model::r4::generated::terminology::IssueType;
use oxidized_fhir_operation_error::OperationOutcomeError;
use serde::Deserialize;
use url::Url;

#[derive(TypedPath, Deserialize)]
#[typed_path("/federated/{identity_provider_id}/callback")]
pub struct FederatedInitiate {
    pub identity_provider_id: String,
}

pub fn federated_callback_url(
    api_url_string: &str,
    uri: &OriginalUri,
    idp_id: &str,
    replace_path: &str,
) -> Result<String, OperationOutcomeError> {
    let Ok(api_url) = Url::parse(&api_url_string) else {
        return Err(OperationOutcomeError::error(
            IssueType::Exception(None),
            "Invalid API_URL format".to_string(),
        ));
    };

    let path = uri.path().to_string().replace(
        replace_path,
        &FederatedInitiate {
            identity_provider_id: idp_id.to_string(),
        }
        .to_string(),
    );

    Ok(api_url.join(&path).unwrap().to_string())
}
