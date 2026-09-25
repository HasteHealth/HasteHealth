use crate::{
    elastic_search::search::clauses::namespace_parameter,
    indexing_conversion::get_decimal_range,
    query::QueryBuildError,
    search_ranges::{above, approximate_decimal, below},
};
use haste_fhir_client::url::{Parameter, parse_prefix};
use haste_fhir_model::r4::generated::resources::SearchParameter;
use serde_json::json;

/// Matches any of `[prefix]value[|system|code]`; empty segments are
/// unconstrained.
pub fn quantity(
    namespace: Option<&str>,
    parsed_parameter: &Parameter,
    search_param: &SearchParameter,
) -> Result<serde_json::Value, QueryBuildError> {
    let column_name = namespace_parameter(namespace, search_param);
    let params = parsed_parameter
        .value
        .iter()
        .map(|value| quantity_value(&column_name, value))
        .collect::<Result<Vec<serde_json::Value>, QueryBuildError>>()?;

    Ok(json!({
        "bool": {
            "should": params
        }
    }))
}

fn quantity_value(column_name: &str, value: &str) -> Result<serde_json::Value, QueryBuildError> {
    let (amount, system, code) = match value.split('|').collect::<Vec<_>>()[..] {
        // A bare value matches any unit.
        [amount] => (amount, "", ""),
        [amount, system, code] => (amount, system, code),
        [_, _, _, _] => {
            return Err(QueryBuildError::UnsupportedParameterValue(
                value.to_string(),
            ));
        }
        _ => return Err(QueryBuildError::InvalidParameterValue(value.to_string())),
    };

    let field = |name: &str| format!("{column_name}.{name}");
    let matches = |name: &str, query: &str| json!({ "match": { field(name): { "query": query } } });

    let amount_clauses = if amount.is_empty() {
        vec![]
    } else {
        amount_clauses(&field, amount)?
    };
    let system_clauses = if system.is_empty() {
        vec![]
    } else {
        vec![
            matches("start_system", system),
            matches("end_system", system),
        ]
    };
    let code_clauses = if code.is_empty() {
        vec![]
    } else {
        vec![matches("start_code", code), matches("end_code", code)]
    };

    let clauses: Vec<serde_json::Value> = amount_clauses
        .into_iter()
        .chain(system_clauses)
        .chain(code_clauses)
        .collect();

    Ok(json!({
        "nested": {
            "path": column_name,
            "query": {
                "bool": {
                    "must": clauses
                }
            }
        }
    }))
}

/// Compares the indexed `[start_value, end_value]` range against
/// `[prefix]amount`.
fn amount_clauses(
    field: &dyn Fn(&str) -> String,
    amount: &str,
) -> Result<Vec<serde_json::Value>, QueryBuildError> {
    let (prefix, number) = parse_prefix(amount);
    let invalid = || QueryBuildError::InvalidParameterValue(amount.to_string());
    let range = get_decimal_range(number).map_err(|_e| invalid())?;
    let bound = |name: &str, comparison: &str, value: f64| json!({ "range": { field(name): { comparison: value } } });

    Ok(match prefix {
        // The value falls inside the indexed range.
        Some("eq" | "ne") | None => {
            let point: f64 = number.parse().map_err(|_e| invalid())?;
            let contains = vec![
                bound("start_value", "lte", point),
                bound("end_value", "gte", point),
            ];
            if prefix == Some("ne") {
                vec![json!({ "bool": { "must_not": [{ "bool": { "filter": contains } }] } })]
            } else {
                contains
            }
        }
        // Wholly above or below the search value's precision range.
        Some("gt" | "sa") => vec![bound("start_value", "gte", above(&range))],
        Some("lt" | "eb") => vec![bound("end_value", "lte", below(&range))],
        Some("ge") => vec![bound("start_value", "gte", range.start)],
        Some("le") => vec![bound("end_value", "lte", range.end)],
        // Overlaps the value widened by 10%.
        Some("ap") => {
            let (low, high) = approximate_decimal(&range);
            vec![
                bound("start_value", "lte", high),
                bound("end_value", "gte", low),
            ]
        }
        Some(p) => return Err(QueryBuildError::UnsupportedPrefix(p.to_string())),
    })
}
