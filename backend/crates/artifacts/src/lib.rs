use once_cell::sync::Lazy;
use oxidized_fhir_model::r4::generated::resources::Resource;
use rust_embed::Embed;

pub mod search_parameters;

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

fn load_resources() -> Vec<Box<Resource>> {
    let mut resources = vec![];

    for path in EmbededResourceAssets::iter() {
        let data = EmbededResourceAssets::get(path.as_ref()).unwrap();
        let resource = oxidized_fhir_serialization_json::from_str::<Resource>(
            str::from_utf8(&data.data).unwrap(),
        )
        .expect("Failed to parse artifact parameters JSON");
        resources.extend(flatten_if_bundle(resource));
    }

    resources
}

#[derive(Embed)]
#[folder = "./artifacts/r4"]
#[include = "oxidized_health/**/*.json"]
#[include = "hl7/minified/**/*.json"]

struct EmbededResourceAssets;

pub static ARTIFACT_RESOURCES: Lazy<Vec<Box<Resource>>> = Lazy::new(|| load_resources());
