use std::collections::HashMap;

use once_cell::sync::Lazy;
use oxidized_fhir_model::r4::types::{Bundle, Resource, SearchParameter};

static SEARCH_PARAMETERS_STR: &str =
    include_str!("../../artifacts/r4/hl7/search-parameters.min.json");

pub static R4_SEARCH_PARAMETERS: Lazy<HashMap<String, SearchParameter>> = Lazy::new(|| {
    let bundle = oxidized_fhir_serialization_json::from_str::<Bundle>(SEARCH_PARAMETERS_STR)
        .expect("Failed to parse search parameters JSON");

    bundle
        .entry
        .unwrap_or(vec![])
        .into_iter()
        .flat_map(|e| e.resource)
        .filter_map(|resource| match *resource {
            Resource::SearchParameter(search_param) => {
                Some((search_param.id.clone().unwrap(), search_param))
            }
            _ => None,
        })
        .collect::<HashMap<String, SearchParameter>>()
});
