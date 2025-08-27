use once_cell::sync::Lazy;
use oxidized_fhir_model::r4::types::Resource;
use oxidized_macro_loads::load_artifacts;

pub mod search_parameters;

static ALL_RESOURCES: &[&str] = load_artifacts!(
    "../artifacts/r4/hl7/minified"
    "../artifacts/r4/oxidized_health"
);

fn flatten_if_bundle(resource: Resource) -> Vec<Box<Resource>> {
    match resource {
        Resource::Bundle(bundle) => bundle
            .entry
            .unwrap_or(vec![])
            .into_iter()
            .flat_map(|e| e.resource)
            .collect::<Vec<_>>(),
        _ => vec![Box::new(resource)],
    }
}

fn load_resources(data_strings: &[&str]) -> Vec<Box<Resource>> {
    let mut resources = vec![];

    for data_str in data_strings.into_iter() {
        let resource = oxidized_fhir_serialization_json::from_str::<Resource>(data_str)
            .expect("Failed to parse search parameters JSON");
        resources.extend(flatten_if_bundle(resource));
    }

    resources
}

pub static ARTIFACT_RESOURCES: Lazy<Vec<Box<Resource>>> = Lazy::new(|| {
    let data_strings = ALL_RESOURCES;
    load_resources(data_strings)
});
