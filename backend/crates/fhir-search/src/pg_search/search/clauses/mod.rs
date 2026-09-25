//! Per-type WHERE clause builders.
//!
//! Each builder turns one search parameter into a [`SqlClause`] against its
//! [`ClauseTarget`]. Bind lists are threaded by value: a helper takes the
//! params so far and returns them extended, so the next placeholder is always
//! `params.len() + 1`.

mod date;
mod number;
mod quantity;
mod reference;
mod string;
mod token;
mod uri;

pub use date::*;
pub use number::*;
pub use quantity::*;
pub use reference::*;
pub use string::*;
pub use token::*;
pub use uri::*;

use haste_fhir_client::url::Parameter;

use crate::pg_search::{schema::ParamColumns, search::QueryBuildError};

/// Alias of the joined resource type table.
pub const RESOURCE_TABLE_ALIAS: &str = "rt";

/// Alias of the anchor table, the `FROM` of every search.
pub const ANCHOR_TABLE_ALIAS: &str = "sr";

/// Alias of a shared table inside its `EXISTS` subquery.
pub const SHARED_TABLE_ALIAS: &str = "v";

/// Where a parameter's values are stored.
#[derive(Debug, Clone)]
pub enum ClauseTarget {
    /// Scalar columns on the row under `alias` (type table or anchor).
    DirectColumn {
        alias: &'static str,
        columns: ParamColumns,
    },
    /// Rows in a shared table, selected by the parameter's identity hash.
    Dynamic { table: String, param_identity: i64 },
}

/// A quantity's value range and unit.
pub struct QuantityExprs {
    pub start: String,
    pub end: String,
    pub system: String,
    pub code: String,
}

/// A WHERE fragment and its binds. Placeholders start at `$1`; see
/// [`rebase_placeholders`].
#[derive(Debug, Clone)]
pub struct SqlClause {
    pub sql: String,
    pub params: Vec<SqlParam>,
}

/// A typed bind value.
#[derive(Debug, Clone)]
pub enum SqlParam {
    Text(String),
    Int64(i64),
    Float64(f64),
}

/// The binds every clause on `target` starts with: `$1` is the parameter
/// identity for a shared table, nothing for a column.
#[must_use]
pub fn target_params(target: &ClauseTarget) -> Vec<SqlParam> {
    match target {
        ClauseTarget::DirectColumn { .. } => Vec::new(),
        ClauseTarget::Dynamic { param_identity, .. } => vec![SqlParam::Int64(*param_identity)],
    }
}

/// The single value column of a string, uri or number.
///
/// # Errors
///
/// Returns an error if the column shape doesn't match the parameter type.
pub fn value_expr(target: &ClauseTarget) -> Result<String, QueryBuildError> {
    match target {
        ClauseTarget::DirectColumn {
            alias,
            columns:
                ParamColumns::String { value }
                | ParamColumns::Uri { value }
                | ParamColumns::Number { value },
        } => Ok(direct_column(alias, value)),
        ClauseTarget::Dynamic { .. } => Ok(shared_column("value")),
        ClauseTarget::DirectColumn { .. } => Err(unsupported("a single value column")),
    }
}

/// A token's `(system, code)`. With no system column (`_id`) the system is SQL
/// `NULL`, so `|abc` matches and `sys|abc` never does.
///
/// # Errors
///
/// Returns an error if the column shape isn't a token's.
pub fn token_exprs(target: &ClauseTarget) -> Result<(String, String), QueryBuildError> {
    let (system, code) = match target {
        // A quantity's unit is itself a token.
        ClauseTarget::DirectColumn {
            alias,
            columns: ParamColumns::Quantity { system, code, .. },
        } => (
            Some(direct_column(alias, system)),
            direct_column(alias, code),
        ),
        ClauseTarget::DirectColumn {
            alias,
            columns: ParamColumns::Token { system, code },
        } => (
            system.as_ref().map(|system| direct_column(alias, system)),
            direct_column(alias, code),
        ),
        ClauseTarget::Dynamic { .. } => (Some(shared_column("system")), shared_column("code")),
        ClauseTarget::DirectColumn { .. } => return Err(unsupported("token columns")),
    };

    Ok((system.unwrap_or_else(|| "NULL".to_string()), code))
}

/// A date's `(start, end)` bounds.
///
/// # Errors
///
/// Returns an error if the column shape isn't a date's.
pub fn date_exprs(target: &ClauseTarget) -> Result<(String, String), QueryBuildError> {
    match target {
        ClauseTarget::DirectColumn {
            alias,
            columns: ParamColumns::Date { start, end },
        } => Ok((direct_column(alias, start), direct_column(alias, end))),
        ClauseTarget::Dynamic { .. } => Ok((shared_column("start_ms"), shared_column("end_ms"))),
        ClauseTarget::DirectColumn { .. } => Err(unsupported("date columns")),
    }
}

/// A reference's `(target type, target id)`.
///
/// # Errors
///
/// Returns an error if the column shape isn't a reference's.
pub fn reference_exprs(target: &ClauseTarget) -> Result<(String, String), QueryBuildError> {
    match target {
        ClauseTarget::DirectColumn {
            alias,
            columns:
                ParamColumns::Reference {
                    target_type,
                    target_id,
                },
        } => Ok((
            direct_column(alias, target_type),
            direct_column(alias, target_id),
        )),
        ClauseTarget::Dynamic { .. } => Ok((
            shared_column("target_resource_type"),
            shared_column("target_id"),
        )),
        ClauseTarget::DirectColumn { .. } => Err(unsupported("reference columns")),
    }
}

/// A quantity's range and unit.
///
/// # Errors
///
/// Returns an error if the column shape isn't a quantity's.
pub fn quantity_exprs(target: &ClauseTarget) -> Result<QuantityExprs, QueryBuildError> {
    match target {
        ClauseTarget::DirectColumn {
            alias,
            columns:
                ParamColumns::Quantity {
                    start,
                    end,
                    system,
                    code,
                },
        } => Ok(QuantityExprs {
            start: direct_column(alias, start),
            end: direct_column(alias, end),
            system: direct_column(alias, system),
            code: direct_column(alias, code),
        }),
        ClauseTarget::Dynamic { .. } => Ok(QuantityExprs {
            start: shared_column("start_value"),
            end: shared_column("end_value"),
            system: shared_column("start_system"),
            code: shared_column("start_code"),
        }),
        ClauseTarget::DirectColumn { .. } => Err(unsupported("quantity columns")),
    }
}

/// Wraps `predicate` for `target`: a test on the row itself, or an `EXISTS`
/// over the parameter's shared-table rows.
#[must_use]
pub fn wrap_predicate(
    target: &ClauseTarget,
    negate: bool,
    predicate: &str,
    params: Vec<SqlParam>,
) -> SqlClause {
    let sql = match target {
        ClauseTarget::DirectColumn { .. } => direct_predicate(negate, predicate),
        ClauseTarget::Dynamic { table, .. } => dynamic_exists(table, negate, Some(predicate)),
    };
    SqlClause { sql, params }
}

/// `:missing=true|false`: whether the parameter has any value.
///
/// # Errors
///
/// Returns an error for a value other than `true`/`false`, or a column shape
/// with no single value column.
pub fn missing_clause(
    target: &ClauseTarget,
    parsed_parameter: &Parameter,
) -> Result<SqlClause, QueryBuildError> {
    let missing = match parsed_parameter.value.first().map(String::as_str) {
        Some("true") => true,
        Some("false") => false,
        _ => {
            return Err(QueryBuildError::InvalidParameterValue(
                parsed_parameter.name.clone(),
            ));
        }
    };

    match target {
        // A scalar column is NULL when there is no value.
        ClauseTarget::DirectColumn { .. } => {
            let column = value_expr(target)?;
            let test = if missing { "IS NULL" } else { "IS NOT NULL" };
            Ok(SqlClause {
                sql: format!("{column} {test}"),
                params: Vec::new(),
            })
        }
        // No value means no shared-table row.
        ClauseTarget::Dynamic {
            table,
            param_identity,
        } => Ok(SqlClause {
            sql: dynamic_exists(table, missing, None),
            params: vec![SqlParam::Int64(*param_identity)],
        }),
    }
}

/// Builds one predicate per value and OR-joins them. `predicate` receives the
/// value and the binds so far, and returns its SQL and the extended binds.
pub(super) fn or_predicates(
    values: &[String],
    params: Vec<SqlParam>,
    predicate: impl Fn(&str, Vec<SqlParam>) -> Result<(String, Vec<SqlParam>), QueryBuildError>,
) -> Result<(String, Vec<SqlParam>), QueryBuildError> {
    let (fragments, params) = values.iter().try_fold(
        (Vec::with_capacity(values.len()), params),
        |(mut fragments, params), value| {
            let (fragment, params) = predicate(value, params)?;
            fragments.push(fragment);
            Ok((fragments, params))
        },
    )?;

    Ok((fragments.join(" OR "), params))
}

/// Appends `values` to `params`; returns them and the first value's
/// placeholder number.
pub(super) fn bind<const N: usize>(
    mut params: Vec<SqlParam>,
    values: [SqlParam; N],
) -> (Vec<SqlParam>, usize) {
    let first = params.len() + 1;
    params.extend(values);
    (params, first)
}

/// Adds `offset` to every `$n` placeholder in a clause with `param_count`
/// binds.
#[must_use]
pub fn rebase_placeholders(sql: &str, param_count: usize, offset: usize) -> String {
    if offset == 0 {
        return sql.to_string();
    }
    // Highest first, or `$1` would also match inside `$10`.
    (1..=param_count).rev().fold(sql.to_string(), |sql, i| {
        sql.replace(&format!("${i}"), &format!("${}", i + offset))
    })
}

/// The identity of a parameter's shared-table rows in this project.
pub fn resolve_param_identity(
    search_param: &haste_fhir_model::r4::generated::resources::SearchParameter,
    tenant: &str,
    project: &str,
) -> i64 {
    crate::pg_search::keys::param_identity(
        tenant,
        project,
        search_param.url.value.as_deref().unwrap_or(""),
    )
}

fn unsupported(shape: &str) -> QueryBuildError {
    QueryBuildError::UnsupportedParameter(format!("search parameter is not backed by {shape}"))
}

fn direct_column(alias: &str, column: &str) -> String {
    format!("{alias}.\"{column}\"")
}

fn shared_column(column: &str) -> String {
    format!("{SHARED_TABLE_ALIAS}.{column}")
}

/// `[NOT] EXISTS (SELECT 1 FROM {table} v WHERE <correlate> AND
/// v.param_identity = $1 [AND (predicate)])`.
fn dynamic_exists(table: &str, negate: bool, predicate: Option<&str>) -> String {
    let prefix = if negate { "NOT " } else { "" };
    let extra = predicate.map_or_else(String::new, |p| format!(" AND ({p})"));

    format!(
        "{prefix}EXISTS (SELECT 1 FROM {table} {SHARED_TABLE_ALIAS} \
         WHERE {SHARED_TABLE_ALIAS}.res_key = {ANCHOR_TABLE_ALIAS}.res_key \
         AND {SHARED_TABLE_ALIAS}.param_identity = $1{extra})"
    )
}

/// A predicate on the row's own columns. Reading the column directly (no
/// `unnest`) keeps its index usable.
///
/// Negation uses `IS NOT TRUE` rather than `NOT`, so `:not` still matches a
/// NULL (valueless) column.
fn direct_predicate(negate: bool, predicate: &str) -> String {
    if negate {
        format!("(({predicate}) IS NOT TRUE)")
    } else {
        format!("({predicate})")
    }
}

/// Rejects a parameter with no values.
fn require_values(parsed_parameter: &Parameter) -> Result<(), QueryBuildError> {
    if parsed_parameter.value.is_empty() {
        Err(QueryBuildError::InvalidParameterValue(
            parsed_parameter.name.clone(),
        ))
    } else {
        Ok(())
    }
}

/// `true` for `:missing`, `false` for no modifier, an error for any other.
fn missing_only(parsed_parameter: &Parameter) -> Result<bool, QueryBuildError> {
    match parsed_parameter.modifier.as_deref() {
        Some("missing") => Ok(true),
        Some(modifier) => Err(QueryBuildError::UnsupportedModifier(modifier.to_string())),
        None => Ok(false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn direct(columns: ParamColumns) -> ClauseTarget {
        ClauseTarget::DirectColumn {
            alias: RESOURCE_TABLE_ALIAS,
            columns,
        }
    }

    fn dynamic() -> ClauseTarget {
        ClauseTarget::Dynamic {
            table: "r4_param_token_idx".to_string(),
            param_identity: 7,
        }
    }

    #[test]
    fn a_target_wraps_its_own_predicate() {
        let column = direct(ParamColumns::String {
            value: "name".to_string(),
        });
        assert_eq!(
            wrap_predicate(&column, false, "rt.\"name\" = $1", Vec::new()).sql,
            "(rt.\"name\" = $1)"
        );

        let sql = wrap_predicate(&dynamic(), false, "v.code = $2", Vec::new()).sql;
        assert!(
            sql.starts_with("EXISTS (SELECT 1 FROM r4_param_token_idx v"),
            "{sql}"
        );
        assert!(sql.contains("v.res_key = sr.res_key"), "{sql}");
        assert!(sql.contains("v.param_identity = $1"), "{sql}");
        assert!(sql.ends_with("AND (v.code = $2))"), "{sql}");
    }

    /// A NULL column makes the predicate NULL, and `:not` must still match it.
    #[test]
    fn a_negated_column_predicate_matches_a_missing_value() {
        let target = direct(ParamColumns::Token {
            system: None,
            code: "gender_code".to_string(),
        });

        assert_eq!(
            wrap_predicate(&target, true, "rt.\"gender_code\" = $1", Vec::new()).sql,
            "((rt.\"gender_code\" = $1) IS NOT TRUE)"
        );
    }

    #[test]
    fn a_mismatched_column_shape_is_rejected() {
        let target = direct(ParamColumns::String {
            value: "name".to_string(),
        });

        assert!(value_expr(&target).is_ok());
        assert!(date_exprs(&target).is_err());
    }

    #[test]
    fn rebase_shifts_placeholders() {
        assert_eq!(
            rebase_placeholders("a = $1 AND b = $2", 2, 3),
            "a = $4 AND b = $5"
        );
    }
}
