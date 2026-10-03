use crate::config::ServerConfig;
use haste_fhir_model::r4::generated::{
    resources::ClientApplication,
    terminology::{ClientapplicationGrantType, ClientapplicationResponseTypes},
    types::FHIRString,
};
use haste_jwt::{ProjectId, TenantId};
use haste_repository::utilities::HOSTNAME_ID_SEPARATOR;

pub fn get_admin_app(config: &ServerConfig) -> Option<ClientApplication> {
    let redirect_uri = &config.admin_app_redirect_uri;

    Some(ClientApplication {
        id: Some("admin-app".to_string()),
        name: Box::new(FHIRString {
            value: Some("Admin Application".to_string()),
            ..Default::default()
        }),
        responseTypes: ClientapplicationResponseTypes::code(),
        scope: Some(Box::new(FHIRString {
            value: Some("offline_access openid email profile fhirUser system/*.*".to_string()),
            ..Default::default()
        })),
        grantType: vec![
            ClientapplicationGrantType::authorization_code(),
            ClientapplicationGrantType::refresh_token(),
        ],
        redirectUri: Some(vec![FHIRString {
            value: Some(redirect_uri.clone()),
            ..Default::default()
        }]),
        ..Default::default()
    })
}

/// The admin app URL for a project: the redirect uri with `*` replaced by the
/// `{tenant}--{project}` subdomain.
pub fn redirect_url(
    config: &ServerConfig,
    tenant_id: &TenantId,
    project_id: &ProjectId,
) -> Option<String> {
    let subdomain = format!(
        "{}{HOSTNAME_ID_SEPARATOR}{}",
        tenant_id.as_ref(),
        project_id.as_ref()
    );

    get_admin_app(config)?
        .redirectUri?
        .first()?
        .value
        .as_ref()
        .map(|uri| uri.replace('*', &subdomain))
}
