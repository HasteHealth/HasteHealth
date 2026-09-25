use crate::{
    elastic_search::search::clauses::namespace_parameter, indexing_conversion::get_decimal_range,
    query::QueryBuildError, search_ranges::approximate_decimal,
};
use haste_fhir_client::url::{Parameter, parse_prefix};
use haste_fhir_model::r4::generated::resources::SearchParameter;
use serde_json::json;

/// A FHIR number has implicit precision, so equality is range containment.
pub fn number(
    namespace: Option<&str>,
    parsed_parameter: &Parameter,
    search_param: &SearchParameter,
) -> Result<serde_json::Value, QueryBuildError> {
    let column_name = namespace_parameter(namespace, search_param);
    let params = parsed_parameter
        .value
        .iter()
        .map(|value| number_value(&column_name, value))
        .collect::<Result<Vec<serde_json::Value>, QueryBuildError>>()?;

    Ok(json!({
        "bool": {
            "should": params
        }
    }))
}

fn number_value(column_name: &str, value: &str) -> Result<serde_json::Value, QueryBuildError> {
    let (prefix, value) = parse_prefix(value);
    let range = get_decimal_range(value)
        .map_err(|_e| QueryBuildError::InvalidParameterValue(value.to_string()))?;
    let range_query = |bounds: serde_json::Value| json!({ "range": { column_name: bounds } });

    Ok(match prefix {
        Some("ne") => json!({
            "bool": {
                "must_not": range_query(json!({ "gte": range.start, "lte": range.end }))
            }
        }),
        // For a single value, `sa`/`eb` mean the same as `gt`/`lt`.
        Some("gt" | "sa") => range_query(json!({ "gt": range.end })),
        Some("lt" | "eb") => range_query(json!({ "lt": range.start })),
        Some("ge") => range_query(json!({ "gte": range.start })),
        Some("le") => range_query(json!({ "lte": range.end })),
        Some("eq") | None => range_query(json!({ "gte": range.start, "lte": range.end })),
        Some("ap") => {
            let (low, high) = approximate_decimal(&range);
            range_query(json!({ "gte": low, "lte": high }))
        }
        Some(prefix) => return Err(QueryBuildError::UnsupportedPrefix(prefix.to_string())),
    })
}
