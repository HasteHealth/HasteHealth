use oxidized_config::Config;
use oxidized_fhir_model::r4::generated::{
    resources::ClientApplication,
    terminology::{ClientapplicationGrantType, ClientapplicationResponseTypes},
    types::FHIRString,
};
use oxidized_repository::types::TenantId;

pub fn get_admin_app(config: &dyn Config) -> Option<ClientApplication> {
    let redirect_uri = config.get("ADMIN_APP_REDIRECT_URI");

    if let Ok(redirect_uri) = redirect_uri {
        Some(ClientApplication {
            id: Some("admin-app".to_string()),
            name: Box::new(FHIRString {
                value: Some("Admin Application".to_string()),
                ..Default::default()
            }),
            responseTypes: Box::new(ClientapplicationResponseTypes::Code(None)),
            scope: Some(Box::new(FHIRString {
                value: Some("offline_access openid email profile fhirUser user/*.*".to_string()),
                ..Default::default()
            })),
            grantType: vec![
                Box::new(ClientapplicationGrantType::Authorization_code(None)),
                Box::new(ClientapplicationGrantType::Refresh_token(None)),
            ],
            redirectUri: Some(vec![Box::new(FHIRString {
                value: Some(redirect_uri),
                ..Default::default()
            })]),
            ..Default::default()
        })
    } else {
        None
    }
}

#[allow(dead_code)]
// Return the Admin app redirect url for the current tenant.
pub fn redirect_url(config: &dyn Config, tenant_id: TenantId) -> Option<String> {
    let admin_app = get_admin_app(config);

    if let Some(app) = admin_app {
        app.redirectUri
            .as_ref()
            .and_then(|uris| uris.get(0))
            .and_then(|uri| uri.value.as_ref())
            .map(|uri| uri.replace("*", tenant_id.as_ref()))
    } else {
        None
    }
}
