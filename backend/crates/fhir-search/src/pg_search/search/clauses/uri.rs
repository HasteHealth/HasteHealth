use haste_fhir_client::url::Parameter;

use super::{ClauseTarget, SqlClause, SqlParam, missing_only, require_values};
use crate::pg_search::search::QueryBuildError;

pub fn uri_clause(
    parsed_parameter: &Parameter,
    target: &ClauseTarget,
) -> Result<SqlClause, QueryBuildError> {
    if missing_only(parsed_parameter)? {
        return target.missing(parsed_parameter);
    }

    require_values(parsed_parameter)?;

    let value_expr = target.value_expr()?;
    let mut params = target.params();
    let mut or_clauses = Vec::new();

    for value in &parsed_parameter.value {
        let idx = params.len() + 1;
        or_clauses.push(format!("{value_expr} = ${idx}"));
        params.push(SqlParam::Text(value.clone()));
    }

    Ok(target.finish(false, &or_clauses.join(" OR "), params))
}
