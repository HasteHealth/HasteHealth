use std::sync::Arc;

use haste_fhir_client::{
    request::SearchRequest,
    url::{Parameter, ParsedParameter, ParsedParameters},
};
use haste_fhir_model::r4::generated::{resources::ResourceType, terminology::IssueType};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_jwt::{ProjectId, TenantId};
use sqlx::{Pool, Postgres, Row, postgres::PgRow};

use crate::{
    ParameterLevel, ResolvedParameter, SearchEntry, SearchOptions, SearchParameterResolve,
    SearchReturn,
    pg_search::{
        keys,
        schema::{
            ParamColumns, ResourceTypeSchema, SchemaRegistry, SharedTable, resource_table_name,
            shared_table_for, shared_table_name,
        },
    },
    query::{Modifier, QueryBuildError, parse_modifier},
};

use clauses::{
    ANCHOR_TABLE_ALIAS, ClauseTarget, RESOURCE_TABLE_ALIAS, SqlClause, SqlParam, bind,
    rebase_placeholders, resolve_param_identity,
};

pub(crate) mod clauses;

static ABSOLUTE_MAX: u64 = 10_000;
static DEFAULT_MAX_COUNT: u64 = 50;

/// Paging, total and sort settings from the result parameters.
struct QueryState {
    max_count: u64,
    offset: u64,
    /// Return a total. Always the planner's estimate refined by the page (see
    /// [`refine_estimate`]); an exact count would scan every match.
    estimate_total: bool,
    sort: Vec<SortEntry>,
}

struct SortEntry {
    target: ClauseTarget,
    param_type: String,
    direction: &'static str,
}

/// What a search's parameters resolve against.
struct SearchScope<'a, ParameterResolver> {
    parameter_resolver: &'a Arc<ParameterResolver>,
    registry: &'a SchemaRegistry,
    /// The resource type's table; `None` for a system-level search.
    schema: Option<&'a ResourceTypeSchema>,
    tenant: &'a TenantId,
    project: &'a ProjectId,
    resource_type: Option<&'a ResourceType>,
}

/// The statements one search runs, sharing one bind list.
struct FinalQuery {
    /// The ordered, limited page.
    page_sql: String,
    /// `EXPLAIN` of the same filter, unordered and unlimited; its top row
    /// estimate answers `_total`.
    estimate_sql: String,
    /// The page's binds. The estimate uses only the first
    /// `estimate_param_count`, so sort and limit binds must come last.
    params: Vec<SqlParam>,
    estimate_param_count: usize,
}

type PgQuery<'q> = sqlx::query::Query<'q, Postgres, sqlx::postgres::PgArguments>;

pub async fn execute_search<ParameterResolver: SearchParameterResolve>(
    pool: &Pool<Postgres>,
    parameter_resolver: Arc<ParameterResolver>,
    schema_registry: &SchemaRegistry,
    tenant: &TenantId,
    project: &ProjectId,
    search_request: &SearchRequest,
    options: Option<&SearchOptions>,
) -> Result<SearchReturn, OperationOutcomeError> {
    let (resource_type, parameters) = request_parts(search_request);
    // A system-level search has no type table and reads the shared tables.
    let schema = resource_type.and_then(|rt| schema_registry.schemas.get(rt.as_ref()));

    let scope = SearchScope {
        parameter_resolver: &parameter_resolver,
        registry: schema_registry,
        schema,
        tenant,
        project,
        resource_type,
    };

    let mut where_clauses = Vec::new();
    let mut state = QueryState {
        max_count: max_count(options)?,
        offset: 0,
        estimate_total: false,
        sort: Vec::new(),
    };
    for parameter in parameters.parameters() {
        match parameter {
            ParsedParameter::Resource(param) => {
                where_clauses.push(resource_clause(&scope, param).await?);
            }
            ParsedParameter::Result(param) => {
                state = apply_result_parameter(&scope, param, state).await?;
            }
        }
    }

    let query = build_final_query(
        &where_clauses,
        schema_registry,
        tenant,
        project,
        resource_type,
        schema,
        &state,
    );

    let page = fetch_page(pool, &query.page_sql, &query.params);
    let (rows, estimate) = if state.estimate_total {
        // The estimate only plans, so run it concurrently with the page.
        let estimate_params = &query.params[..query.estimate_param_count];
        let (rows, estimate) = tokio::try_join!(
            page,
            estimate_rows(pool, &query.estimate_sql, estimate_params)
        )?;
        (rows, Some(estimate))
    } else {
        (page.await?, None)
    };

    Ok(SearchReturn {
        total: estimate
            .map(|estimate| refine_estimate(estimate, rows.len(), state.offset, state.max_count)),
        entries: rows.iter().map(search_entry).collect(),
    })
}

fn request_parts(request: &SearchRequest) -> (Option<&ResourceType>, &ParsedParameters) {
    match request {
        SearchRequest::Type(r) => (Some(&r.resource_type), &r.parameters),
        SearchRequest::System(r) => (None, &r.parameters),
    }
}

/// Corrects the planner's estimate with what the page proves: a short page is
/// exact, a full page is a lower bound, and an empty page past the start caps
/// the total at the offset.
fn refine_estimate(estimate: i64, returned: usize, offset: u64, limit: u64) -> i64 {
    let offset = offset.cast_signed();
    let returned = i64::try_from(returned).unwrap_or(i64::MAX);

    if returned == 0 && offset > 0 {
        return estimate.min(offset);
    }

    let seen = offset.saturating_add(returned);
    if returned < limit.cast_signed() {
        seen
    } else {
        estimate.max(seen)
    }
}

fn max_count(options: Option<&SearchOptions>) -> Result<u64, OperationOutcomeError> {
    match options.and_then(|o| o.count_limit) {
        Some(limit) if limit > ABSOLUTE_MAX => Err(OperationOutcomeError::fatal(
            IssueType::too_costly(),
            "Count limit exceeds maximum allowed.".to_string(),
        )),
        Some(limit) => Ok(limit),
        None => Ok(DEFAULT_MAX_COUNT),
    }
}

async fn resolve_parameter<ParameterResolver: SearchParameterResolve>(
    scope: &SearchScope<'_, ParameterResolver>,
    name: &str,
) -> Result<ResolvedParameter, OperationOutcomeError> {
    Ok(scope
        .parameter_resolver
        .by_name(scope.tenant, scope.project, scope.resource_type, name)
        .await?
        .ok_or_else(|| QueryBuildError::MissingParameter(name.to_string()))?)
}

async fn resource_clause<ParameterResolver: SearchParameterResolve>(
    scope: &SearchScope<'_, ParameterResolver>,
    param: &Parameter,
) -> Result<SqlClause, OperationOutcomeError> {
    let parameter = resolve_parameter(scope, &param.name).await?;
    let target = clause_target(scope, &parameter);
    Ok(parameter_to_sql_clause(&parameter, &target, param)?)
}

/// Where a parameter's values are read from: an anchor column, a type table
/// column (typed searches only), or otherwise the shared table for its type.
fn clause_target<ParameterResolver>(
    scope: &SearchScope<'_, ParameterResolver>,
    parameter: &ResolvedParameter,
) -> ClauseTarget {
    let search_param = parameter.search_parameter.as_ref();

    let column = search_param
        .code
        .value
        .as_deref()
        .filter(|_| matches!(parameter.level, ParameterLevel::System))
        .and_then(|code| {
            scope
                .registry
                .anchor
                .parameters
                .get(code)
                .map(|columns| (ANCHOR_TABLE_ALIAS, columns))
                .or_else(|| {
                    scope
                        .schema?
                        .parameters
                        .get(code)
                        .map(|columns| (RESOURCE_TABLE_ALIAS, columns))
                })
        });

    match column {
        Some((alias, columns)) => ClauseTarget::DirectColumn {
            alias,
            columns: columns.clone(),
        },
        // An unmapped type gets an empty table name and is rejected by
        // `parameter_to_sql_clause`.
        None => ClauseTarget::Dynamic {
            table: shared_table_for(&search_param.type_)
                .map(|table| shared_table_name(&scope.registry.version, table))
                .unwrap_or_default(),
            param_identity: resolve_param_identity(
                search_param,
                scope.tenant.as_ref(),
                scope.project.as_ref(),
            ),
        },
    }
}

fn parameter_to_sql_clause(
    parameter: &ResolvedParameter,
    target: &ClauseTarget,
    param: &Parameter,
) -> Result<SqlClause, QueryBuildError> {
    let search_param = parameter.search_parameter.as_ref();
    let modifier = parse_modifier(param, &search_param.type_)?;

    // `:missing` works the same for every stored type.
    if let Modifier::Missing(missing) = modifier {
        return Ok(clauses::missing_clause(target, missing));
    }

    // The same mapping that chose where values are written.
    match shared_table_for(&search_param.type_) {
        Some(SharedTable::String) => clauses::string_clause(param, target, modifier),
        Some(SharedTable::Token) => clauses::token_clause(param, target, modifier == Modifier::Not),
        Some(SharedTable::Date) => clauses::date_clause(param, target),
        Some(SharedTable::Number) => clauses::number_clause(param, target),
        Some(SharedTable::Quantity) => clauses::quantity_clause(param, target),
        Some(SharedTable::Reference) => clauses::reference_clause(param, target),
        Some(SharedTable::Uri) => clauses::uri_clause(param, target),
        None => Err(QueryBuildError::UnsupportedParameter(
            search_param.name.value.clone().unwrap_or_default(),
        )),
    }
}

/// Returns `state` updated by one result parameter (`_count`, `_sort`, ...).
async fn apply_result_parameter<ParameterResolver: SearchParameterResolve>(
    scope: &SearchScope<'_, ParameterResolver>,
    param: &Parameter,
    state: QueryState,
) -> Result<QueryState, OperationOutcomeError> {
    Ok(match param.name.as_str() {
        "_count" => QueryState {
            max_count: parse_u64(param)?,
            ..state
        },
        "_offset" => QueryState {
            offset: parse_u64(param)?,
            ..state
        },
        // `accurate` also gets the estimate; an exact count scans every match.
        "_total" => QueryState {
            estimate_total: match param.value.iter().map(String::as_str).collect::<Vec<_>>()[..] {
                ["none"] => false,
                ["estimate" | "accurate"] => true,
                _ => return Err(QueryBuildError::InvalidParameterValue(param.name.clone()).into()),
            },
            ..state
        },
        "_sort" => {
            let mut sort = state.sort;
            for value in &param.value {
                sort.push(sort_entry(scope, value).await?);
            }
            QueryState { sort, ..state }
        }
        // Handled in middleware.
        "_summary" | "_elements" => state,
        _ => return Err(QueryBuildError::UnsupportedParameter(param.name.clone()).into()),
    })
}

fn parse_u64(param: &Parameter) -> Result<u64, OperationOutcomeError> {
    let value = param.value.first().ok_or_else(|| {
        OperationOutcomeError::error(
            IssueType::required(),
            format!("Missing parameter value: {}", param.name),
        )
    })?;
    value.parse().map_err(|_| {
        OperationOutcomeError::fatal(
            IssueType::invalid(),
            format!("Invalid {} value: '{value}'.", param.name),
        )
    })
}

/// One `_sort` value: `name` ascending, `-name` descending.
async fn sort_entry<ParameterResolver: SearchParameterResolve>(
    scope: &SearchScope<'_, ParameterResolver>,
    value: &str,
) -> Result<SortEntry, OperationOutcomeError> {
    let (name, direction) = match value.strip_prefix('-') {
        Some(name) => (name, "DESC"),
        None => (value, "ASC"),
    };

    let parameter = resolve_parameter(scope, name).await?;
    let param_type = parameter
        .search_parameter
        .type_
        .as_str()
        .unwrap_or("string")
        .to_string();

    // Same sortable types as Elasticsearch.
    if !matches!(param_type.as_str(), "date" | "string" | "token") {
        return Err(QueryBuildError::UnsupportedSortParameter(name.to_string()).into());
    }

    Ok(SortEntry {
        target: clause_target(scope, &parameter),
        param_type,
        direction,
    })
}

/// Combines the scope filter and parameter clauses into the page and estimate
/// statements.
fn build_final_query(
    where_clauses: &[SqlClause],
    registry: &SchemaRegistry,
    tenant: &TenantId,
    project: &ProjectId,
    resource_type: Option<&ResourceType>,
    schema: Option<&ResourceTypeSchema>,
    state: &QueryState,
) -> FinalQuery {
    let text = |value: &str| SqlParam::Text(value.to_string());

    // Scope binds come first.
    let (scope_filter, scope_params) = match resource_type {
        Some(rt) => (
            "sr.tenant = $1 AND sr.project = $2 AND sr.resource_type = $3",
            vec![
                text(tenant.as_ref()),
                text(project.as_ref()),
                text(rt.as_ref()),
            ],
        ),
        None => (
            "sr.tenant = $1 AND sr.project = $2",
            vec![text(tenant.as_ref()), text(project.as_ref())],
        ),
    };

    let (fragments, params) = where_clauses.iter().fold(
        (Vec::with_capacity(where_clauses.len()), scope_params),
        |(mut fragments, mut params), clause| {
            fragments.push(rebase_placeholders(
                &clause.sql,
                clause.params.len(),
                params.len(),
            ));
            params.extend(clause.params.iter().cloned());
            (fragments, params)
        },
    );

    let where_sql = std::iter::once(scope_filter)
        .chain(fragments.iter().map(String::as_str))
        .collect::<Vec<_>>()
        .join(" AND ");

    // Join the type table only if a clause or sort reads it. Checked on the sort
    // entries, not the ORDER BY, so the join's bind precedes the sort's.
    let reads_type_table = fragments
        .iter()
        .any(|fragment| fragment.contains(RESOURCE_TABLE_ALIAS))
        || state.sort.iter().any(|entry| {
            matches!(
                entry.target,
                ClauseTarget::DirectColumn { alias, .. } if alias == RESOURCE_TABLE_ALIAS
            )
        });

    // `scope` repeats the anchor's tenant and project, but it leads every type
    // table index, so the predicate there can use the index before the join.
    let (join_sql, params) = match schema {
        Some(schema) if reads_type_table => {
            let scope_key = keys::scope_key(tenant.as_ref(), project.as_ref());
            let (params, i) = bind(params, [SqlParam::Int64(scope_key)]);
            let join = format!(
                "JOIN {table} {RESOURCE_TABLE_ALIAS} \
                 ON {RESOURCE_TABLE_ALIAS}.res_key = sr.res_key \
                 AND {RESOURCE_TABLE_ALIAS}.scope = ${i} ",
                table = schema.table_name,
            );
            (join, params)
        }
        _ => (String::new(), params),
    };

    // Everything the estimate reads is bound by now.
    let estimate_param_count = params.len();

    let (order_by, params) = build_order_by(&state.sort, params);
    let (params, limit_idx) = bind(
        params,
        [
            SqlParam::Int64(state.max_count.cast_signed()),
            SqlParam::Int64(state.offset.cast_signed()),
        ],
    );
    let offset_idx = limit_idx + 1;

    let resource_table = resource_table_name(&registry.version);

    FinalQuery {
        page_sql: format!(
            "SELECT sr.resource_id, sr.resource_type, sr.version_id, sr.project \
             FROM {resource_table} sr \
             {join_sql}\
             WHERE {where_sql}\
             {order_by} \
             LIMIT ${limit_idx} OFFSET ${offset_idx}",
        ),
        estimate_sql: format!(
            "EXPLAIN SELECT 1 FROM {resource_table} sr {join_sql}WHERE {where_sql}"
        ),
        params,
        estimate_param_count,
    }
}

/// ` ORDER BY ...`, or empty when nothing is sortable.
fn build_order_by(sort_entries: &[SortEntry], params: Vec<SqlParam>) -> (String, Vec<SqlParam>) {
    let (parts, params) = sort_entries.iter().fold(
        (Vec::with_capacity(sort_entries.len()), params),
        |(mut parts, params), entry| {
            let (expression, params) = sort_expression(entry, params);
            parts.extend(expression.map(|e| format!("{e} {} NULLS LAST", entry.direction)));
            (parts, params)
        },
    );

    if parts.is_empty() {
        (String::new(), params)
    } else {
        (format!(" ORDER BY {}", parts.join(", ")), params)
    }
}

/// The expression one sort entry orders by, or `None` if its type isn't
/// sortable. A column sorts directly (so a B-tree can supply the order); a
/// shared table sorts by the resource's lowest value via a `MIN` subquery.
fn sort_expression(entry: &SortEntry, params: Vec<SqlParam>) -> (Option<String>, Vec<SqlParam>) {
    let ascending = entry.direction == "ASC";

    match &entry.target {
        ClauseTarget::DirectColumn { alias, columns } => {
            let column = match (entry.param_type.as_str(), columns) {
                // Ascending reads the period's start, descending its end.
                ("date", ParamColumns::Date { start, end }) => {
                    Some(if ascending { start } else { end })
                }
                ("string", ParamColumns::String { value } | ParamColumns::Uri { value })
                | ("number", ParamColumns::Number { value }) => Some(value),
                (
                    "token",
                    ParamColumns::Token { code, .. } | ParamColumns::Quantity { code, .. },
                ) => Some(code),
                _ => None,
            };
            (column.map(|column| format!("{alias}.\"{column}\"")), params)
        }
        ClauseTarget::Dynamic {
            table,
            param_identity,
        } => {
            let column = match entry.param_type.as_str() {
                "date" if ascending => "start_ms",
                "date" => "end_ms",
                "string" => "value",
                "token" => "code",
                _ => return (None, params),
            };
            // Bound only once the subquery is known to use it.
            let (params, i) = bind(params, [SqlParam::Int64(*param_identity)]);
            let subquery = format!(
                "(SELECT MIN(v.{column}) FROM {table} v \
                 WHERE v.res_key = sr.res_key AND v.param_identity = ${i})"
            );
            (Some(subquery), params)
        }
    }
}

fn bind_params<'q>(sql: &'q str, params: &'q [SqlParam]) -> PgQuery<'q> {
    params
        .iter()
        .fold(sqlx::query(sql), |query, param| match param {
            SqlParam::Text(v) => query.bind(v.as_str()),
            SqlParam::Int64(v) => query.bind(*v),
            SqlParam::Float64(v) => query.bind(*v),
        })
}

async fn fetch_page(
    pool: &Pool<Postgres>,
    sql: &str,
    params: &[SqlParam],
) -> Result<Vec<PgRow>, OperationOutcomeError> {
    bind_params(sql, params).fetch_all(pool).await.map_err(|e| {
        OperationOutcomeError::fatal(
            IssueType::exception(),
            format!("PG search query failed: {e}"),
        )
    })
}

/// Row estimate from the top line of an `EXPLAIN`.
async fn estimate_rows(
    pool: &Pool<Postgres>,
    sql: &str,
    params: &[SqlParam],
) -> Result<i64, OperationOutcomeError> {
    let top: String = bind_params(sql, params)
        .fetch_one(pool)
        .await
        .map_err(|e| {
            OperationOutcomeError::fatal(
                IssueType::exception(),
                format!("PG search estimate failed: {e}"),
            )
        })?
        .get(0);

    parse_plan_rows(&top).ok_or_else(|| {
        OperationOutcomeError::fatal(
            IssueType::exception(),
            format!("PG search estimate has no row count: '{top}'"),
        )
    })
}

/// Reads `rows=N` from a plan line like
/// `... (cost=0.42..3522.78 rows=133596 width=4)`.
fn parse_plan_rows(plan_line: &str) -> Option<i64> {
    let rest = &plan_line[plan_line.find(" rows=")? + " rows=".len()..];
    rest.split(|c: char| !c.is_ascii_digit())
        .next()?
        .parse()
        .ok()
}

fn search_entry(row: &PgRow) -> SearchEntry {
    let resource_type: String = row.get("resource_type");

    SearchEntry {
        id: haste_jwt::ResourceId::new(row.get("resource_id")),
        resource_type: resource_type
            .as_str()
            .try_into()
            .expect("Invalid resource type in search index"),
        version_id: haste_jwt::VersionId::new(row.get("version_id")),
        project: ProjectId::new(row.get("project")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pg_search::schema::generate_schemas;
    use haste_repository::types::SupportedFHIRVersions;

    /// The highest `$n` in `sql`: how many binds Postgres expects.
    fn highest_placeholder(sql: &str) -> usize {
        sql.split('$')
            .skip(1)
            .filter_map(|rest| {
                let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
                digits.parse().ok()
            })
            .max()
            .unwrap_or(0)
    }

    async fn registry() -> SchemaRegistry {
        let parameters = crate::memory::R4_SEARCH_PARAMETERS_INDEX
            .all(&TenantId::System, &ProjectId::System)
            .await
            .expect("system parameters resolve");
        generate_schemas(SupportedFHIRVersions::R4, &parameters)
    }

    fn sort(target: ClauseTarget, param_type: &str) -> SortEntry {
        SortEntry {
            target,
            param_type: param_type.to_string(),
            direction: "DESC",
        }
    }

    /// The estimate drops ORDER BY and LIMIT, so it must bind exactly its own
    /// placeholders or Postgres rejects it.
    #[tokio::test]
    async fn each_statement_binds_exactly_its_placeholders() {
        let registry = registry().await;
        let patient = registry.schemas.get("Patient").expect("Patient schema");
        let last_updated = registry
            .anchor
            .parameters
            .get("_lastUpdated")
            .expect("_lastUpdated column")
            .clone();
        let birthdate = patient
            .parameters
            .get("birthdate")
            .expect("birthdate column")
            .clone();

        let dynamic = || ClauseTarget::Dynamic {
            table: shared_table_name(&registry.version, SharedTable::String),
            param_identity: 7,
        };

        // Without, then with, the type table join (whose bind the estimate
        // does read).
        for sorts in [
            vec![
                sort(
                    ClauseTarget::DirectColumn {
                        alias: ANCHOR_TABLE_ALIAS,
                        columns: last_updated.clone(),
                    },
                    "date",
                ),
                sort(dynamic(), "string"),
            ],
            vec![
                sort(
                    ClauseTarget::DirectColumn {
                        alias: RESOURCE_TABLE_ALIAS,
                        columns: birthdate.clone(),
                    },
                    "date",
                ),
                sort(dynamic(), "string"),
            ],
        ] {
            let state = QueryState {
                max_count: 20,
                offset: 0,
                estimate_total: true,
                sort: sorts,
            };
            let query = build_final_query(
                &[SqlClause {
                    sql: format!("{ANCHOR_TABLE_ALIAS}.\"resource_id\" = $1"),
                    params: vec![SqlParam::Text("a".to_string())],
                }],
                &registry,
                &TenantId::new("t".to_string()),
                &ProjectId::new("p".to_string()),
                Some(&ResourceType::Patient),
                Some(patient),
                &state,
            );

            assert_eq!(
                highest_placeholder(&query.page_sql),
                query.params.len(),
                "{}",
                query.page_sql
            );
            assert_eq!(
                highest_placeholder(&query.estimate_sql),
                query.estimate_param_count,
                "{}",
                query.estimate_sql
            );
        }
    }

    #[test]
    fn plan_rows_are_read_from_the_top_node() {
        assert_eq!(
            parse_plan_rows(
                "Index Only Scan using idx on r4_resource_idx sr  \
                 (cost=0.42..3522.78 rows=133596 width=4)"
            ),
            Some(133_596)
        );
        assert_eq!(
            parse_plan_rows("Result  (cost=0.00..0.01 rows=1 width=4)"),
            Some(1)
        );
        assert_eq!(parse_plan_rows("no estimate here"), None);
    }

    #[test]
    fn a_short_page_is_exact() {
        assert_eq!(refine_estimate(5_000, 7, 0, 20), 7);
        assert_eq!(refine_estimate(5_000, 7, 40, 20), 47);
        assert_eq!(refine_estimate(5_000, 0, 0, 20), 0);
    }

    #[test]
    fn a_full_page_raises_a_low_estimate() {
        assert_eq!(refine_estimate(10, 20, 100, 20), 120);
        assert_eq!(refine_estimate(133_596, 20, 0, 20), 133_596);
    }

    #[test]
    fn a_page_past_the_end_caps_the_estimate() {
        assert_eq!(refine_estimate(5_000, 0, 200, 20), 200);
        assert_eq!(refine_estimate(150, 0, 200, 20), 150);
    }
}
