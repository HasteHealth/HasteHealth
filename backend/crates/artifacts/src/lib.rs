use haste_fhir_model::r4::generated::resources::{Resource, SearchParameter};
use rust_embed::Embed;
use std::{collections::HashMap, sync::LazyLock};

fn flatten_if_bundle(resource: Resource) -> Vec<Resource> {
    match resource {
        Resource::Bundle(bundle) => bundle
            .entry
            .unwrap_or_default()
            .into_iter()
            .filter_map(|e| e.resource.map(|r| *r))
            .collect(),
        _ => vec![resource],
    }
}

fn load_resources() -> Vec<Resource> {
    let mut resources = HashMap::new();

    for path in EmbededResourceAssets::iter() {
        let data = EmbededResourceAssets::get(path.as_ref()).unwrap();
        let resource = serde_json::from_str::<Resource>(str::from_utf8(&data.data).unwrap())
            .expect("Failed to parse artifact parameters JSON");

        for r in flatten_if_bundle(resource) {
            let resource_type = r.resource_type();
            let id = r.id().clone().unwrap_or_else(|| {
                panic!("Resource in '{}' does not have an ID", path.as_ref());
            });

            let key = (resource_type, id);

            if resources.contains_key(&key) {
                println!(
                    "Duplicate resource ID '{}' '{}' found in '{}'",
                    key.0.as_ref(),
                    key.1,
                    path.as_ref()
                );
            }

            resources.insert(key, r);
        }
    }

    resources.into_values().collect()
}

#[derive(Embed)]
#[folder = "./artifacts"]
#[include = "r4/haste_health/**/*.json"]
#[include = "r4/hl7/minified/**/*.json"]
#[include = "universal/**/*.json"]
#[include = "r4/r4-to-r5-subscription-backport/**/*.json"]
struct EmbededResourceAssets;

pub static ARTIFACT_RESOURCES: LazyLock<Vec<Resource>> = LazyLock::new(load_resources);

#[derive(Embed)]
#[folder = "./artifacts/r4"]
#[include = "haste_health/search_parameter/*.json"]
#[include = "hl7/minified/search-parameters.min.json"]
struct EmbededSearchParameterAssets;

/// System level Search Parameters. These are used for all tenants and projects and are loaded from embedded assets at startup.
pub static R4_SEARCH_PARAMETERS: LazyLock<Vec<SearchParameter>> =
    LazyLock::new(|| {
        let mut search_parameters = Vec::new();

        for path in EmbededSearchParameterAssets::iter() {
            let data = EmbededSearchParameterAssets::get(path.as_ref()).unwrap();

            let bundle = serde_json::from_str::<Resource>(std::str::from_utf8(&data.data).unwrap())
                .expect("Failed to parse search parameters JSON");

            search_parameters.extend(flatten_if_bundle(bundle).into_iter().filter_map(
                |resource| match resource {
                    Resource::SearchParameter(param) => Some(param),
                    _ => None,
                },
            ));
        }

        search_parameters
    });
