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

/// The per-resource-type table's alias. Must match the JOIN in
/// `build_final_query`.
pub const RESOURCE_TABLE_ALIAS: &str = "rt";

/// The anchor table's alias. Resource-level parameters read their columns from
/// here, the only table a search naming no resource type has.
pub const ANCHOR_TABLE_ALIAS: &str = "sr";

/// The alias a shared table takes inside the `EXISTS` subquery that reads it.
pub const SHARED_TABLE_ALIAS: &str = "v";

/// Where a parameter's values live, and so how a clause looks them up.
///
/// Single-valued HL7 base parameters have scalar columns on the
/// per-resource-type table; everything that may repeat shares the
/// `{version}_param_{type}_idx` tables, discriminated by an identity hash.
#[derive(Debug, Clone)]
pub enum ClauseTarget {
    /// Columns read straight off a row — the resource type's table, or the
    /// anchor for a resource-level parameter. The alias says which.
    DirectColumn {
        alias: &'static str,
        columns: ParamColumns,
    },
    /// A row lookup in the shared table for this value type, discriminated by
    /// the parameter's identity (see [`crate::pg_search::keys`]).
    Dynamic { table: String, param_identity: i64 },
}

/// A quantity's four expressions: its range and the unit it is stated in.
pub struct QuantityExprs {
    pub start: String,
    pub end: String,
    pub system: String,
    pub code: String,
}

impl ClauseTarget {
    /// The binds that precede the clause's own: a shared table takes its
    /// parameter identity as `$1`, a column takes nothing.
    #[must_use]
    pub fn params(&self) -> Vec<SqlParam> {
        match self {
            ClauseTarget::DirectColumn { .. } => Vec::new(),
            ClauseTarget::Dynamic { param_identity, .. } => vec![SqlParam::Int64(*param_identity)],
        }
    }

    /// The single column a string, uri or number is read from.
    ///
    /// # Errors
    ///
    /// Returns an error when the column's shape does not match, meaning the
    /// schema and the search disagree about where the values are.
    pub fn value_expr(&self) -> Result<String, QueryBuildError> {
        match self {
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

    /// A token's system and code. A token that never carries a system (`_id`)
    /// gets SQL `NULL`: `|abc` matches it, `sys|abc` cannot.
    ///
    /// # Errors
    ///
    /// Returns an error when the column's shape is not a token's.
    pub fn token_exprs(&self) -> Result<(String, String), QueryBuildError> {
        let (system, code) = match self {
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

    /// The bounds of an indexed period.
    ///
    /// # Errors
    ///
    /// Returns an error when the column's shape is not a date's.
    pub fn date_exprs(&self) -> Result<(String, String), QueryBuildError> {
        match self {
            ClauseTarget::DirectColumn {
                alias,
                columns: ParamColumns::Date { start, end },
            } => Ok((direct_column(alias, start), direct_column(alias, end))),
            ClauseTarget::Dynamic { .. } => {
                Ok((shared_column("start_ms"), shared_column("end_ms")))
            }
            ClauseTarget::DirectColumn { .. } => Err(unsupported("date columns")),
        }
    }

    /// A reference's target type and id.
    ///
    /// # Errors
    ///
    /// Returns an error when the column's shape is not a reference's.
    pub fn reference_exprs(&self) -> Result<(String, String), QueryBuildError> {
        match self {
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
    /// Returns an error when the column's shape is not a quantity's.
    pub fn quantity_exprs(&self) -> Result<QuantityExprs, QueryBuildError> {
        match self {
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

    /// Wraps a predicate into the clause the target needs: a test on the row
    /// itself, or an `EXISTS` over this parameter's shared-table rows.
    #[must_use]
    pub fn finish(&self, negate: bool, predicate: &str, params: Vec<SqlParam>) -> SqlClause {
        match self {
            ClauseTarget::DirectColumn { .. } => {
                SqlClause::new(direct_predicate(negate, predicate), params)
            }
            ClauseTarget::Dynamic { table, .. } => {
                SqlClause::new(dynamic_exists(table, negate, Some(predicate)), params)
            }
        }
    }

    /// `:missing=true`/`false` — whether the parameter produced any value at
    /// all, rather than a comparison.
    ///
    /// # Errors
    ///
    /// Returns an error for a value other than `true` or `false`, or a column
    /// whose shape has no single value to test.
    pub fn missing(&self, parsed_parameter: &Parameter) -> Result<SqlClause, QueryBuildError> {
        let missing = match parsed_parameter.value.first().map(String::as_str) {
            Some("true") => true,
            Some("false") => false,
            _ => {
                return Err(QueryBuildError::InvalidParameterValue(
                    parsed_parameter.name.clone(),
                ));
            }
        };

        match self {
            // A scalar column holds the value or NULL.
            ClauseTarget::DirectColumn { .. } => {
                let column = self.value_expr()?;
                let sql = if missing {
                    format!("{column} IS NULL")
                } else {
                    format!("{column} IS NOT NULL")
                };
                Ok(SqlClause::new(sql, Vec::new()))
            }
            // A parameter with no value has no row at all.
            ClauseTarget::Dynamic {
                table,
                param_identity,
            } => Ok(SqlClause::new(
                dynamic_exists(table, missing, None),
                vec![SqlParam::Int64(*param_identity)],
            )),
        }
    }
}

fn unsupported(shape: &str) -> QueryBuildError {
    QueryBuildError::UnsupportedParameter(format!("search parameter is not backed by {shape}"))
}

/// A column on the table `alias` names.
fn direct_column(alias: &str, column: &str) -> String {
    format!("{alias}.\"{column}\"")
}

/// A shared table's own column, inside the `EXISTS` that reads it.
fn shared_column(column: &str) -> String {
    format!("{SHARED_TABLE_ALIAS}.{column}")
}

/// The `EXISTS (SELECT 1 FROM {table} v WHERE ...)` skeleton every shared-table
/// clause shares: correlation to the anchor by `res_key`, plus the
/// `param_identity = $1` discriminator. `predicate` is appended as `AND (...)`,
/// `negate` prefixes `NOT`.
fn dynamic_exists(table: &str, negate: bool, predicate: Option<&str>) -> String {
    let prefix = if negate { "NOT " } else { "" };
    let extra = predicate.map_or_else(String::new, |p| format!(" AND ({p})"));

    format!(
        "{prefix}EXISTS (SELECT 1 FROM {table} {SHARED_TABLE_ALIAS} \
         WHERE {SHARED_TABLE_ALIAS}.res_key = {ANCHOR_TABLE_ALIAS}.res_key \
         AND {SHARED_TABLE_ALIAS}.param_identity = $1{extra})"
    )
}

/// Wraps a predicate over a row's own columns.
///
/// No `unnest` and no `EXISTS`: a column exists only where the parameter cannot
/// repeat, so the predicate reads the row directly — which is also what keeps
/// the column's index usable, since `unnest(column)` is a correlated function
/// scan no index can answer.
///
/// Negation is `IS NOT TRUE`, not `NOT`: a NULL column makes the predicate
/// NULL, and `:not` must still match a resource carrying no value.
fn direct_predicate(negate: bool, predicate: &str) -> String {
    if negate {
        format!("(({predicate}) IS NOT TRUE)")
    } else {
        format!("({predicate})")
    }
}

/// Rejects a search with no value to compare.
fn require_values(parsed_parameter: &Parameter) -> Result<(), QueryBuildError> {
    if parsed_parameter.value.is_empty() {
        return Err(QueryBuildError::InvalidParameterValue(
            parsed_parameter.name.clone(),
        ));
    }
    Ok(())
}

/// Only `:missing` is accepted, and only by the types that implement it here.
fn missing_only(parsed_parameter: &Parameter) -> Result<bool, QueryBuildError> {
    match parsed_parameter.modifier.as_deref() {
        Some("missing") => Ok(true),
        Some(modifier) => Err(QueryBuildError::UnsupportedModifier(modifier.to_string())),
        None => Ok(false),
    }
}

/// A fragment of a WHERE clause with its binds. `sql` numbers its placeholders
/// from `$1`; the caller rebases them to their position in the final query.
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

impl SqlClause {
    /// Placeholders start at `$1`; the caller rebases before combining
    /// clauses.
    pub fn new(sql: String, params: Vec<SqlParam>) -> Self {
        SqlClause { sql, params }
    }

    /// Adds `offset` to every `$N` placeholder.
    pub fn rebase(&mut self, offset: usize) {
        if offset == 0 {
            return;
        }
        // Highest N first, or $1 would be replaced inside $10.
        let mut rebased = self.sql.clone();
        for i in (1..=self.params.len()).rev() {
            let old = format!("${i}");
            let new = format!("${}", i + offset);
            rebased = rebased.replace(&old, &new);
        }
        self.sql = rebased;
    }
}

/// The identity this parameter's shared-table rows carry in the project.
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

    /// A column clause reads the row; the same clause over a shared table
    /// becomes a correlated `EXISTS`.
    #[test]
    fn a_target_wraps_its_own_predicate() {
        let column = direct(ParamColumns::String {
            value: "name".to_string(),
        });
        assert_eq!(
            column.finish(false, "rt.\"name\" = $1", Vec::new()).sql,
            "(rt.\"name\" = $1)"
        );

        let sql = dynamic().finish(false, "v.code = $2", Vec::new()).sql;
        assert!(
            sql.starts_with("EXISTS (SELECT 1 FROM r4_param_token_idx v"),
            "{sql}"
        );
        assert!(sql.contains("v.res_key = sr.res_key"), "{sql}");
        assert!(sql.contains("v.param_identity = $1"), "{sql}");
        assert!(sql.ends_with("AND (v.code = $2))"), "{sql}");
    }

    /// `:not` must match a resource carrying no value, and a NULL column makes
    /// the predicate NULL rather than false.
    #[test]
    fn a_negated_column_predicate_matches_a_missing_value() {
        let target = direct(ParamColumns::Token {
            system: None,
            code: "gender_code".to_string(),
        });

        assert_eq!(
            target
                .finish(true, "rt.\"gender_code\" = $1", Vec::new())
                .sql,
            "((rt.\"gender_code\" = $1) IS NOT TRUE)"
        );
    }

    /// A mismatched shape means the schema and the search disagree about where
    /// the values are.
    #[test]
    fn a_mismatched_column_shape_is_rejected() {
        let target = direct(ParamColumns::String {
            value: "name".to_string(),
        });

        assert!(target.value_expr().is_ok());
        assert!(target.date_exprs().is_err());
    }

    #[test]
    fn rebase_shifts_placeholders() {
        let mut clause = SqlClause::new(
            "a = $1 AND b = $2".to_string(),
            vec![SqlParam::Text("a".into()), SqlParam::Text("b".into())],
        );
        clause.rebase(3);
        assert_eq!(clause.sql, "a = $4 AND b = $5");
    }
}
