use crate::{
    elastic_search::search::clauses::{all_of, any_of, match_query, nested, range_query},
    query::value::{NumberValue, QuantityValue},
    search_ranges::{above, below},
};
use haste_fhir_client::url::Prefix;

/// Matches any of `[prefix]value[|system|code]`; an unconstrained segment is
/// left out.
pub fn quantity(field: &str, values: &[QuantityValue]) -> serde_json::Value {
    any_of(
        values
            .iter()
            .map(|quantity| nested(field, quantity_query(field, quantity)))
            .collect(),
    )
}

fn quantity_query(field: &str, quantity: &QuantityValue) -> serde_json::Value {
    let mut filter = Vec::new();
    if let Some(amount) = &quantity.amount {
        filter.extend(amount_queries(field, amount));
    }
    if let Some(system) = quantity.system {
        filter.push(match_query(field, "start_system", system));
        filter.push(match_query(field, "end_system", system));
    }
    if let Some(code) = quantity.code {
        filter.push(match_query(field, "start_code", code));
        filter.push(match_query(field, "end_code", code));
    }
    all_of(filter, Vec::new())
}

/// Compares the indexed `[start_value, end_value]` range against
/// `[prefix]amount`.
fn amount_queries(field: &str, amount: &NumberValue) -> Vec<serde_json::Value> {
    let bound =
        |name: &str, comparison: &str, value: f64| range_query(field, name, comparison, value);
    let (start, end) = (amount.range.start, amount.range.end);
    // The value falls inside the indexed range.
    let contains = || {
        vec![
            bound("start_value", "lte", amount.value),
            bound("end_value", "gte", amount.value),
        ]
    };

    match amount.prefix {
        Prefix::Eq => contains(),
        Prefix::Ne => vec![all_of(Vec::new(), vec![all_of(contains(), Vec::new())])],
        // Wholly above or below the search value's precision range.
        Prefix::Gt | Prefix::Sa => vec![bound("start_value", "gte", above(&amount.range))],
        Prefix::Lt | Prefix::Eb => vec![bound("end_value", "lte", below(&amount.range))],
        Prefix::Ge => vec![bound("start_value", "gte", start)],
        Prefix::Le => vec![bound("end_value", "lte", end)],
        // Overlaps the range, widened when it was parsed.
        Prefix::Ap => vec![
            bound("start_value", "lte", end),
            bound("end_value", "gte", start),
        ],
    }
}
