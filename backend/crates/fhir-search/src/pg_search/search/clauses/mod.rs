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

use crate::pg_search::schema::ParamColumns;

/// Alias for the per-resource-type table in the generated query. Must match
/// the alias used by the JOIN in `build_final_query`.
pub const RESOURCE_TABLE_ALIAS: &str = "rt";

/// Where a search parameter's values live, and therefore how a clause has to
/// look them up.
///
/// Singular system-level (HL7 base) parameters get scalar columns on the
/// per-resource-type table, so their clauses read those columns directly.
/// Everything that may repeat shares the `{version}_param_{type}_idx` tables
/// and is discriminated by `param_url`.
#[derive(Debug, Clone)]
pub enum ClauseTarget {
    /// Columns read straight off a row: the resource type's table, or the
    /// anchor for a resource-level parameter. The alias says which.
    DirectColumn {
        alias: &'static str,
        columns: ParamColumns,
    },
    /// A row lookup in the shared table for this parameter's value type,
    /// discriminated by the parameter's canonical URL.
    Dynamic { table: String, param_url: String },
}

/// Builds the `EXISTS (SELECT 1 FROM {table} {alias} WHERE ...)` skeleton
/// shared by every clause reading a shared table: the correlation back to the
/// anchor row, plus the `param_url = $1` discriminator.
///
/// `predicate` is appended as an additional `AND (...)` when non-empty.
/// `negate` prefixes the whole thing with `NOT`.
#[must_use]
pub fn dynamic_exists(table: &str, alias: &str, negate: bool, predicate: Option<&str>) -> String {
    let prefix = if negate { "NOT " } else { "" };
    let extra = predicate.map_or_else(String::new, |p| format!(" AND ({p})"));

    format!(
        "{prefix}EXISTS (SELECT 1 FROM {table} {alias} \
         WHERE {alias}.tenant = sr.tenant AND {alias}.project = sr.project \
         AND {alias}.resource_type = sr.resource_type AND {alias}.resource_id = sr.resource_id \
         AND {alias}.param_url = $1{extra})"
    )
}

/// The anchor table's alias in a generated query. Resource-level parameters
/// read their columns from here, which is also the only table a search that
/// names no resource type has.
pub const ANCHOR_TABLE_ALIAS: &str = "sr";

/// A reference to a column on the table `alias` names.
#[must_use]
pub fn direct_column(alias: &str, column: &str) -> String {
    format!("{alias}.\"{column}\"")
}

/// Wraps a predicate written against a resource type table's columns.
///
/// No `unnest` and no `EXISTS`: a column exists only for a parameter that
/// cannot repeat, so the value is on the row and the predicate reads it
/// directly. That is also what makes the column's index usable — a predicate
/// over `unnest(column)` is a correlated function scan the planner cannot
/// answer from an index.
///
/// Negation is `IS NOT TRUE` rather than `NOT`, because a column with no value
/// makes the predicate NULL, and `:not` has to match a resource that does not
/// carry the value at all.
#[must_use]
pub fn direct_predicate(negate: bool, predicate: &str) -> String {
    if negate {
        format!("(({predicate}) IS NOT TRUE)")
    } else {
        format!("({predicate})")
    }
}

/// `:missing=true`/`false` against a direct column: a scalar column holds the
/// value or NULL, so absence is exactly NULL.
#[must_use]
pub fn direct_missing(alias: &str, column: &str, missing: bool) -> String {
    let col = direct_column(alias, column);
    if missing {
        format!("{col} IS NULL")
    } else {
        format!("{col} IS NOT NULL")
    }
}

/// A fragment of a SQL WHERE clause with its bind parameters.
///
/// The `sql` field contains a SQL expression that uses positional placeholders
/// like `${offset+1}`, `${offset+2}`, etc. Before execution, the caller
/// rebases these placeholders to the actual position in the final query.
///
/// The `params` vector holds the corresponding bind values in order.
#[derive(Debug, Clone)]
pub struct SqlClause {
    pub sql: String,
    pub params: Vec<SqlParam>,
}

/// A typed bind parameter for the SQL query.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum SqlParam {
    Text(String),
    Int64(i64),
    Float64(f64),
    Bool(bool),
    OptionalText(Option<String>),
}

impl SqlClause {
    /// Creates a new clause with positional placeholders starting at 1.
    /// The caller must rebase placeholder numbers before combining clauses.
    pub fn new(sql: String, params: Vec<SqlParam>) -> Self {
        SqlClause { sql, params }
    }

    /// Rebases all `$N` placeholders in the SQL by adding `offset` to each N.
    pub fn rebase(&mut self, offset: usize) {
        if offset == 0 {
            return;
        }
        // Replace $N with $(N+offset), working from highest N down to avoid
        // $1 being replaced inside $10.
        let mut rebased = self.sql.clone();
        for i in (1..=self.params.len()).rev() {
            let old = format!("${i}");
            let new = format!("${}", i + offset);
            rebased = rebased.replace(&old, &new);
        }
        self.sql = rebased;
    }
}

/// Resolves the `param_url` to use in dynamic (EAV) queries. Project-level
/// parameters are keyed by their canonical URL.
pub fn resolve_param_url(
    search_param: &haste_fhir_model::r4::generated::resources::SearchParameter,
) -> String {
    search_param.url.value.as_deref().unwrap_or("").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A scalar column is read straight off the row, which is the only shape
    /// its B-tree can serve.
    #[test]
    fn a_direct_predicate_reads_the_column_itself() {
        let sql = direct_predicate(
            false,
            &format!(
                "{} = $1 AND {} = $2",
                direct_column(RESOURCE_TABLE_ALIAS, "identifier_system"),
                direct_column(RESOURCE_TABLE_ALIAS, "identifier_code")
            ),
        );

        assert_eq!(
            sql,
            "(rt.\"identifier_system\" = $1 AND rt.\"identifier_code\" = $2)"
        );
        assert!(!sql.contains("unnest"));
    }

    /// `:not` has to match a resource carrying no value at all, and a NULL
    /// column makes the predicate NULL rather than false.
    #[test]
    fn a_negated_predicate_matches_a_missing_value() {
        let sql = direct_predicate(
            true,
            &format!(
                "{} = $1",
                direct_column(RESOURCE_TABLE_ALIAS, "gender_code")
            ),
        );

        assert_eq!(sql, "((rt.\"gender_code\" = $1) IS NOT TRUE)");
    }

    #[test]
    fn direct_missing_is_a_null_check() {
        assert_eq!(
            direct_missing(RESOURCE_TABLE_ALIAS, "name", true),
            "rt.\"name\" IS NULL"
        );
        assert_eq!(
            direct_missing(RESOURCE_TABLE_ALIAS, "name", false),
            "rt.\"name\" IS NOT NULL"
        );
    }

    #[test]
    fn dynamic_exists_correlates_and_discriminates() {
        let sql = dynamic_exists("r4_param_token_idx", "st", false, Some("st.code = $2"));
        assert!(sql.contains("FROM r4_param_token_idx st"));
        assert!(sql.contains("st.param_url = $1"));
        assert!(sql.ends_with("AND (st.code = $2))"));
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
