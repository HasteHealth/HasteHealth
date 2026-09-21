use std::sync::Arc;

use haste_fhir_client::{
    request::SearchRequest,
    url::{Parameter, ParsedParameter, ParsedParameters},
};
use haste_fhir_model::r4::generated::{
    resources::ResourceType,
    terminology::{IssueType, SearchParamType},
};
use haste_fhir_operation_error::{OperationOutcomeError, derive::OperationOutcomeError};
use haste_jwt::{ProjectId, TenantId};
use sqlx::{Pool, Postgres, Row};

use crate::{
    ParameterLevel, ResolvedParameter, SearchEntry, SearchOptions, SearchParameterResolve,
    SearchReturn,
    pg_search::schema::{ParamColumns, ResourceTypeSchema, SchemaRegistry, SharedTable},
};

use clauses::{
    ANCHOR_TABLE_ALIAS, ClauseTarget, RESOURCE_TABLE_ALIAS, SqlClause, SqlParam, resolve_param_url,
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

    // A system-level search spans every resource type, so there is no single
    // per-resource-type table to join — those searches resolve entirely
    // through the dynamic EAV tables.
    let schema = resource_type.and_then(|rt| schema_registry.get(rt.as_ref()));

    let mut where_clauses: Vec<SqlClause> = Vec::new();
    let mut state = QueryState {
        max_count: get_max_count(options)?,
        offset: 0,
        show_total: false,
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

    // Process each search parameter.
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

    // Build and execute the query.
    let (sql, all_params) = build_final_query(
        &where_clauses,
        schema_registry,
        tenant,
        project,
        resource_type,
        schema,
        &state,
    );

    execute_sql(pool, &sql, &all_params, state.show_total).await
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
    show_total: bool,
    sort: Vec<SortEntry>,
}

struct SortEntry {
    target: ClauseTarget,
    param_type: String,
    direction: &'static str,
}

/// What every parameter of one search is resolved against: who is searching,
/// which resource type, and the schema its columns come from.
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
        clause_target(parameter, self.registry, self.schema)
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

/// Decides where a resolved parameter's values are read from.
///
/// A parameter reads its own column only when the search is scoped to a
/// resource type *and* the generated schema claimed a column for it — which
/// happens only for parameters known to produce a single value. Everything
/// else reads the shared table for its value type, which can serve any
/// parameter.
fn clause_target(
    parameter: &ResolvedParameter,
    registry: &SchemaRegistry,
    schema: Option<&ResourceTypeSchema>,
) -> ClauseTarget {
    let search_param = parameter.search_parameter.as_ref();

    let code = search_param.code.value.as_deref();

    if matches!(parameter.level, ParameterLevel::System)
        && let Some(code) = code
    {
        // Resource-level parameters are columns on the anchor, which every
        // search reads whether or not it names a resource type.
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

    // No column for it, so it resolves through the shared table for its
    // value type. A parameter type with no table there cannot be searched at
    // all, and the clause builder rejects it by name.
    let table = SharedTable::for_param_type(&search_param.type_)
        .map(|table| registry.shared_table_name(table))
        .unwrap_or_default();

    ClauseTarget::Dynamic {
        table,
        param_url: resolve_param_url(search_param),
    }
}

fn parameter_to_sql_clause(
    parameter: &ResolvedParameter,
    target: &ClauseTarget,
    parsed_parameter: &Parameter,
) -> Result<SqlClause, QueryBuildError> {
    let search_param = parameter.search_parameter.as_ref();
    match &search_param.type_ {
        t if t == &SearchParamType::string() => clauses::string_clause(parsed_parameter, target),
        t if t == &SearchParamType::token() => clauses::token_clause(parsed_parameter, target),
        t if t == &SearchParamType::date() => clauses::date_clause(parsed_parameter, target),
        t if t == &SearchParamType::number() => clauses::number_clause(parsed_parameter, target),
        t if t == &SearchParamType::quantity() => {
            clauses::quantity_clause(parsed_parameter, target)
        }
        t if t == &SearchParamType::reference() => {
            clauses::reference_clause(parsed_parameter, target)
        }
        t if t == &SearchParamType::uri() => clauses::uri_clause(parsed_parameter, target),
        _ => Err(QueryBuildError::UnsupportedParameter(
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
            state.show_total = match result_param
                .value
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>()
                .as_slice()
            {
                ["none"] => false,
                ["accurate" | "estimate"] => true,
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

                // Only date, string, and token are supported for sorting
                // (matches ES behavior).
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
            // Handled in middleware, not in the search query.
        }
        _ => {
            return Err(QueryBuildError::UnsupportedParameter(result_param.name.clone()).into());
        }
    }

    Ok(())
}

/// Assembles the final SQL query string and a flat params list from the
/// individual per-parameter clauses plus context (tenant, project, `resource_type`).
fn build_final_query(
    where_clauses: &[SqlClause],
    registry: &SchemaRegistry,
    tenant: &TenantId,
    project: &ProjectId,
    resource_type: Option<&ResourceType>,
    schema: Option<&ResourceTypeSchema>,
    state: &QueryState,
) -> (String, Vec<SqlParam>) {
    let mut all_params: Vec<SqlParam> = Vec::new();

    // Context params come first: tenant, project, and optionally resource_type.
    all_params.push(SqlParam::Text(tenant.as_ref().to_string()));
    all_params.push(SqlParam::Text(project.as_ref().to_string()));

    let mut context_where = String::from("sr.tenant = $1 AND sr.project = $2");

    if let Some(rt) = resource_type {
        all_params.push(SqlParam::Text(rt.as_ref().to_string()));
        context_where.push_str(" AND sr.resource_type = $3");
    }

    // Rebase each clause's placeholders and collect them.
    let mut clause_fragments = Vec::new();
    for clause in where_clauses {
        let mut rebased = clause.clone();
        rebased.rebase(all_params.len());
        clause_fragments.push(rebased.sql);
        all_params.extend(rebased.params);
    }

    // Build WHERE.
    let mut where_sql = context_where;
    for fragment in &clause_fragments {
        where_sql.push_str(" AND ");
        where_sql.push_str(fragment);
    }

    // Build ORDER BY.
    let order_by = build_order_by(&state.sort, &mut all_params);

    // The per-resource-type table is only joined when something actually
    // reads a column off it, so a query using only dynamic parameters stays a
    // single-table scan.
    let needs_join = schema.is_some()
        && (clause_fragments
            .iter()
            .any(|fragment| fragment.contains(RESOURCE_TABLE_ALIAS))
            || order_by.contains(RESOURCE_TABLE_ALIAS));

    let join_sql = if needs_join {
        // `schema` is Some by the `needs_join` guard above.
        schema.map_or_else(String::new, |schema| {
            format!(
                "JOIN {table} {RESOURCE_TABLE_ALIAS} \
                 ON {RESOURCE_TABLE_ALIAS}.tenant = sr.tenant \
                 AND {RESOURCE_TABLE_ALIAS}.project = sr.project \
                 AND {RESOURCE_TABLE_ALIAS}.resource_id = sr.resource_id ",
                table = schema.table_name,
            )
        })
    } else {
        String::new()
    };

    // Build the full query.
    let total_col = if state.show_total {
        ", COUNT(*) OVER() AS total_count"
    } else {
        ""
    };

    let limit_idx = all_params.len() + 1;
    let offset_idx = all_params.len() + 2;
    all_params.push(SqlParam::Int64(state.max_count.cast_signed()));
    all_params.push(SqlParam::Int64(state.offset.cast_signed()));

    let sql = format!(
        "SELECT sr.resource_id, sr.resource_type, sr.version_id, sr.project{total_col} \
         FROM {resource_table} sr \
         {join_sql}\
         WHERE {where_sql}\
         {order_by} \
         LIMIT ${limit_idx} OFFSET ${offset_idx}",
        resource_table = registry.resource_table_name(),
    );

    (sql, all_params)
}

/// Builds the ORDER BY clause.
///
/// A sort on a column reads it directly. A sort on a shared table uses a
/// correlated subquery picking the minimum value, so a resource with several
/// values sorts by its earliest/lowest one.
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

/// The scalar SQL expression a single sort entry orders by, or `None` when the
/// parameter's type has no sortable representation.
fn sort_expression(entry: &SortEntry, all_params: &mut Vec<SqlParam>) -> Option<String> {
    match &entry.target {
        ClauseTarget::DirectColumn { alias, columns } => {
            // A scalar column is the sort key itself, so the ORDER BY reads it
            // directly and a B-tree on it can supply the order — no per-row
            // subquery for the planner to compute over every candidate.
            let column = match (entry.param_type.as_str(), columns) {
                // Descending date sorts read the period's end so the latest
                // period wins, matching the ascending case reading its start.
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
        ClauseTarget::Dynamic { table, param_url } => {
            let idx = all_params.len() + 1;

            let subquery = match entry.param_type.as_str() {
                "date" => {
                    let col = if entry.direction == "ASC" {
                        "start_ms"
                    } else {
                        "end_ms"
                    };
                    format!(
                        "(SELECT MIN(sd.{col}) FROM {table} sd \
                         WHERE sd.tenant = sr.tenant AND sd.project = sr.project \
                         AND sd.resource_type = sr.resource_type AND sd.resource_id = sr.resource_id \
                         AND sd.param_url = ${idx})"
                    )
                }
                "string" => format!(
                    "(SELECT MIN(ss.value) FROM {table} ss \
                     WHERE ss.tenant = sr.tenant AND ss.project = sr.project \
                     AND ss.resource_type = sr.resource_type AND ss.resource_id = sr.resource_id \
                     AND ss.param_url = ${idx})"
                ),
                "token" => format!(
                    "(SELECT MIN(st.code) FROM {table} st \
                     WHERE st.tenant = sr.tenant AND st.project = sr.project \
                     AND st.resource_type = sr.resource_type AND st.resource_id = sr.resource_id \
                     AND st.param_url = ${idx})"
                ),
                _ => return None,
            };

            // Only bind once the subquery is known to use the placeholder.
            all_params.push(SqlParam::Text(param_url.clone()));
            Some(subquery)
        }
    }
}

async fn execute_sql(
    pool: &Pool<Postgres>,
    sql: &str,
    params: &[SqlParam],
    show_total: bool,
) -> Result<SearchReturn, OperationOutcomeError> {
    let mut query = sqlx::query(sql);

    for param in params {
        query = match param {
            SqlParam::Text(v) => query.bind(v.as_str()),
            SqlParam::Int64(v) => query.bind(*v),
            SqlParam::Float64(v) => query.bind(*v),
            SqlParam::Bool(v) => query.bind(*v),
            SqlParam::OptionalText(v) => query.bind(v.as_deref()),
        };
    }

    let rows = query.fetch_all(pool).await.map_err(|e| {
        OperationOutcomeError::fatal(
            IssueType::exception(),
            format!("PG search query failed: {e}"),
        )
    })?;

    let total = if show_total {
        rows.first()
            .and_then(|r| r.try_get::<i64, _>("total_count").ok())
    } else {
        None
    };

    let entries = rows
        .into_iter()
        .map(|row| {
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
        })
        .collect();

    Ok(SearchReturn { total, entries })
}
