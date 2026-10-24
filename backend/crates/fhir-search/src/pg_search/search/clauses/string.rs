use super::{
    ClauseTarget, SqlClause, SqlParam, bind, escape_like, or_predicates, target_params, value_expr,
    wrap_predicate,
};
use crate::query::{QueryBuildError, value::StringValues};

/// Matches any of the strings, compared as their modifier says.
pub fn string_clause(
    values: &StringValues<'_>,
    target: &ClauseTarget,
) -> Result<SqlClause, QueryBuildError> {
    let column = value_expr(target)?;
    let (predicate, params) =
        or_predicates(values.values(), target_params(target), |value, params| {
            let value = match values {
                StringValues::Prefix(_) | StringValues::Contains(_) => escape_like(value),
                StringValues::Exact(_) | StringValues::Text(_) => value.clone(),
            };
            let (params, i) = bind(params, [SqlParam::Text(value)]);
            Ok((string_predicate(values, &column, i)?, params))
        })?;

    Ok(wrap_predicate(target, false, &predicate, params))
}

/// Compares `column` against the term bound at `$i`.
fn string_predicate(
    values: &StringValues<'_>,
    column: &str,
    i: usize,
) -> Result<String, QueryBuildError> {
    Ok(match values {
        // Case-insensitive prefix.
        StringValues::Prefix(_) => format!("LOWER({column}) LIKE LOWER(${i} || '%') ESCAPE '\\'"),
        // Case-sensitive equality.
        StringValues::Exact(_) => format!("{column} = ${i}"),
        // Case-insensitive substring.
        StringValues::Contains(_) => {
            format!("LOWER({column}) LIKE LOWER('%' || ${i} || '%') ESCAPE '\\'")
        }
        // Needs a text index.
        StringValues::Text(_) => {
            return Err(QueryBuildError::unsupported_modifier(values.modifier()));
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pg_search::schema::ParamColumns;
    use std::borrow::Cow;

    fn string_target() -> ClauseTarget {
        ClauseTarget::DirectColumn {
            alias: Cow::Borrowed("rt"),
            columns: ParamColumns::String {
                value: "name".to_string(),
            },
        }
    }

    #[test]
    fn like_values_escape_wildcards_and_the_escape_character() {
        let values = vec!["100%_\\".to_string()];
        let prefix = string_clause(
            &StringValues::Prefix(Cow::Borrowed(&values)),
            &string_target(),
        )
        .unwrap();
        assert!(matches!(
            prefix.params.as_slice(),
            [SqlParam::Text(value)] if value == "100\\%\\_\\\\"
        ));
        assert!(prefix.sql.contains("ESCAPE '\\'"), "{}", prefix.sql);

        let contains = string_clause(
            &StringValues::Contains(Cow::Borrowed(&values)),
            &string_target(),
        )
        .unwrap();
        assert!(matches!(
            contains.params.as_slice(),
            [SqlParam::Text(value)] if value == "100\\%\\_\\\\"
        ));
        assert!(contains.sql.contains("ESCAPE '\\'"), "{}", contains.sql);
    }

    #[test]
    fn exact_values_are_not_escaped() {
        let values = vec!["100%_\\".to_string()];
        let clause = string_clause(
            &StringValues::Exact(Cow::Borrowed(&values)),
            &string_target(),
        )
        .unwrap();
        assert!(matches!(
            clause.params.as_slice(),
            [SqlParam::Text(value)] if value == "100%_\\"
        ));
        assert!(!clause.sql.contains("ESCAPE"), "{}", clause.sql);
    }
}
