use haste_fhir_client::url::Parameter;

use super::{
    ClauseTarget, SqlClause, SqlParam, bind, or_predicates, reference_exprs, require_values,
    target_params, wrap_predicate,
};
use crate::pg_search::search::QueryBuildError;

/// Matches `id` or `Type/id`.
pub fn reference_clause(
    parsed_parameter: &Parameter,
    target: &ClauseTarget,
) -> Result<SqlClause, QueryBuildError> {
    require_values(parsed_parameter)?;

    let (type_column, id_column) = reference_exprs(target)?;
    let (predicate, params) = or_predicates(
        &parsed_parameter.value,
        target_params(target),
        |value, params| match value.split('/').collect::<Vec<_>>()[..] {
            [id] => {
                let (params, i) = bind(params, [SqlParam::Text(id.to_string())]);
                Ok((format!("{id_column} = ${i}"), params))
            }
            [resource_type, id] => {
                let (params, i) = bind(
                    params,
                    [
                        SqlParam::Text(resource_type.to_string()),
                        SqlParam::Text(id.to_string()),
                    ],
                );
                Ok((
                    format!("({type_column} = ${i} AND {id_column} = ${})", i + 1),
                    params,
                ))
            }
            _ => Err(QueryBuildError::InvalidParameterValue(value.to_string())),
        },
    )?;

    Ok(wrap_predicate(target, false, &predicate, params))
}
