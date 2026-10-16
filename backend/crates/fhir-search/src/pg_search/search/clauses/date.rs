use haste_fhir_client::url::Prefix;

use super::{
    ClauseTarget, SqlClause, SqlParam, bind, date_exprs, or_predicates, target_params,
    wrap_predicate,
};
use crate::query::{QueryBuildError, value::DateValue};

/// Compares each supplied date's range against the indexed period.
pub fn date_clause(
    values: &[DateValue],
    target: &ClauseTarget,
) -> Result<SqlClause, QueryBuildError> {
    let (start, end) = date_exprs(target)?;
    let (predicate, params) = or_predicates(values, target_params(target), |date, params| {
        Ok(date_predicate(date, &start, &end, params))
    })?;

    Ok(wrap_predicate(target, false, &predicate, params))
}

fn date_predicate(
    date: &DateValue,
    start: &str,
    end: &str,
    params: Vec<SqlParam>,
) -> (String, Vec<SqlParam>) {
    let (lower, upper) = (
        SqlParam::Int64(date.range.start),
        SqlParam::Int64(date.range.end),
    );

    match date.prefix {
        // Starts after the search range ends.
        Prefix::Gt | Prefix::Sa => {
            let (params, i) = bind(params, [upper]);
            (format!("{start} > ${i}"), params)
        }
        // Starts before the search range begins.
        Prefix::Lt => {
            let (params, i) = bind(params, [lower]);
            (format!("{start} < ${i}"), params)
        }
        // Ends before the search range begins.
        Prefix::Eb => {
            let (params, i) = bind(params, [lower]);
            (format!("{end} < ${i}"), params)
        }
        Prefix::Ge => {
            let (params, i) = bind(params, [lower]);
            (format!("{start} >= ${i}"), params)
        }
        Prefix::Le => {
            let (params, i) = bind(params, [upper]);
            (format!("{end} <= ${i}"), params)
        }
        // Doesn't overlap.
        Prefix::Ne => {
            let (params, i) = bind(params, [lower, upper]);
            (
                format!("NOT ({start} <= ${} AND {end} >= ${i})", i + 1),
                params,
            )
        }
        // Overlaps; `ap` was widened when it was parsed.
        Prefix::Eq | Prefix::Ap => {
            let (params, i) = bind(params, [lower, upper]);
            (format!("({start} <= ${} AND {end} >= ${i})", i + 1), params)
        }
    }
}
