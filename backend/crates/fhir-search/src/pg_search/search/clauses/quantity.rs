use haste_fhir_client::url::Prefix;

use super::{
    ClauseTarget, QuantityExprs, SqlClause, SqlParam, all_of, bind, equals, or_predicates,
    quantity_exprs, target_params, wrap_predicate,
};
use crate::{
    query::{
        QueryBuildError,
        value::{NumberValue, QuantityValue},
    },
    search_ranges::{above, below},
};

/// Matches any of `[prefix]value[|system|code]`; an unconstrained segment is
/// left out. Value and unit share a row, so no recombination is needed.
pub fn quantity_clause(
    values: &[QuantityValue],
    target: &ClauseTarget,
) -> Result<SqlClause, QueryBuildError> {
    let exprs = quantity_exprs(target)?;
    let (predicate, params) = or_predicates(values, target_params(target), |quantity, params| {
        Ok(quantity_predicate(quantity, &exprs, params))
    })?;

    Ok(wrap_predicate(target, false, &predicate, params))
}

fn quantity_predicate(
    quantity: &QuantityValue,
    exprs: &QuantityExprs,
    params: Vec<SqlParam>,
) -> (String, Vec<SqlParam>) {
    let (params, amount) = match &quantity.amount {
        None => (params, None),
        Some(amount) => {
            let (params, sql) = amount_predicate(amount, exprs, params);
            (params, Some(sql))
        }
    };
    let (params, system) = equals(params, &exprs.system, quantity.system.as_deref());
    let (params, code) = equals(params, &exprs.code, quantity.code.as_deref());
    let matches = all_of([amount, system, code]);

    // `ne` is the negation of the whole match, so another unit also qualifies.
    let is_ne = quantity
        .amount
        .as_ref()
        .is_some_and(|amount| amount.prefix == Prefix::Ne);
    if is_ne {
        (format!("({matches}) IS NOT TRUE"), params)
    } else {
        (matches, params)
    }
}

/// Compares the indexed `[start, end]` range against `[prefix]amount`.
fn amount_predicate(
    amount: &NumberValue,
    exprs: &QuantityExprs,
    params: Vec<SqlParam>,
) -> (Vec<SqlParam>, String) {
    let (start, end) = (&exprs.start, &exprs.end);
    let float = SqlParam::Float64;

    match amount.prefix {
        // The value falls inside the indexed range (`ne` negates the match).
        Prefix::Eq | Prefix::Ne => {
            let (params, i) = bind(params, [float(amount.value)]);
            (params, format!("({start} <= ${i} AND {end} >= ${i})"))
        }
        // Wholly above or below the search value's precision range.
        Prefix::Gt | Prefix::Sa => {
            let (params, i) = bind(params, [float(above(&amount.range))]);
            (params, format!("{start} >= ${i}"))
        }
        Prefix::Lt | Prefix::Eb => {
            let (params, i) = bind(params, [float(below(&amount.range))]);
            (params, format!("{end} <= ${i}"))
        }
        Prefix::Ge => {
            let (params, i) = bind(params, [float(amount.range.start)]);
            (params, format!("{start} >= ${i}"))
        }
        Prefix::Le => {
            let (params, i) = bind(params, [float(amount.range.end)]);
            (params, format!("{end} <= ${i}"))
        }
        // Overlaps the range, widened when it was parsed.
        Prefix::Ap => {
            let (params, i) = bind(params, [float(amount.range.start), float(amount.range.end)]);
            (params, format!("({start} <= ${} AND {end} >= ${i})", i + 1))
        }
    }
}
