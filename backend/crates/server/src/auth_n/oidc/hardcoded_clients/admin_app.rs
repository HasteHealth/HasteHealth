use crate::config::ServerConfig;
use haste_fhir_model::r4::generated::{
    resources::ClientApplication,
    terminology::{ClientapplicationGrantType, ClientapplicationResponseTypes},
    types::FHIRString,
};
use haste_jwt::{ProjectId, TenantId};

pub fn get_admin_app(config: &ServerConfig) -> Option<ClientApplication> {
    let redirect_uri = &config.admin_app_redirect_uri;

    Some(ClientApplication {
        id: Some("admin-app".to_string()),
        name: Box::new(FHIRString {
            value: Some("Admin Application".to_string()),
            ..Default::default()
        }),
        responseTypes: ClientapplicationResponseTypes::CODE,
        scope: Some(Box::new(FHIRString {
            value: Some("offline_access openid email profile fhirUser system/*.*".to_string()),
            ..Default::default()
        })),
        grantType: vec![
            ClientapplicationGrantType::AUTHORIZATION_CODE,
            ClientapplicationGrantType::REFRESH_TOKEN,
        ],
        redirectUri: Some(vec![Box::new(FHIRString {
            value: Some(redirect_uri.clone()),
            ..Default::default()
        })]),
        ..Default::default()
    })
}

// Return the Admin app redirect url for the current tenant.
pub fn redirect_url(
    config: &ServerConfig,
    tenant_id: &TenantId,
    project_id: &ProjectId,
) -> Option<String> {
    let admin_app = get_admin_app(config);

    if let Some(app) = admin_app {
        app.redirectUri
            .as_ref()
            .and_then(|uris| uris.get(0))
            .and_then(|uri| uri.value.as_ref())
            .map(|uri| {
                uri.replace(
                    "*",
                    &(tenant_id.as_ref().to_string() + "_" + project_id.as_ref()),
                )
            })
    } else {
        None
    }
}
