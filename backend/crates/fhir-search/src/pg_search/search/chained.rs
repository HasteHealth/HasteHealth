//! Chained search parameters, e.g. `Observation?subject:Patient.name=Smith`.
//!
//! Each reference followed is a correlated `EXISTS` over the referenced
//! resource's anchor row, nested one level per link, with the final
//! parameter's clause innermost:
//!
//! ```sql
//! EXISTS (SELECT 1 FROM r4_resource_idx c0 JOIN r4_patient ct0 ON ...
//!         WHERE c0.tenant = sr.tenant AND c0.project = sr.project
//!           AND c0.resource_type = 'Patient'
//!           AND (rt."subject_id" = c0.resource_id AND rt."subject_type" = 'Patient')
//!           AND (<final clause against c0 / ct0>))
//! ```
//!
//! `EXISTS`, rather than fetching the inner ids first, lets the planner pick
//! the join order from statistics and never needs a capped id list, which
//! would silently drop matches. On 2M Observations over 200k Patients a
//! selective chain answers a 50-row page in 771 buffer hits.

use std::{borrow::Cow, fmt::Write as _};

use haste_fhir_client::url::Parameter;
use haste_fhir_model::r4::generated::{
    resources::{ResourceType, SearchParameter},
    terminology::BoundCode,
};
use haste_fhir_operation_error::OperationOutcomeError;

use crate::{
    SearchParameterResolve,
    pg_search::{keys, schema::resource_table_name},
    query::QueryBuildError,
};

use super::{
    ClauseRow, SearchScope, clause_target,
    clauses::{
        ANCHOR_TABLE_ALIAS, SqlClause, SqlParam, rebase_placeholders, reference_exprs,
        target_params, wrap_predicate,
    },
    parameter_to_sql_clause, resolve_parameter,
};

/// How many references a chain may follow. Each is another join for the
/// planner to get right.
const MAX_CHAIN_DEPTH: usize = 3;

/// The clause for a chained parameter.
///
/// Levels are written outside-in into one bind list, so only the final
/// parameter's clause has its placeholders shifted.
pub(super) async fn chained_clause<ParameterResolver: SearchParameterResolve>(
    scope: &SearchScope<'_, ParameterResolver>,
    param: &Parameter,
) -> Result<SqlClause, OperationOutcomeError> {
    // `subject:Patient.organization.name` follows `subject:Patient` and
    // `organization`, then searches `name`.
    let Some((last, rest)) = param.chains.split_last() else {
        return Err(QueryBuildError::InvalidParameterValue(param.key()).into());
    };
    let depth = rest.len() + 1;
    if depth > MAX_CHAIN_DEPTH {
        return Err(QueryBuildError::UnsupportedParameter(format!(
            "'{}' follows {depth} references; at most {MAX_CHAIN_DEPTH} are supported",
            param.key()
        ))
        .into());
    }
    let links = std::iter::once((param.name.as_str(), param.modifier.as_deref())).chain(
        rest.iter()
            .map(|link| (link.name.as_str(), link.modifier.as_deref())),
    );

    let anchor_table = resource_table_name(&scope.registry.version);
    let scope_key = keys::scope_key(scope.tenant.as_ref(), scope.project.as_ref());

    let mut sql = String::new();
    let mut params: Vec<SqlParam> = Vec::new();
    // The row holding the next reference: the searched row, then each level.
    let mut row = scope.root_row();
    let mut resource_type = scope.resource_type.cloned();

    for (level, (name, modifier)) in links.enumerate() {
        let reference = resolve_parameter(scope, resource_type.as_ref(), name).await?;
        if reference.search_parameter.type_.as_str() != Some("reference") {
            return Err(QueryBuildError::UnsupportedParameter(format!(
                "'{name}' is not a reference, so it cannot be chained"
            ))
            .into());
        }
        let target_type = link_target_type(&reference.search_parameter, modifier)?;
        let target = target_type.as_ref();
        let alias = format!("c{level}");

        // The reference, read on the row holding it, names this level's row.
        // Ids are unique only within a type, so the type must match as well.
        // It is compared to a literal: correlating it to `{alias}.resource_type`
        // is not sargable and turns the seek into a sequential scan.
        let reference_target = clause_target(scope, &row, &reference);
        let (type_column, id_column) = reference_exprs(&reference_target)?;
        let correlation = wrap_predicate(
            &reference_target,
            false,
            &format!("{id_column} = {alias}.resource_id AND {type_column} = '{target}'"),
            target_params(&reference_target),
        );
        let correlation_sql =
            rebase_placeholders(&correlation.sql, correlation.params.len(), params.len());
        params.extend(correlation.params);

        // The type table holds the level's singular parameters.
        let schema = scope.registry.schemas.get(target);
        let type_alias = format!("ct{level}");
        let type_join = match schema {
            Some(schema) => {
                params.push(SqlParam::Int64(scope_key));
                format!(
                    " JOIN {table} {type_alias} ON {type_alias}.res_key = {alias}.res_key \
                     AND {type_alias}.scope = ${bind}",
                    table = schema.table_name,
                    bind = params.len(),
                )
            }
            None => String::new(),
        };

        // Tenant and project come from the searched row, so a chain cannot
        // reach another tenant's resources.
        let _ = write!(
            sql,
            "EXISTS (SELECT 1 FROM {anchor_table} {alias}{type_join} \
             WHERE {alias}.tenant = {ANCHOR_TABLE_ALIAS}.tenant \
             AND {alias}.project = {ANCHOR_TABLE_ALIAS}.project \
             AND {alias}.resource_type = '{target}' \
             AND {correlation_sql} AND ("
        );

        row = ClauseRow {
            anchor: Cow::Owned(alias),
            type_table: schema.map(|schema| (Cow::Owned(type_alias), schema)),
        };
        resource_type = Some(target_type);
    }

    // The final parameter, searched on the innermost level's row.
    let parameter = resolve_parameter(scope, resource_type.as_ref(), &last.name).await?;
    let target = clause_target(scope, &row, &parameter);
    let clause = parameter_to_sql_clause(
        &parameter,
        &target,
        &Parameter {
            name: last.name.clone(),
            value: param.value.clone(),
            modifier: last.modifier.clone(),
            chains: Vec::new(),
        },
    )?;
    sql.push_str(&rebase_placeholders(
        &clause.sql,
        clause.params.len(),
        params.len(),
    ));
    params.extend(clause.params);
    sql.push_str(&"))".repeat(depth));

    Ok(SqlClause { sql, params })
}

/// The type a reference is followed into: its `:Type` modifier, or its only
/// target. A modifier must be one of the declared targets, and a reference
/// with several targets needs one.
fn link_target_type(
    reference: &SearchParameter,
    modifier: Option<&str>,
) -> Result<ResourceType, QueryBuildError> {
    let code = reference.code.value.as_deref().unwrap_or_default();
    let targets: Vec<&str> = reference
        .target
        .iter()
        .flatten()
        .filter_map(BoundCode::as_str)
        .collect();

    let target = match (modifier, targets.as_slice()) {
        (Some(modifier), _) if targets.is_empty() || targets.contains(&modifier) => modifier,
        (Some(modifier), _) => {
            return Err(QueryBuildError::InvalidParameterValue(format!(
                "'{code}' cannot point at '{modifier}'"
            )));
        }
        (None, [only]) => only,
        (None, []) => {
            return Err(QueryBuildError::UnsupportedParameter(format!(
                "'{code}' declares no target type; name one as '{code}:<Type>'"
            )));
        }
        (None, _) => {
            return Err(QueryBuildError::UnsupportedParameter(format!(
                "'{code}' can point at several types; name one as '{code}:<Type>'"
            )));
        }
    };

    ResourceType::try_from(target).map_err(|_| {
        QueryBuildError::InvalidParameterValue(format!("'{target}' is not a resource type"))
    })
}
