use crate::{elastic_search::search::clauses::namespace_parameter, query::QueryBuildError};
use haste_fhir_client::url::Parameter;
use haste_fhir_model::r4::generated::resources::SearchParameter;
use serde_json::json;

/// Matches any of `[system|]code`. `:not` is applied by the caller, outside
/// the nested query.
pub fn token(
    namespace: Option<&str>,
    parameter: &Parameter,
    search_param: &SearchParameter,
) -> Result<serde_json::Value, QueryBuildError> {
    let column_name = namespace_parameter(namespace, search_param);

    let params = parameter
        .value
        .iter()
        .map(|value| token_value(&column_name, value))
        .collect::<Result<Vec<serde_json::Value>, QueryBuildError>>()?;

    Ok(json!({
        "bool": {
            "should": params
        }
    }))
}

fn token_value(column_name: &str, value: &str) -> Result<serde_json::Value, QueryBuildError> {
    let field = |name: &str| format!("{column_name}.{name}");
    let matches = |name: &str, query: &str| json!({ "match": { field(name): { "query": query } } });

    let query = match value.split('|').collect::<Vec<_>>()[..] {
        // `code`: any system.
        [code] => matches("code", code),
        // `|`: any token.
        ["", ""] => json!({ "match_all": {} }),
        // `|code`: a code with no system.
        ["", code] => json!({
            "bool": {
                "filter": [matches("code", code)],
                "must_not": [{ "exists": { "field": field("system") } }]
            }
        }),
        // `system|`: any code in the system.
        [system, ""] => matches("system", system),
        [system, code] => json!({
            "bool": {
                "filter": [matches("code", code), matches("system", system)]
            }
        }),
        _ => return Err(QueryBuildError::InvalidParameterValue(value.to_string())),
    };

    Ok(json!({
        "nested": {
            "path": column_name,
            "query": query
        }
    }))
}
