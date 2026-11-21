use crate::{
    elastic_search::search::clauses::{all_of, any_of},
    query::value::NumberValue,
};
use haste_fhir_client::url::Prefix;
use serde_json::json;

/// A FHIR number has implicit precision, so equality is range containment.
pub fn number(field: &str, values: &[NumberValue]) -> serde_json::Value {
    any_of(
        values
            .iter()
            .map(|number| number_query(field, number))
            .collect(),
    )
}

fn number_query(field: &str, number: &NumberValue) -> serde_json::Value {
    let range = |bounds: serde_json::Value| json!({ "range": { field: bounds } });
    let (start, end) = (number.range.start, number.range.end);
    let within = || range(json!({ "gte": start, "lte": end }));

    match number.prefix {
        Prefix::Ne => all_of(Vec::new(), vec![within()]),
        // For a single value, `sa`/`eb` mean the same as `gt`/`lt`.
        Prefix::Gt | Prefix::Sa => range(json!({ "gt": end })),
        Prefix::Lt | Prefix::Eb => range(json!({ "lt": start })),
        Prefix::Ge => range(json!({ "gte": start })),
        Prefix::Le => range(json!({ "lte": end })),
        // `ap` was widened when it was parsed.
        Prefix::Eq | Prefix::Ap => within(),
    }
}
