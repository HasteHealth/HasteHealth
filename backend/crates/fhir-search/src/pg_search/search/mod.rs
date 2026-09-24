use std::sync::Arc;

use haste_fhir_client::{
    request::SearchRequest,
    url::{Parameter, ParsedParameter, ParsedParameters},
};
use haste_fhir_model::r4::generated::{resources::ResourceType, terminology::IssueType};
use haste_fhir_operation_error::{OperationOutcomeError, derive::OperationOutcomeError};
use haste_jwt::{ProjectId, TenantId};
use sqlx::{Pool, Postgres, Row, postgres::PgRow};

use crate::{
    ParameterLevel, ResolvedParameter, SearchEntry, SearchOptions, SearchParameterResolve,
    SearchReturn,
    pg_search::{
        keys,
        schema::{ParamColumns, ResourceTypeSchema, SchemaRegistry, SharedTable},
    },
};

use clauses::{
    ANCHOR_TABLE_ALIAS, ClauseTarget, RESOURCE_TABLE_ALIAS, SqlClause, SqlParam,
    resolve_param_identity,
};

pub(crate) mod clauses;

#[derive(OperationOutcomeError, Debug)]
pub enum QueryBuildError {
    #[error(
        code = "not-found",
        diagnostic = "Search parameter with name '{arg0}' not found.'"
    )]
    MissingParameter(String),
    #[error(code = "not-supported", diagnostic = "Unsupported parameter: '{arg0}'")]
    UnsupportedParameter(String),
    #[error(
        code = "not-supported",
        diagnostic = "Unsupported sorting parameter: '{arg0}'"
    )]
    UnsupportedSortParameter(String),
    #[error(
        code = "not-supported",
        diagnostic = "Unsupported modifier parameter: '{arg0}'"
    )]
    UnsupportedModifier(String),
    #[error(
        code = "not-supported",
        diagnostic = "Prefix '{arg0}' is not supported for this search type."
    )]
    UnsupportedPrefix(String),
    #[error(
        code = "not-supported",
        diagnostic = "Parameter value '{arg0}' is not supported for this search type."
    )]
    UnsupportedParameterValue(String),
    #[error(code = "invalid", diagnostic = "Invalid parameter value: '{arg0}'")]
    InvalidParameterValue(String),
    #[error(code = "invalid", diagnostic = "Invalid date format: '{arg0}'")]
    InvalidDateFormat(String),
    #[error(
        code = "not-supported",
        diagnostic = "Modifier '{arg0}' is not supported"
    )]
    ModifierNotSupported(String),
}

static ABSOLUTE_MAX: u64 = 10_000;
static DEFAULT_MAX_COUNT: u64 = 50;

fn get_resource_type(request: &SearchRequest) -> Option<&ResourceType> {
    match request {
        SearchRequest::Type(r) => Some(&r.resource_type),
        SearchRequest::System(_) => None,
    }
}

fn get_parameters(request: &SearchRequest) -> &ParsedParameters {
    match request {
        SearchRequest::Type(r) => &r.parameters,
        SearchRequest::System(r) => &r.parameters,
    }
}

pub async fn execute_search<ParameterResolver: SearchParameterResolve>(
    pool: &Pool<Postgres>,
    parameter_resolver: Arc<ParameterResolver>,
    schema_registry: &SchemaRegistry,
    tenant: &TenantId,
    project: &ProjectId,
    search_request: &SearchRequest,
    options: Option<&SearchOptions>,
) -> Result<SearchReturn, OperationOutcomeError> {
    let resource_type = get_resource_type(search_request);
    let parameters = get_parameters(search_request);

    // A system-level search spans every resource type, so it has no
    // per-resource-type table to join and resolves through the shared tables.
    let schema = resource_type.and_then(|rt| schema_registry.get(rt.as_ref()));

    let mut where_clauses: Vec<SqlClause> = Vec::new();
    let mut state = QueryState {
        max_count: get_max_count(options)?,
        offset: 0,
        estimate_total: false,
        sort: Vec::new(),
    };

    let scope = SearchScope {
        parameter_resolver: &parameter_resolver,
        registry: schema_registry,
        schema,
        tenant,
        project,
        resource_type,
    };

    for parameter in parameters.parameters() {
        match parameter {
            ParsedParameter::Resource(resource_param) => {
                where_clauses.push(build_resource_clause(&scope, resource_param).await?);
            }
            ParsedParameter::Result(result_param) => {
                handle_result_parameter(&scope, result_param, &mut state).await?;
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

    let page = execute_sql(pool, &query.page_sql, &query.params);

    let (rows, estimate) = if state.estimate_total {
        // Planning only, on its own connection, so it overlaps the page query.
        let (rows, estimate) = tokio::try_join!(
            page,
            estimate_rows(pool, &query.estimate_sql, query.estimate_params())
        )?;
        (rows, Some(estimate))
    } else {
        (page.await?, None)
    };

    let total = estimate
        .map(|estimate| refine_estimate(estimate, rows.len(), state.offset, state.max_count));

    Ok(SearchReturn {
        total,
        entries: rows.iter().map(search_entry).collect(),
    })
}

/// The planner's estimate, corrected by what the page proves: a short page is
/// the end and so exact, a full page raises a low estimate, and a page past the
/// end caps the total at the offset.
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

fn get_max_count(options: Option<&SearchOptions>) -> Result<u64, OperationOutcomeError> {
    if let Some(count_limit) = options.as_ref().and_then(|o| o.count_limit) {
        if count_limit > ABSOLUTE_MAX {
            return Err(OperationOutcomeError::fatal(
                IssueType::too_costly(),
                "Count limit exceeds maximum allowed.".to_string(),
            ));
        }
        Ok(count_limit)
    } else {
        Ok(DEFAULT_MAX_COUNT)
    }
}

struct QueryState {
    max_count: u64,
    offset: u64,
    /// Whether to return a total. Always the planner's estimate, tightened by
    /// the page (see [`refine_estimate`]): that costs a plan, where an exact
    /// count has to find every match.
    estimate_total: bool,
    sort: Vec<SortEntry>,
}

struct SortEntry {
    target: ClauseTarget,
    param_type: String,
    direction: &'static str,
}

/// What one search's parameters resolve against: who is searching, which
/// resource type, and the schema its columns come from.
struct SearchScope<'a, ParameterResolver> {
    parameter_resolver: &'a Arc<ParameterResolver>,
    registry: &'a SchemaRegistry,
    /// The resource type's table, or `None` for a system-level search.
    schema: Option<&'a ResourceTypeSchema>,
    tenant: &'a TenantId,
    project: &'a ProjectId,
    resource_type: Option<&'a ResourceType>,
}

impl<ParameterResolver: SearchParameterResolve> SearchScope<'_, ParameterResolver> {
    /// The search parameter `name` refers to in this scope.
    async fn resolve(&self, name: &str) -> Result<ResolvedParameter, OperationOutcomeError> {
        Ok(self
            .parameter_resolver
            .by_name(self.tenant, self.project, self.resource_type, name)
            .await?
            .ok_or_else(|| QueryBuildError::MissingParameter(name.to_string()))?)
    }

    fn clause_target(&self, parameter: &ResolvedParameter) -> ClauseTarget {
        clause_target(
            parameter,
            self.registry,
            self.schema,
            self.tenant,
            self.project,
        )
    }
}

async fn build_resource_clause<ParameterResolver: SearchParameterResolve>(
    scope: &SearchScope<'_, ParameterResolver>,
    resource_param: &Parameter,
) -> Result<SqlClause, OperationOutcomeError> {
    let parameter = scope.resolve(&resource_param.name).await?;
    let target = scope.clause_target(&parameter);

    Ok(parameter_to_sql_clause(
        &parameter,
        &target,
        resource_param,
    )?)
}

/// Where a resolved parameter's values are read from.
///
/// A parameter gets its own column only when the search names a resource type
/// and the schema claimed a column for it, which happens only for single-valued
/// parameters. Everything else reads the shared table for its value type.
fn clause_target(
    parameter: &ResolvedParameter,
    registry: &SchemaRegistry,
    schema: Option<&ResourceTypeSchema>,
    tenant: &TenantId,
    project: &ProjectId,
) -> ClauseTarget {
    let search_param = parameter.search_parameter.as_ref();

    let code = search_param.code.value.as_deref();

    if matches!(parameter.level, ParameterLevel::System)
        && let Some(code) = code
    {
        // Resource-level parameters are anchor columns, which every search
        // reads whether or not it names a resource type.
        if let Some(columns) = registry.anchor().columns_for(code) {
            return ClauseTarget::DirectColumn {
                alias: ANCHOR_TABLE_ALIAS,
                columns: columns.clone(),
            };
        }

        if let Some(columns) = schema.and_then(|schema| schema.columns_for(code)) {
            return ClauseTarget::DirectColumn {
                alias: RESOURCE_TABLE_ALIAS,
                columns: columns.clone(),
            };
        }
    }

    // No column, so the shared table for its value type. A type with no table
    // there is rejected by name in the clause builder.
    let table = SharedTable::for_param_type(&search_param.type_)
        .map(|table| registry.shared_table_name(table))
        .unwrap_or_default();

    ClauseTarget::Dynamic {
        table,
        param_identity: resolve_param_identity(search_param, tenant.as_ref(), project.as_ref()),
    }
}

fn parameter_to_sql_clause(
    parameter: &ResolvedParameter,
    target: &ClauseTarget,
    parsed_parameter: &Parameter,
) -> Result<SqlClause, QueryBuildError> {
    let search_param = parameter.search_parameter.as_ref();

    // The mapping that decides where values are written decides which clause
    // reads them back.
    match SharedTable::for_param_type(&search_param.type_) {
        Some(SharedTable::String) => clauses::string_clause(parsed_parameter, target),
        Some(SharedTable::Token) => clauses::token_clause(parsed_parameter, target),
        Some(SharedTable::Date) => clauses::date_clause(parsed_parameter, target),
        Some(SharedTable::Number) => clauses::number_clause(parsed_parameter, target),
        Some(SharedTable::Quantity) => clauses::quantity_clause(parsed_parameter, target),
        Some(SharedTable::Reference) => clauses::reference_clause(parsed_parameter, target),
        Some(SharedTable::Uri) => clauses::uri_clause(parsed_parameter, target),
        None => Err(QueryBuildError::UnsupportedParameter(
            search_param.name.value.clone().unwrap_or_default(),
        )),
    }
}

async fn handle_result_parameter<ParameterResolver: SearchParameterResolve>(
    scope: &SearchScope<'_, ParameterResolver>,
    result_param: &Parameter,
    state: &mut QueryState,
) -> Result<(), OperationOutcomeError> {
    match result_param.name.as_str() {
        "_count" => {
            let v = result_param.value.first().ok_or_else(|| {
                OperationOutcomeError::error(
                    IssueType::required(),
                    format!("Missing parameter value: {}", result_param.name),
                )
            })?;
            state.max_count = v.parse::<u64>().map_err(|_| {
                OperationOutcomeError::fatal(
                    IssueType::invalid(),
                    format!("Invalid _count value: '{v}'."),
                )
            })?;
        }
        "_offset" => {
            let v = result_param.value.first().ok_or_else(|| {
                OperationOutcomeError::error(
                    IssueType::required(),
                    format!("Missing parameter value: {}", result_param.name),
                )
            })?;
            state.offset = v.parse::<u64>().map_err(|_| {
                OperationOutcomeError::fatal(
                    IssueType::invalid(),
                    format!("Invalid _offset value: '{v}'."),
                )
            })?;
        }
        "_total" => {
            // `accurate` gets the estimate too: an exact count scans every
            // match.
            state.estimate_total = match result_param
                .value
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>()
                .as_slice()
            {
                ["none"] => false,
                ["estimate" | "accurate"] => true,
                _ => {
                    return Err(
                        QueryBuildError::InvalidParameterValue(result_param.name.clone()).into(),
                    );
                }
            };
        }
        "_sort" => {
            for sort_value in &result_param.value {
                let param_name = sort_value.strip_prefix('-').unwrap_or(sort_value);
                let direction = if sort_value.starts_with('-') {
                    "DESC"
                } else {
                    "ASC"
                };

                let parameter = scope.resolve(param_name).await?;

                let sp = parameter.search_parameter.as_ref();
                let param_type = sp.type_.as_str().unwrap_or("string").to_string();

                // Sortable types, matching ES.
                match param_type.as_str() {
                    "date" | "string" | "token" => {}
                    _ => {
                        return Err(QueryBuildError::UnsupportedSortParameter(
                            param_name.to_string(),
                        )
                        .into());
                    }
                }

                state.sort.push(SortEntry {
                    target: scope.clause_target(&parameter),
                    param_type,
                    direction,
                });
            }
        }
        "_summary" | "_elements" => {
            // Handled in middleware.
        }
        _ => {
            return Err(QueryBuildError::UnsupportedParameter(result_param.name.clone()).into());
        }
    }

    Ok(())
}

/// The statements one search runs, sharing one parameter list.
struct FinalQuery {
    /// The page of matches, in order.
    page_sql: String,
    /// `EXPLAIN` of the same matches, unordered and unlimited; its top node's
    /// row estimate answers `_total=estimate`.
    estimate_sql: String,
    /// The page's binds; the estimate's are a prefix. A prepared statement
    /// takes exactly as many as its highest placeholder, so the page-only ones
    /// (sort, limit) come last for the estimate to drop.
    params: Vec<SqlParam>,
    estimate_param_count: usize,
}

impl FinalQuery {
    fn estimate_params(&self) -> &[SqlParam] {
        &self.params[..self.estimate_param_count]
    }
}

/// Assembles the per-parameter clauses and the scope (tenant, project,
/// `resource_type`) into the statements and their flat bind list.
fn build_final_query(
    where_clauses: &[SqlClause],
    registry: &SchemaRegistry,
    tenant: &TenantId,
    project: &ProjectId,
    resource_type: Option<&ResourceType>,
    schema: Option<&ResourceTypeSchema>,
    state: &QueryState,
) -> FinalQuery {
    let mut all_params: Vec<SqlParam> = Vec::new();

    // Scope binds come first.
    all_params.push(SqlParam::Text(tenant.as_ref().to_string()));
    all_params.push(SqlParam::Text(project.as_ref().to_string()));

    let mut context_where = String::from("sr.tenant = $1 AND sr.project = $2");

    if let Some(rt) = resource_type {
        all_params.push(SqlParam::Text(rt.as_ref().to_string()));
        context_where.push_str(" AND sr.resource_type = $3");
    }

    let mut clause_fragments = Vec::new();
    for clause in where_clauses {
        let mut rebased = clause.clone();
        rebased.rebase(all_params.len());
        clause_fragments.push(rebased.sql);
        all_params.extend(rebased.params);
    }

    let mut where_sql = context_where;
    for fragment in &clause_fragments {
        where_sql.push_str(" AND ");
        where_sql.push_str(fragment);
    }

    // Join the per-resource-type table only when something reads a column off
    // it, so a purely dynamic query stays a single-table scan. Read from the
    // sort entries, not the built ORDER BY, so the join's bind comes first.
    let needs_join = schema.is_some()
        && (clause_fragments
            .iter()
            .any(|fragment| fragment.contains(RESOURCE_TABLE_ALIAS))
            || state.sort.iter().any(|entry| {
                matches!(
                    entry.target,
                    ClauseTarget::DirectColumn { alias, .. } if alias == RESOURCE_TABLE_ALIAS
                )
            }));

    // The scope repeats the anchor's tenant and project, but it leads every
    // index on the resource type table, so stating it lets a predicate there be
    // answered from the index before the join.
    let join_sql = match schema {
        Some(schema) if needs_join => {
            all_params.push(SqlParam::Int64(keys::scope_key(
                tenant.as_ref(),
                project.as_ref(),
            )));
            format!(
                "JOIN {table} {RESOURCE_TABLE_ALIAS} \
                 ON {RESOURCE_TABLE_ALIAS}.res_key = sr.res_key \
                 AND {RESOURCE_TABLE_ALIAS}.scope = ${scope_idx} ",
                table = schema.table_name,
                scope_idx = all_params.len(),
            )
        }
        _ => String::new(),
    };

    // Everything the estimate reads is bound by now.
    let estimate_param_count = all_params.len();

    let order_by = build_order_by(&state.sort, &mut all_params);

    let limit_idx = all_params.len() + 1;
    let offset_idx = all_params.len() + 2;
    all_params.push(SqlParam::Int64(state.max_count.cast_signed()));
    all_params.push(SqlParam::Int64(state.offset.cast_signed()));

    let resource_table = registry.resource_table_name();

    let page_sql = format!(
        "SELECT sr.resource_id, sr.resource_type, sr.version_id, sr.project \
         FROM {resource_table} sr \
         {join_sql}\
         WHERE {where_sql}\
         {order_by} \
         LIMIT ${limit_idx} OFFSET ${offset_idx}",
    );

    let estimate_sql =
        format!("EXPLAIN SELECT 1 FROM {resource_table} sr {join_sql}WHERE {where_sql}");

    FinalQuery {
        page_sql,
        estimate_sql,
        params: all_params,
        estimate_param_count,
    }
}

/// A sort on a column reads it directly; a sort on a shared table uses a
/// correlated `MIN` subquery, so a resource sorts by its lowest value.
fn build_order_by(sort_entries: &[SortEntry], all_params: &mut Vec<SqlParam>) -> String {
    if sort_entries.is_empty() {
        return String::new();
    }

    let mut parts = Vec::new();
    for entry in sort_entries {
        let Some(expr) = sort_expression(entry, all_params) else {
            continue;
        };
        parts.push(format!("{expr} {} NULLS LAST", entry.direction));
    }

    if parts.is_empty() {
        return String::new();
    }

    format!(" ORDER BY {}", parts.join(", "))
}

/// What one sort entry orders by, or `None` when its type is not sortable.
fn sort_expression(entry: &SortEntry, all_params: &mut Vec<SqlParam>) -> Option<String> {
    match &entry.target {
        ClauseTarget::DirectColumn { alias, columns } => {
            // The column is the sort key, so a B-tree on it can supply the
            // order outright — no per-row subquery.
            let column = match (entry.param_type.as_str(), columns) {
                // Descending reads the period's end so the latest wins,
                // mirroring ascending reading its start.
                ("date", ParamColumns::Date { start, end }) => {
                    if entry.direction == "ASC" {
                        start
                    } else {
                        end
                    }
                }
                ("string", ParamColumns::String { value } | ParamColumns::Uri { value })
                | ("number", ParamColumns::Number { value }) => value,
                (
                    "token",
                    ParamColumns::Token { code, .. } | ParamColumns::Quantity { code, .. },
                ) => code,
                _ => return None,
            };

            Some(format!("{alias}.\"{column}\""))
        }
        ClauseTarget::Dynamic {
            table,
            param_identity,
        } => {
            let idx = all_params.len() + 1;

            let subquery = match entry.param_type.as_str() {
                "date" => {
                    let col = if entry.direction == "ASC" {
                        "start_ms"
                    } else {
                        "end_ms"
                    };
                    format!(
                        "(SELECT MIN(v.{col}) FROM {table} v \
                         WHERE v.res_key = sr.res_key AND v.param_identity = ${idx})"
                    )
                }
                "string" => format!(
                    "(SELECT MIN(v.value) FROM {table} v \
                     WHERE v.res_key = sr.res_key AND v.param_identity = ${idx})"
                ),
                "token" => format!(
                    "(SELECT MIN(v.code) FROM {table} v \
                     WHERE v.res_key = sr.res_key AND v.param_identity = ${idx})"
                ),
                _ => return None,
            };

            // Bound only now that the subquery is known to use it.
            all_params.push(SqlParam::Int64(*param_identity));
            Some(subquery)
        }
    }
}

fn bind_params<'q>(sql: &'q str, params: &'q [SqlParam]) -> PgQuery<'q> {
    let mut query = sqlx::query(sql);

    for param in params {
        query = match param {
            SqlParam::Text(v) => query.bind(v.as_str()),
            SqlParam::Int64(v) => query.bind(*v),
            SqlParam::Float64(v) => query.bind(*v),
        };
    }

    query
}

type PgQuery<'q> = sqlx::query::Query<'q, Postgres, sqlx::postgres::PgArguments>;

async fn execute_sql(
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

/// The row estimate from the top node of `sql`, an `EXPLAIN`:
/// `... (cost=0.42..3522.78 rows=133596 width=4)`.
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

fn parse_plan_rows(plan_line: &str) -> Option<i64> {
    let rest = &plan_line[plan_line.find(" rows=")? + " rows=".len()..];
    let digits = rest.split(|c: char| !c.is_ascii_digit()).next()?;
    digits.parse().ok()
}

fn search_entry(row: &PgRow) -> SearchEntry {
    let resource_id: String = row.get("resource_id");
    let resource_type_str: String = row.get("resource_type");
    let version_id: String = row.get("version_id");
    let project: String = row.get("project");

    SearchEntry {
        id: haste_jwt::ResourceId::new(resource_id),
        resource_type: resource_type_str
            .as_str()
            .try_into()
            .expect("Invalid resource type in search index"),
        version_id: haste_jwt::VersionId::new(version_id),
        project: ProjectId::new(project),
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

    /// `Patient?_total=estimate&_sort=-_lastUpdated,-address-city`: the
    /// estimate drops the ORDER BY and LIMIT, so binding their parameters would
    /// be rejected outright.
    #[tokio::test]
    async fn each_statement_binds_exactly_its_placeholders() {
        let registry = registry().await;
        let patient = registry.get("Patient").expect("Patient schema");
        let last_updated = registry
            .anchor()
            .columns_for("_lastUpdated")
            .expect("_lastUpdated column")
            .clone();
        let birthdate = patient
            .columns_for("birthdate")
            .expect("birthdate column")
            .clone();

        let dynamic = || ClauseTarget::Dynamic {
            table: registry.shared_table_name(SharedTable::String),
            param_identity: 7,
        };

        // Without, then with the per-resource-type join, whose scope bind the
        // estimate does read.
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
                &[SqlClause::new(
                    format!("{ANCHOR_TABLE_ALIAS}.\"resource_id\" = $1"),
                    vec![SqlParam::Text("a".to_string())],
                )],
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
                query.estimate_params().len(),
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

    /// A short page is the end of the results, so it is the exact total.
    #[test]
    fn a_short_page_is_exact() {
        assert_eq!(refine_estimate(5_000, 7, 0, 20), 7);
        assert_eq!(refine_estimate(5_000, 7, 40, 20), 47);
        assert_eq!(refine_estimate(5_000, 0, 0, 20), 0);
    }

    /// A full page proves at least that many, whatever the planner thought.
    #[test]
    fn a_full_page_raises_a_low_estimate() {
        assert_eq!(refine_estimate(10, 20, 100, 20), 120);
        assert_eq!(refine_estimate(133_596, 20, 0, 20), 133_596);
    }

    /// Past the end nothing comes back, and the total cannot exceed the offset.
    #[test]
    fn a_page_past_the_end_caps_the_estimate() {
        assert_eq!(refine_estimate(5_000, 0, 200, 20), 200);
        assert_eq!(refine_estimate(150, 0, 200, 20), 150);
    }
}
