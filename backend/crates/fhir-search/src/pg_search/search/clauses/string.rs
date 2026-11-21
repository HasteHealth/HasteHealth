use super::{
    ClauseTarget, SqlClause, SqlParam, bind, escape_like, or_predicates, target_params, value_expr,
    wrap_predicate,
};
use crate::pg_search::schema::{STRING_PREFIX_CHARS, string_prefix_expr};
use crate::query::{QueryBuildError, value::StringValues};

/// Matches any of the strings, compared as their modifier says.
pub fn string_clause(
    values: &StringValues<'_>,
    target: &ClauseTarget,
) -> Result<SqlClause, QueryBuildError> {
    let column = value_expr(target)?;
    let (predicate, params) =
        or_predicates(values.values(), target_params(target), |term, params| {
            let value = match values {
                StringValues::Prefix(_) | StringValues::Contains(_) => escape_like(term),
                StringValues::Exact(_) | StringValues::Text(_) => term.clone(),
            };
            let (params, i) = bind(params, [SqlParam::Text(value)]);
            Ok((string_predicate(values, &column, term, i)?, params))
        })?;

    Ok(wrap_predicate(target, false, &predicate, params))
}

/// Compares `column` against `term`, bound at `$i`.
fn string_predicate(
    values: &StringValues<'_>,
    column: &str,
    term: &str,
    i: usize,
) -> Result<String, QueryBuildError> {
    let indexed = string_prefix_expr(column);
    Ok(match values {
        // Case-insensitive prefix, on the indexed expression. It covers the
        // first `STRING_PREFIX_CHARS` of a value, so a term that fits in them
        // matches exactly as it would on the whole value.
        StringValues::Prefix(_) if term.chars().count() <= STRING_PREFIX_CHARS => {
            format!("{indexed} LIKE LOWER(${i} || '%') ESCAPE '\\'")
        }
        // A longer term can only be matched on the whole value.
        StringValues::Prefix(_) => format!("LOWER({column}) LIKE LOWER(${i} || '%') ESCAPE '\\'"),
        // Case-sensitive equality. The indexed term finds the rows;
        // `column = $i` keeps the match case-sensitive and whole.
        StringValues::Exact(_) => {
            format!("{indexed} = LOWER(LEFT(${i}, {STRING_PREFIX_CHARS})) AND {column} = ${i}")
        }
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

    /// The truncated, lowered value is the only indexed expression on string
    /// columns, so a bare `value = $n` reads every row of the parameter.
    #[test]
    fn exact_compares_indexed_values_so_the_index_applies() {
        let values = vec!["Smith".to_string()];
        let target = string_target();
        let clause = string_clause(&StringValues::Exact(Cow::Borrowed(&values)), &target).unwrap();
        let column = value_expr(&target).unwrap();
        let i = clause.params.len();
        assert!(
            clause.sql.contains(&format!(
                "{} = LOWER(LEFT(${i}, {STRING_PREFIX_CHARS})) AND {column} = ${i}",
                string_prefix_expr(&column)
            )),
            "{}",
            clause.sql
        );
    }

    /// A prefix term longer than the indexed prefix cannot be answered by the
    /// index; it falls back to the whole value instead of matching nothing.
    #[test]
    fn prefix_terms_longer_than_the_indexed_prefix_compare_the_whole_value() {
        let target = string_target();
        let column = value_expr(&target).unwrap();

        let short = vec!["a".repeat(STRING_PREFIX_CHARS)];
        let clause = string_clause(&StringValues::Prefix(Cow::Borrowed(&short)), &target).unwrap();
        assert!(
            clause.sql.contains(&string_prefix_expr(&column)),
            "{}",
            clause.sql
        );

        let long = vec!["a".repeat(STRING_PREFIX_CHARS + 1)];
        let clause = string_clause(&StringValues::Prefix(Cow::Borrowed(&long)), &target).unwrap();
        assert!(
            clause.sql.contains(&format!("LOWER({column}) LIKE")),
            "{}",
            clause.sql
        );
    }
}
