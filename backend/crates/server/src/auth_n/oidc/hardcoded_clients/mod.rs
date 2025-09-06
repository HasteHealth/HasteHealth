use oxidized_config::Config;
use oxidized_fhir_model::r4::generated::resources::ClientApplication;

pub mod admin_app;

#[allow(dead_code)]
pub fn get_hardcoded_clients(config: &Box<dyn Config>) -> Vec<ClientApplication> {
    let mut hardcoded_apps = vec![];

    if let Some(app) = admin_app::get_admin_app(config) {
        hardcoded_apps.push(app);
    }

    hardcoded_apps
}
