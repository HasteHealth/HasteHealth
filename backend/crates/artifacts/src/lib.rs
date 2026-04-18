use haste_fhir_model::r4::generated::resources::{Resource, SearchParameter};
use rust_embed::Embed;
use std::sync::LazyLock;

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
        let resource = haste_fhir_serialization_json::from_str::<Resource>(
            str::from_utf8(&data.data).unwrap(),
        )
        .expect("Failed to parse artifact parameters JSON");
        resources.extend(flatten_if_bundle(resource));
    }

    resources
}

#[derive(Embed)]
#[folder = "./artifacts/r4"]
#[include = "haste_health/**/*.json"]
#[include = "hl7/minified/**/*.json"]
#[include = "us-core/**/*.json"]

struct EmbededResourceAssets;

pub static ARTIFACT_RESOURCES: LazyLock<Vec<Box<Resource>>> = LazyLock::new(|| load_resources());

#[derive(Embed)]
#[folder = "./artifacts/r4"]
#[include = "haste_health/search_parameter/*.json"]
#[include = "hl7/minified/search-parameters.min.json"]

struct EmbededSearchParameterAssets;

/// System level Search Parameters. These are used for all tenants and projects and are loaded from embedded assets at startup.
pub static R4_SEARCH_PARAMETERS: LazyLock<Vec<Box<SearchParameter>>> = LazyLock::new(|| {
    let mut search_parameters = vec![];

    for path in EmbededSearchParameterAssets::iter() {
        let data = EmbededSearchParameterAssets::get(path.as_ref()).unwrap();
        let bundle = haste_fhir_serialization_json::from_str::<Resource>(
            std::str::from_utf8(&data.data).unwrap(),
        )
        .expect("Failed to parse search parameters JSON");

        search_parameters.extend(flatten_if_bundle(bundle).into_iter().filter_map(|r| {
            if let Resource::SearchParameter(param) = *r {
                Some(Box::new(param))
            } else {
                None
            }
        }));
    }

    search_parameters
});
