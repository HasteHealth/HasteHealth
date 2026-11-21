use crate::{
    elastic_search::search::clauses::{all_of, any_of, nested, range_query},
    query::value::DateValue,
};
use haste_fhir_client::url::Prefix;

/// Compares each supplied date's range against the indexed period.
pub fn date(field: &str, values: &[DateValue]) -> serde_json::Value {
    any_of(
        values
            .iter()
            .map(|date| nested(field, date_query(field, date)))
            .collect(),
    )
}

fn date_query(field: &str, date: &DateValue) -> serde_json::Value {
    let bound =
        |name: &str, comparison: &str, value: i64| range_query(field, name, comparison, value);
    let (start, end) = (date.range.start, date.range.end);
    let overlaps = || {
        all_of(
            vec![bound("start", "lte", end), bound("end", "gte", start)],
            Vec::new(),
        )
    };

    match date.prefix {
        // Starts after the search range ends.
        Prefix::Gt | Prefix::Sa => bound("start", "gt", end),
        Prefix::Lt => bound("start", "lt", start),
        // Ends before the search range begins.
        Prefix::Eb => bound("end", "lt", start),
        Prefix::Ge => bound("start", "gte", start),
        Prefix::Le => bound("start", "lte", end),
        Prefix::Ne => all_of(Vec::new(), vec![overlaps()]),
        // Overlaps; `ap` was widened when it was parsed.
        Prefix::Eq | Prefix::Ap => overlaps(),
    }
}
