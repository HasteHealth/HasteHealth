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
/// System-level (HL7 base) parameters get dedicated array columns on the
/// per-resource-type table, so their clauses read those columns directly.
/// Project-level parameters share the EAV `search_dynamic_*` tables and are
/// discriminated by `param_url`.
#[derive(Debug, Clone)]
pub enum ClauseTarget {
    /// Direct array columns on the per-resource-type table, aliased
    /// [`RESOURCE_TABLE_ALIAS`].
    DirectColumn(ParamColumns),
    /// EAV lookup against the `search_dynamic_*` table for this type.
    Dynamic { param_url: String },
}

/// Builds the `EXISTS (SELECT 1 FROM search_dynamic_{suffix} {alias} WHERE ...)`
/// skeleton shared by every dynamic clause: the correlation back to
/// `search_resource`, plus the `param_url = $1` discriminator.
///
/// `predicate` is appended as an additional `AND (...)` when non-empty.
/// `negate` prefixes the whole thing with `NOT`.
#[must_use]
pub fn dynamic_exists(suffix: &str, alias: &str, negate: bool, predicate: Option<&str>) -> String {
    let prefix = if negate { "NOT " } else { "" };
    let extra = predicate.map_or_else(String::new, |p| format!(" AND ({p})"));

    format!(
        "{prefix}EXISTS (SELECT 1 FROM search_dynamic_{suffix} {alias} \
         WHERE {alias}.tenant = sr.tenant AND {alias}.project = sr.project \
         AND {alias}.resource_type = sr.resource_type AND {alias}.resource_id = sr.resource_id \
         AND {alias}.param_url = $1{extra})"
    )
}

/// Builds an `EXISTS (SELECT 1 FROM unnest(...) ... WHERE predicate)` over one
/// or more parallel array columns on the per-resource-type table.
///
/// `bindings` pairs each column name with the alias its unnested value takes
/// in `predicate`. Passing more than one column unnests them in parallel, so
/// index `i` of every column is visible in the same row — which is what makes
/// `system`/`code` and `start`/`end` pairs line up.
#[must_use]
pub fn direct_exists(bindings: &[(&str, &str)], negate: bool, predicate: &str) -> String {
    let columns = bindings
        .iter()
        .map(|(column, _)| format!("{RESOURCE_TABLE_ALIAS}.\"{column}\""))
        .collect::<Vec<_>>()
        .join(", ");

    let aliases = bindings
        .iter()
        .map(|(_, alias)| (*alias).to_string())
        .collect::<Vec<_>>()
        .join(", ");

    let prefix = if negate { "NOT " } else { "" };

    format!("{prefix}EXISTS (SELECT 1 FROM unnest({columns}) AS u({aliases}) WHERE {predicate})")
}

/// `:missing=true`/`false` against a direct column: a parameter is missing
/// when its array column is NULL or empty.
#[must_use]
pub fn direct_missing(column: &str, missing: bool) -> String {
    let col = format!("{RESOURCE_TABLE_ALIAS}.\"{column}\"");
    if missing {
        format!("({col} IS NULL OR cardinality({col}) = 0)")
    } else {
        format!("({col} IS NOT NULL AND cardinality({col}) > 0)")
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

    #[test]
    fn direct_exists_unnests_parallel_columns() {
        let sql = direct_exists(
            &[("identifier_system", "sys"), ("identifier_code", "cod")],
            false,
            "sys = $1 AND cod = $2",
        );
        assert_eq!(
            sql,
            "EXISTS (SELECT 1 FROM unnest(rt.\"identifier_system\", rt.\"identifier_code\") \
             AS u(sys, cod) WHERE sys = $1 AND cod = $2)"
        );
    }

    #[test]
    fn direct_exists_negates() {
        let sql = direct_exists(&[("name", "v")], true, "v = $1");
        assert!(sql.starts_with("NOT EXISTS (SELECT 1 FROM unnest(rt.\"name\") AS u(v)"));
    }

    #[test]
    fn direct_missing_checks_null_and_empty() {
        assert_eq!(
            direct_missing("name", true),
            "(rt.\"name\" IS NULL OR cardinality(rt.\"name\") = 0)"
        );
        assert_eq!(
            direct_missing("name", false),
            "(rt.\"name\" IS NOT NULL AND cardinality(rt.\"name\") > 0)"
        );
    }

    #[test]
    fn dynamic_exists_correlates_and_discriminates() {
        let sql = dynamic_exists("token", "st", false, Some("st.code = $2"));
        assert!(sql.contains("FROM search_dynamic_token st"));
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
