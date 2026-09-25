//! Table and column layout for the PG search backend.
//!
//! Where a parameter is stored depends on whether it can repeat:
//!
//! - **Singular**: a scalar column on its resource type's table
//!   (`r4_patient_idx`). Scalars support ordered comparisons, prefix matches,
//!   sorts and composite indexes; arrays only support overlap.
//! - **Repeating**: one row per value in the shared table for its type
//!   (`r4_param_token_idx`, ...), keyed by canonical URL. Same indexes, at the
//!   cost of a join.
//!
//! Every table references its resource by `res_key`, a `BIGINT` allocated by
//! the anchor (`r4_resource_idx`). Only the anchor stores tenant, project, type
//! and id, which keeps the other indexes small.
//!
//! Cardinality is precomputed in [`crate::search_parameter_cardinality`].
//! Anything not known to be singular goes to the shared tables, since a scalar
//! column would silently drop extra values.

use std::collections::{HashMap, HashSet};

use haste_fhir_model::r4::generated::terminology::{BoundCode, SearchParamType};
use haste_repository::types::SupportedFHIRVersions;

use crate::{ParameterLevel, ResolvedParameter, search_parameter_cardinality};

/// A generated column's type. Always scalar: repeating values live in rows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColumnType {
    Text,
    BigInt,
    Double,
}

/// How a column is indexed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndexKind {
    /// Not indexed; filtered on the row its indexed sibling finds.
    None,
    /// Plain B-tree for equality and range comparisons.
    BTree,
    /// B-tree stored `DESC NULLS LAST`, so a descending sort can stop at the
    /// page size. (A plain B-tree read backwards is `DESC NULLS FIRST`.)
    BTreeDescending,
    /// B-tree over `LOWER(column) text_pattern_ops`, for case-insensitive
    /// prefix matches.
    LoweredPrefix,
}

/// One generated column.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColumnDef {
    pub name: String,
    pub column_type: ColumnType,
    pub index: IndexKind,
}

/// The columns backing one parameter. Multi-part values keep their parts on the
/// same row, so e.g. a token's system stays paired with its code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParamColumns {
    String {
        value: String,
    },
    /// `system` is `None` only for `_id`, which reads the anchor's
    /// `resource_id`.
    Token {
        system: Option<String>,
        code: String,
    },
    Date {
        start: String,
        end: String,
    },
    Number {
        value: String,
    },
    Uri {
        value: String,
    },
    Reference {
        target_type: String,
        target_id: String,
    },
    Quantity {
        start: String,
        end: String,
        system: String,
        code: String,
    },
}

/// A value column of a [`SharedTable`]. Shared by the DDL and the batch insert.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SharedColumn {
    pub name: &'static str,
    /// Valid both in `CREATE TABLE` and as a `$n::<sql_type>[]` cast.
    pub sql_type: &'static str,
    pub nullable: bool,
}

/// The shared table for one value type, holding repeating parameters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SharedTable {
    String,
    Token,
    Date,
    Number,
    Quantity,
    Uri,
    Reference,
}

/// Every shared table, in creation order.
pub const SHARED_TABLES: [SharedTable; 7] = [
    SharedTable::String,
    SharedTable::Token,
    SharedTable::Date,
    SharedTable::Number,
    SharedTable::Quantity,
    SharedTable::Uri,
    SharedTable::Reference,
];

/// One resource type's table of singular parameters.
#[derive(Debug, Clone, Default)]
pub struct ResourceTypeSchema {
    /// e.g. `"Patient"`.
    pub resource_type: String,
    /// e.g. `"r4_patient_idx"`.
    pub table_name: String,
    /// Parameter `code` → its columns.
    pub parameters: HashMap<String, ParamColumns>,
    /// All value columns, sorted by name.
    pub columns: Vec<ColumnDef>,
}

/// Every generated table layout for one FHIR version.
#[derive(Debug, Clone, Default)]
pub struct SchemaRegistry {
    pub version: SupportedFHIRVersions,
    /// The anchor's columns: parameters on `Resource`/`DomainResource`
    /// (`_lastUpdated`, `_tag`, ...). Stored once here instead of on every
    /// type table, so untyped searches can read them too.
    pub anchor: ResourceTypeSchema,
    /// Keyed by resource type name.
    pub schemas: HashMap<String, ResourceTypeSchema>,
}

/// SQL type for `CREATE TABLE` / `ADD COLUMN`.
#[must_use]
pub const fn sql_type(column_type: ColumnType) -> &'static str {
    match column_type {
        ColumnType::Text => "TEXT",
        ColumnType::BigInt => "BIGINT",
        ColumnType::Double => "DOUBLE PRECISION",
    }
}

/// Every column a parameter occupies, in insert order.
#[must_use]
pub fn column_names(columns: &ParamColumns) -> Vec<&str> {
    match columns {
        ParamColumns::String { value }
        | ParamColumns::Number { value }
        | ParamColumns::Uri { value } => vec![value.as_str()],
        ParamColumns::Token { system, code } => system
            .iter()
            .map(String::as_str)
            .chain(std::iter::once(code.as_str()))
            .collect(),
        ParamColumns::Date { start, end } => vec![start.as_str(), end.as_str()],
        ParamColumns::Reference {
            target_type,
            target_id,
        } => vec![target_type.as_str(), target_id.as_str()],
        ParamColumns::Quantity {
            start,
            end,
            system,
            code,
        } => vec![start.as_str(), end.as_str(), system.as_str(), code.as_str()],
    }
}

const fn shared_column(name: &'static str, sql_type: &'static str, nullable: bool) -> SharedColumn {
    SharedColumn {
        name,
        sql_type,
        nullable,
    }
}

/// The value type a shared table holds.
#[must_use]
pub const fn shared_table_kind(table: SharedTable) -> &'static str {
    match table {
        SharedTable::String => "string",
        SharedTable::Token => "token",
        SharedTable::Date => "date",
        SharedTable::Number => "number",
        SharedTable::Quantity => "quantity",
        SharedTable::Uri => "uri",
        SharedTable::Reference => "reference",
    }
}

/// A shared table's value columns, after the `res_key, param_identity` keys.
/// The DDL and the inserts both read this list, so they cannot drift.
#[must_use]
pub const fn shared_value_columns(table: SharedTable) -> &'static [SharedColumn] {
    match table {
        SharedTable::String | SharedTable::Uri => {
            const COLUMNS: &[SharedColumn] = &[shared_column("value", "text", false)];
            COLUMNS
        }
        SharedTable::Token => {
            // A Coding may have only a system or only a code.
            const COLUMNS: &[SharedColumn] = &[
                shared_column("system", "text", true),
                shared_column("code", "text", true),
            ];
            COLUMNS
        }
        SharedTable::Date => {
            const COLUMNS: &[SharedColumn] = &[
                shared_column("start_ms", "int8", false),
                shared_column("end_ms", "int8", false),
            ];
            COLUMNS
        }
        SharedTable::Number => {
            const COLUMNS: &[SharedColumn] = &[shared_column("value", "float8", false)];
            COLUMNS
        }
        SharedTable::Quantity => {
            const COLUMNS: &[SharedColumn] = &[
                shared_column("start_value", "float8", false),
                shared_column("end_value", "float8", false),
                shared_column("start_system", "text", true),
                shared_column("start_code", "text", true),
            ];
            COLUMNS
        }
        SharedTable::Reference => {
            // A type with no id, or an absolute URL, leaves one half empty.
            const COLUMNS: &[SharedColumn] = &[
                shared_column("target_resource_type", "text", true),
                shared_column("target_id", "text", true),
            ];
            COLUMNS
        }
    }
}

/// `r4_param_token_idx`. The `param_` segment can't clash with a type table,
/// since no resource type name contains an underscore.
#[must_use]
pub fn shared_table_name(version: &SupportedFHIRVersions, table: SharedTable) -> String {
    format!("{version}_param_{}_idx", shared_table_kind(table))
}

/// The shared table for a parameter type; `None` for composite and special.
#[must_use]
pub fn shared_table_for(param_type: &BoundCode<SearchParamType>) -> Option<SharedTable> {
    [
        (SearchParamType::string(), SharedTable::String),
        (SearchParamType::token(), SharedTable::Token),
        (SearchParamType::date(), SharedTable::Date),
        (SearchParamType::number(), SharedTable::Number),
        (SearchParamType::quantity(), SharedTable::Quantity),
        (SearchParamType::uri(), SharedTable::Uri),
        (SearchParamType::reference(), SharedTable::Reference),
    ]
    .into_iter()
    .find_map(|(candidate, table)| (param_type == &candidate).then_some(table))
}

/// `r4_resource_idx`: the anchor, one row per indexed resource.
#[must_use]
pub fn resource_table_name(version: &SupportedFHIRVersions) -> String {
    format!("{version}_resource_idx")
}

/// `r4_patient_idx`: one table per resource type.
#[must_use]
pub fn resource_type_to_table(version: &SupportedFHIRVersions, resource_type: &str) -> String {
    format!("{version}_{}_idx", resource_type.to_ascii_lowercase())
}

/// Position of `name` in `schema.columns` (sorted, so binary search).
#[must_use]
pub fn column_index(schema: &ResourceTypeSchema, name: &str) -> Option<usize> {
    schema
        .columns
        .binary_search_by(|column| column.name.as_str().cmp(name))
        .ok()
}

/// `_id` reads the anchor's `resource_id` primary key instead of a column.
const RESOURCE_ID_CODE: &str = "_id";

/// Bases whose parameters apply to every resource, so live on the anchor.
const UNIVERSAL_BASES: [&str; 2] = ["Resource", "DomainResource"];

/// Fixed columns; a parameter whose name lands on one uses the shared tables.
const RESERVED_COLUMNS: [&str; 8] = [
    "res_key",
    "scope",
    "tenant",
    "project",
    "resource_type",
    "resource_id",
    "version_id",
    "sequence",
];

/// Folds a `code` to `[a-z0-9_]`, so a column name never needs escaping.
fn code_to_column_base(code: &str) -> String {
    code.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect()
}

/// Column names for a singular parameter of `param_type`. Derived from the
/// same mapping as the shared tables, so both shapes always agree.
fn columns_for_type(base: &str, param_type: &BoundCode<SearchParamType>) -> Option<ParamColumns> {
    let part = |suffix: &str| format!("{base}{suffix}");

    Some(match shared_table_for(param_type)? {
        SharedTable::String => ParamColumns::String {
            value: base.to_string(),
        },
        SharedTable::Uri => ParamColumns::Uri {
            value: base.to_string(),
        },
        SharedTable::Number => ParamColumns::Number {
            value: base.to_string(),
        },
        SharedTable::Token => ParamColumns::Token {
            system: Some(part("_system")),
            code: part("_code"),
        },
        SharedTable::Date => ParamColumns::Date {
            start: part("_start"),
            end: part("_end"),
        },
        SharedTable::Reference => ParamColumns::Reference {
            target_type: part("_type"),
            target_id: part("_id"),
        },
        SharedTable::Quantity => ParamColumns::Quantity {
            start: part("_start"),
            end: part("_end"),
            system: part("_system"),
            code: part("_code"),
        },
    })
}

/// Column definitions for a parameter. Of a pair, only the half queries filter
/// on is indexed (token code, reference id); the other is checked on the row.
fn column_defs(columns: &ParamColumns) -> Vec<ColumnDef> {
    let def = |name: &String, column_type, index| ColumnDef {
        name: name.clone(),
        column_type,
        index,
    };
    let text = |name, index| def(name, ColumnType::Text, index);

    match columns {
        ParamColumns::String { value } => vec![text(value, IndexKind::LoweredPrefix)],
        ParamColumns::Uri { value } => vec![text(value, IndexKind::BTree)],
        ParamColumns::Number { value } => vec![def(value, ColumnType::Double, IndexKind::BTree)],
        ParamColumns::Token { system, code } => system
            .iter()
            .map(|system| text(system, IndexKind::None))
            .chain(std::iter::once(text(code, IndexKind::BTree)))
            .collect(),
        ParamColumns::Date { start, end } => vec![
            def(start, ColumnType::BigInt, IndexKind::BTree),
            // Only descending sorts read the end.
            def(end, ColumnType::BigInt, IndexKind::BTreeDescending),
        ],
        ParamColumns::Reference {
            target_type,
            target_id,
        } => vec![
            text(target_type, IndexKind::None),
            text(target_id, IndexKind::BTree),
        ],
        ParamColumns::Quantity {
            start,
            end,
            system,
            code,
        } => vec![
            def(start, ColumnType::Double, IndexKind::BTree),
            def(end, ColumnType::Double, IndexKind::BTree),
            text(system, IndexKind::None),
            text(code, IndexKind::None),
        ],
    }
}

/// The `(code, url, columns)` a system-level parameter with an expression and
/// a column-mappable type would use. Cardinality is checked per base by the
/// caller.
fn singular_columns(parameter: &ResolvedParameter) -> Option<(&str, &str, ParamColumns)> {
    if !matches!(parameter.level, ParameterLevel::System) {
        return None;
    }

    let search_parameter = &parameter.search_parameter;
    let code = search_parameter.code.value.as_deref()?;
    search_parameter
        .expression
        .as_ref()
        .and_then(|e| e.value.as_deref())?;
    let url = search_parameter.url.value.as_deref()?;
    let columns = columns_for_type(&code_to_column_base(code), &search_parameter.type_)?;

    Some((code, url, columns))
}

/// Lays out one table. On a column name clash (with a fixed column or an
/// earlier parameter) the later parameter falls back to the shared tables.
fn build_table_schema(
    table_name: String,
    resource_type: String,
    entries: &[(String, ParamColumns)],
) -> ResourceTypeSchema {
    let reserved: HashSet<&str> = RESERVED_COLUMNS.into_iter().collect();

    let (_, parameters) = entries.iter().fold(
        (reserved, Vec::with_capacity(entries.len())),
        |(mut claimed, mut accepted), (code, columns)| {
            let names = column_names(columns);
            if names.iter().any(|name| claimed.contains(name)) {
                tracing::warn!(
                    "PG search: '{code}' on '{table_name}' collides with an existing column; \
                     it resolves through the shared tables instead.",
                );
            } else {
                claimed.extend(names);
                accepted.push((code, columns));
            }
            (claimed, accepted)
        },
    );

    let mut columns: Vec<ColumnDef> = parameters
        .iter()
        .flat_map(|(_, columns)| column_defs(columns))
        .collect();
    columns.sort_by(|a, b| a.name.cmp(&b.name));

    ResourceTypeSchema {
        table_name,
        resource_type,
        parameters: parameters
            .into_iter()
            .map(|(code, columns)| (code.clone(), columns.clone()))
            .collect(),
        columns,
    }
}

/// Builds every resource type table from the singular system parameters, plus
/// the anchor's columns for the resource-level ones.
#[must_use]
pub fn generate_schemas(
    version: SupportedFHIRVersions,
    parameters: &[ResolvedParameter],
) -> SchemaRegistry {
    // (base, code, columns) for every base where the parameter is singular.
    // Checked per base: a parameter can repeat on one type but not another.
    let placements = parameters
        .iter()
        .filter_map(|parameter| Some((parameter, singular_columns(parameter)?)))
        .flat_map(|(parameter, (code, url, columns))| {
            parameter
                .search_parameter
                .base
                .iter()
                .filter_map(BoundCode::as_str)
                .filter(move |base| search_parameter_cardinality::is_single_valued(url, base))
                // `_id` is the anchor's own key, added below.
                .filter(move |base| !(code == RESOURCE_ID_CODE && UNIVERSAL_BASES.contains(base)))
                .map(move |base| (base, code, columns.clone()))
        });

    // First occurrence of each code per table wins.
    let (universal, per_type) = placements.fold(
        (
            Vec::<(String, ParamColumns)>::new(),
            HashMap::<String, Vec<(String, ParamColumns)>>::new(),
        ),
        |(mut universal, mut per_type), (base, code, columns)| {
            let entries = if UNIVERSAL_BASES.contains(&base) {
                &mut universal
            } else {
                per_type.entry(base.to_string()).or_default()
            };
            if !entries.iter().any(|(existing, _)| existing == code) {
                entries.push((code.to_string(), columns));
            }
            (universal, per_type)
        },
    );

    let mut anchor = build_table_schema(
        resource_table_name(&version),
        "Resource".to_string(),
        &universal,
    );
    anchor.parameters.insert(
        RESOURCE_ID_CODE.to_string(),
        ParamColumns::Token {
            system: None,
            code: "resource_id".to_string(),
        },
    );

    let schemas = per_type
        .into_iter()
        .map(|(resource_type, entries)| {
            let schema = build_table_schema(
                resource_type_to_table(&version, &resource_type),
                resource_type.clone(),
                &entries,
            );
            (resource_type, schema)
        })
        .collect();

    SchemaRegistry {
        version,
        anchor,
        schemas,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SearchParameterResolve;
    use crate::memory::R4_SEARCH_PARAMETERS_INDEX;
    use haste_jwt::{ProjectId, TenantId};

    async fn registry() -> SchemaRegistry {
        let parameters = R4_SEARCH_PARAMETERS_INDEX
            .all(&TenantId::System, &ProjectId::System)
            .await
            .expect("system parameters resolve");

        generate_schemas(SupportedFHIRVersions::R4, &parameters)
    }

    fn schema<'a>(registry: &'a SchemaRegistry, resource_type: &str) -> &'a ResourceTypeSchema {
        registry
            .schemas
            .get(resource_type)
            .unwrap_or_else(|| panic!("{resource_type} schema"))
    }

    fn column<'a>(schema: &'a ResourceTypeSchema, name: &str) -> Option<&'a ColumnDef> {
        schema.columns.iter().find(|c| c.name == name)
    }

    #[test]
    fn tables_are_named_by_version_and_purpose() {
        assert_eq!(
            resource_type_to_table(&SupportedFHIRVersions::R4, "Patient"),
            "r4_patient_idx"
        );
        assert_eq!(
            shared_table_name(&SupportedFHIRVersions::R4, SharedTable::Token),
            "r4_param_token_idx"
        );
        assert_eq!(
            resource_table_name(&SupportedFHIRVersions::R4),
            "r4_resource_idx"
        );
    }

    #[tokio::test]
    async fn singular_parameters_get_scalar_columns() {
        let registry = registry().await;
        let patient = schema(&registry, "Patient");

        let birth_date = column(patient, "birthdate_start").expect("birthdate_start");
        assert_eq!(birth_date.column_type, ColumnType::BigInt);
        assert_eq!(birth_date.index, IndexKind::BTree);

        let gender = column(patient, "gender_code").expect("gender_code");
        assert_eq!(gender.column_type, ColumnType::Text);
    }

    /// `Patient.name` is `0..*`, so it uses the shared string table.
    #[tokio::test]
    async fn repeating_parameters_get_no_column() {
        let registry = registry().await;
        let patient = schema(&registry, "Patient");

        for code in ["name", "family", "given", "identifier", "address"] {
            assert!(
                !patient.parameters.contains_key(code),
                "'{code}' repeats and must not claim a column",
            );
        }
    }

    /// Case-insensitive prefix search needs an index over `LOWER(column)`.
    #[tokio::test]
    async fn string_columns_index_the_lowered_value() {
        let registry = registry().await;

        let string_columns: Vec<_> = registry
            .schemas
            .values()
            .flat_map(|schema| {
                schema
                    .parameters
                    .values()
                    .filter_map(move |columns| match columns {
                        ParamColumns::String { value } => Some((schema, value)),
                        _ => None,
                    })
            })
            .collect();

        assert!(!string_columns.is_empty(), "expected some string columns");
        for (schema, value) in string_columns {
            assert_eq!(
                column(schema, value).expect(value).index,
                IndexKind::LoweredPrefix,
                "'{value}' on '{}' is searched by prefix",
                schema.table_name,
            );
        }
    }

    #[tokio::test]
    async fn only_the_filtered_half_of_a_pair_is_indexed() {
        let registry = registry().await;
        let observation = schema(&registry, "Observation");

        let Some(ParamColumns::Reference {
            target_type,
            target_id,
        }) = observation.parameters.get("subject")
        else {
            panic!("Observation.subject should be a singular reference column");
        };

        assert_eq!(
            column(observation, target_id).unwrap().index,
            IndexKind::BTree
        );
        assert_eq!(
            column(observation, target_type).unwrap().index,
            IndexKind::None
        );
    }

    /// Each type takes only its own branch of a union expression.
    #[tokio::test]
    async fn shared_parameters_get_columns_where_their_branch_is_single() {
        let registry = registry().await;
        let has = |resource_type: &str, code: &str| {
            schema(&registry, resource_type)
                .parameters
                .contains_key(code)
        };

        assert!(has("Observation", "patient"));
        assert!(has("Observation", "encounter"));
        assert!(has("Condition", "patient"));
        assert!(has("Encounter", "date"));

        // `Observation.effective` may be a Timing (several dates).
        assert!(!has("Observation", "date"));
        // A CodeableConcept is one token per coding.
        assert!(!has("Observation", "code"));
        // `DocumentReference.context.encounter` repeats.
        assert!(!has("DocumentReference", "encounter"));
    }

    #[tokio::test]
    async fn resource_level_parameters_live_on_the_anchor() {
        let registry = registry().await;

        assert_eq!(registry.anchor.table_name, "r4_resource_idx");
        assert!(
            registry.anchor.parameters.contains_key("_lastUpdated"),
            "_lastUpdated belongs on the anchor",
        );
        assert!(
            !schema(&registry, "Patient")
                .parameters
                .contains_key("_lastUpdated"),
            "_lastUpdated must not be replicated onto every resource type table",
        );
    }

    #[tokio::test]
    async fn id_is_answered_from_the_resource_key() {
        let registry = registry().await;
        let anchor = &registry.anchor;

        assert_eq!(
            anchor.parameters.get("_id"),
            Some(&ParamColumns::Token {
                system: None,
                code: "resource_id".to_string()
            })
        );
        assert!(
            anchor.columns.iter().all(|c| !c.name.starts_with("_id")),
            "no column is created for _id"
        );
    }

    #[tokio::test]
    async fn anchor_columns_are_indexed() {
        let registry = registry().await;
        let anchor = &registry.anchor;

        let Some(ParamColumns::Date { start, end }) = anchor.parameters.get("_lastUpdated") else {
            panic!("_lastUpdated should be a date column pair");
        };
        assert_eq!(column(anchor, start).unwrap().index, IndexKind::BTree);
        // `_sort=-_lastUpdated` reads the end in descending order.
        assert_eq!(
            column(anchor, end).unwrap().index,
            IndexKind::BTreeDescending
        );
    }

    #[tokio::test]
    async fn reserved_columns_are_never_generated() {
        let registry = registry().await;

        for schema in registry.schemas.values() {
            for column in &schema.columns {
                assert!(
                    !RESERVED_COLUMNS.contains(&column.name.as_str()),
                    "generated column '{}' on '{}' collides with a fixed column",
                    column.name,
                    schema.table_name,
                );
            }
        }
    }

    #[tokio::test]
    async fn column_names_are_unique_per_table() {
        let registry = registry().await;

        for schema in registry.schemas.values() {
            let mut seen = HashSet::new();
            for column in &schema.columns {
                assert!(
                    seen.insert(column.name.clone()),
                    "duplicate column '{}' on '{}'",
                    column.name,
                    schema.table_name,
                );
            }
        }
    }
}
