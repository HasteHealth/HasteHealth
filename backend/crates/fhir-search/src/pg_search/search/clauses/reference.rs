use super::{
    ClauseTarget, SqlClause, SqlParam, bind, or_predicates, reference_exprs, target_params,
    wrap_predicate,
};
use crate::query::{QueryBuildError, value::ReferenceValues};

/// Matches any of the references: by id, and by type when one is named.
pub fn reference_clause(
    values: &ReferenceValues<'_>,
    target: &ClauseTarget,
) -> Result<SqlClause, QueryBuildError> {
    let references = match values {
        ReferenceValues::References(references) => references,
        // A hierarchy of references isn't followed, and a target's
        // identifier isn't indexed with the reference.
        ReferenceValues::Above(_) | ReferenceValues::Below(_) | ReferenceValues::Identifier(_) => {
            return Err(QueryBuildError::unsupported_modifier(values.modifier()));
        }
    };

    let (type_column, id_column) = reference_exprs(target)?;
    let (predicate, params) =
        or_predicates(references, target_params(target), |reference, params| {
            let text = |s: &str| SqlParam::Text(s.to_string());
            Ok(match reference.resource_type.as_deref() {
                None => {
                    let (params, i) = bind(params, [text(&reference.id)]);
                    (format!("{id_column} = ${i}"), params)
                }
                Some(resource_type) => {
                    let (params, i) = bind(params, [text(resource_type), text(&reference.id)]);
                    (
                        format!("({type_column} = ${i} AND {id_column} = ${})", i + 1),
                        params,
                    )
                }
            })
        })?;

    Ok(wrap_predicate(target, false, &predicate, params))
}
