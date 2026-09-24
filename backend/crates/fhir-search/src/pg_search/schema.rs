//! Table and column layout for the PG search backend.
//!
//! How a parameter is stored depends on whether it can produce more than one
//! value for a resource:
//!
//! - **Singular** parameters become columns on their resource type's table,
//!   `r4_patient_idx`. A scalar is what an index needs for ordered
//!   comparisons, prefix matches and sorts — an array only indexes for
//!   overlap — and several on one row let the planner combine predicates and
//!   walk a composite index straight down a sort order.
//!
//! - **Repeating** parameters become rows in the shared table for their value
//!   type, `r4_param_token_idx` and friends, keyed by canonical URL. One row
//!   per value keeps the value scalar, so the same indexes work; the cost is a
//!   join instead of a column read.
//!
//! Every table refers to its resource by `res_key`, a `BIGINT` the anchor
//! (`r4_resource_idx`) allocates. Only the anchor stores the tenant, project,
//! type and id; carrying eight bytes of key instead keeps the other tables'
//! indexes small enough to stay in memory.
//!
//! Cardinality is compiled in ahead of time — see
//! [`crate::search_parameter_cardinality`]. Anything not known to be singular
//! goes to the shared tables, since a scalar column keeps the first value and
//! silently drops the rest.

use std::collections::{HashMap, HashSet};

use haste_fhir_model::r4::generated::terminology::{BoundCode, SearchParamType};
use haste_repository::types::SupportedFHIRVersions;

use crate::{ParameterLevel, ResolvedParameter, search_parameter_cardinality};

/// The type of a generated column. Always a scalar: repeating values live in
/// rows, not arrays.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColumnType {
    Text,
    BigInt,
    Double,
}

impl ColumnType {
    /// The name used in `CREATE TABLE` / `ADD COLUMN`.
    #[must_use]
    pub const fn sql_type(self) -> &'static str {
        match self {
            ColumnType::Text => "TEXT",
            ColumnType::BigInt => "BIGINT",
            ColumnType::Double => "DOUBLE PRECISION",
        }
    }
}

/// How a column is indexed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndexKind {
    /// No index of its own: the other half of the pair carries it, and both
    /// sit on the same row.
    None,
    /// A plain B-tree, for equality and ordered comparisons.
    BTree,
    /// A B-tree stored `DESC NULLS LAST`, for a column a descending sort
    /// reads. Read backwards a plain B-tree yields `DESC NULLS FIRST`, so the
    /// sort would order every match instead of stopping at the page size.
    /// Serves comparisons as well as a plain one.
    BTreeDescending,
    /// A B-tree over `LOWER(column)` with `text_pattern_ops`, for the
    /// case-insensitive prefix match a string search defaults to. A plain
    /// B-tree cannot serve a predicate on an expression of the column.
    LoweredPrefix,
}

/// A single generated column.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColumnDef {
    pub name: String,
    pub column_type: ColumnType,
    pub index: IndexKind,
}

/// The columns backing one parameter, by role. Multi-part types keep their
/// parts in separate columns of the same row, so a token's system stays with
/// its own code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParamColumns {
    String {
        value: String,
    },
    /// `system` is absent for a token that never carries one: `_id` reads the
    /// anchor's `resource_id`, already indexed by its primary key.
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

impl ParamColumns {
    /// Every column this parameter occupies, in insert order.
    #[must_use]
    pub fn column_names(&self) -> Vec<&str> {
        match self {
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
}

/// One value column of a [`SharedTable`], as the migration's DDL and the batch
/// insert's array cast both need it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SharedColumn {
    pub name: &'static str,
    /// Valid both in `CREATE TABLE` and in the `$n::<sql_type>[]` cast the
    /// batch insert binds through.
    pub sql_type: &'static str,
    pub nullable: bool,
}

impl SharedColumn {
    const fn new(name: &'static str, sql_type: &'static str, nullable: bool) -> Self {
        Self {
            name,
            sql_type,
            nullable,
        }
    }
}

/// The shared table a repeating parameter writes to, one per value type.
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

impl SharedTable {
    /// Every shared table, in the order the migration creates them.
    pub const ALL: [SharedTable; 7] = [
        SharedTable::String,
        SharedTable::Token,
        SharedTable::Date,
        SharedTable::Number,
        SharedTable::Quantity,
        SharedTable::Uri,
        SharedTable::Reference,
    ];

    /// The value type this table holds.
    #[must_use]
    pub const fn kind(self) -> &'static str {
        match self {
            SharedTable::String => "string",
            SharedTable::Token => "token",
            SharedTable::Date => "date",
            SharedTable::Number => "number",
            SharedTable::Quantity => "quantity",
            SharedTable::Uri => "uri",
            SharedTable::Reference => "reference",
        }
    }

    /// The value columns. The DDL and the batch inserts read this same list,
    /// so neither can have a column the other lacks. The key columns
    /// (`res_key`, `param_identity`) are the same everywhere and not listed.
    #[must_use]
    pub const fn value_columns(self) -> &'static [SharedColumn] {
        match self {
            SharedTable::String | SharedTable::Uri => {
                const COLUMNS: &[SharedColumn] = &[SharedColumn::new("value", "text", false)];
                COLUMNS
            }
            SharedTable::Token => {
                // A Coding can carry either half alone.
                const COLUMNS: &[SharedColumn] = &[
                    SharedColumn::new("system", "text", true),
                    SharedColumn::new("code", "text", true),
                ];
                COLUMNS
            }
            SharedTable::Date => {
                const COLUMNS: &[SharedColumn] = &[
                    SharedColumn::new("start_ms", "int8", false),
                    SharedColumn::new("end_ms", "int8", false),
                ];
                COLUMNS
            }
            SharedTable::Number => {
                const COLUMNS: &[SharedColumn] = &[SharedColumn::new("value", "float8", false)];
                COLUMNS
            }
            SharedTable::Quantity => {
                const COLUMNS: &[SharedColumn] = &[
                    SharedColumn::new("start_value", "float8", false),
                    SharedColumn::new("end_value", "float8", false),
                    SharedColumn::new("start_system", "text", true),
                    SharedColumn::new("start_code", "text", true),
                ];
                COLUMNS
            }
            SharedTable::Reference => {
                // A Reference may name a type with no id, or be an absolute
                // URL that cannot be split.
                const COLUMNS: &[SharedColumn] = &[
                    SharedColumn::new("target_resource_type", "text", true),
                    SharedColumn::new("target_id", "text", true),
                ];
                COLUMNS
            }
        }
    }

    /// `r4_param_token_idx` — version, value type, and an index suffix. The
    /// `param_` segment keeps these apart from the resource type tables
    /// (`r4_patient_idx`), since no resource type name contains an underscore.
    #[must_use]
    pub fn table_name(self, version: &SupportedFHIRVersions) -> String {
        format!("{version}_param_{}_idx", self.kind())
    }

    /// The table `param_type` writes to, or `None` for a type with no
    /// representation here (composite, special).
    #[must_use]
    pub fn for_param_type(param_type: &BoundCode<SearchParamType>) -> Option<SharedTable> {
        if param_type == &SearchParamType::string() {
            Some(SharedTable::String)
        } else if param_type == &SearchParamType::token() {
            Some(SharedTable::Token)
        } else if param_type == &SearchParamType::date() {
            Some(SharedTable::Date)
        } else if param_type == &SearchParamType::number() {
            Some(SharedTable::Number)
        } else if param_type == &SearchParamType::quantity() {
            Some(SharedTable::Quantity)
        } else if param_type == &SearchParamType::uri() {
            Some(SharedTable::Uri)
        } else if param_type == &SearchParamType::reference() {
            Some(SharedTable::Reference)
        } else {
            None
        }
    }
}

/// One resource type's table of singular parameters.
#[derive(Debug, Clone, Default)]
pub struct ResourceTypeSchema {
    /// Resource type name, e.g. `"Patient"`.
    pub resource_type: String,
    /// Table name, e.g. `"r4_patient_idx"`.
    pub table_name: String,
    /// Search parameter `code` → the columns backing it.
    pub parameters: HashMap<String, ParamColumns>,
    /// All value columns, sorted by name.
    pub columns: Vec<ColumnDef>,
}

impl ResourceTypeSchema {
    /// The columns backing a search parameter `code`, if it has a column here.
    #[must_use]
    pub fn columns_for(&self, code: &str) -> Option<&ParamColumns> {
        self.parameters.get(code)
    }

    /// The position of a column within `columns`. A batched insert binds one
    /// fixed column list per statement, so values are collected into a slot per
    /// position. `columns` is sorted by name, hence the binary search.
    #[must_use]
    pub fn column_index(&self, name: &str) -> Option<usize> {
        self.columns
            .binary_search_by(|column| column.name.as_str().cmp(name))
            .ok()
    }
}

/// Every generated resource type schema, keyed by resource type name.
#[derive(Debug, Clone, Default)]
pub struct SchemaRegistry {
    version: SupportedFHIRVersions,
    /// Columns on the anchor table, for the parameters every resource has.
    ///
    /// `_lastUpdated`, `_tag` and the rest are based on `Resource`, so
    /// replicating them would store the same five columns on ~145 tables and
    /// still not answer a search naming no resource type. The anchor is the
    /// `FROM` of every search, typed or not.
    anchor: ResourceTypeSchema,
    schemas: HashMap<String, ResourceTypeSchema>,
}

impl SchemaRegistry {
    #[must_use]
    pub fn get(&self, resource_type: &str) -> Option<&ResourceTypeSchema> {
        self.schemas.get(resource_type)
    }

    pub fn iter(&self) -> impl Iterator<Item = &ResourceTypeSchema> {
        self.schemas.values()
    }

    /// The anchor table's own schema, carrying the resource-level parameters.
    #[must_use]
    pub fn anchor(&self) -> &ResourceTypeSchema {
        &self.anchor
    }

    /// The anchor table: one row per indexed resource, selected from by every
    /// search and hung off by every other table.
    #[must_use]
    pub fn resource_table_name(&self) -> String {
        resource_table_name(&self.version)
    }

    /// A shared table's name in this registry's FHIR version.
    #[must_use]
    pub fn shared_table_name(&self, table: SharedTable) -> String {
        table.table_name(&self.version)
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.schemas.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.schemas.is_empty()
    }
}

/// The resource id parameter, answered from the anchor's `resource_id` key
/// rather than a column of its own duplicating it.
const RESOURCE_ID_CODE: &str = "_id";

/// Bases whose parameters apply to every resource and so live on the anchor
/// (`_lastUpdated`, `_tag`, `_profile`, ...).
const UNIVERSAL_BASES: [&str; 2] = ["Resource", "DomainResource"];

/// The fixed columns of the anchor and the resource type tables. A parameter
/// whose generated name lands on one falls back to the shared tables.
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

/// Normalizes a `code` into a safe column base name.
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

/// The anchor table for a FHIR version: one row per indexed resource.
#[must_use]
pub fn resource_table_name(version: &SupportedFHIRVersions) -> String {
    format!("{version}_resource_idx")
}

/// `r4_patient_idx` — one table per resource type.
#[must_use]
pub fn resource_type_to_table(version: &SupportedFHIRVersions, resource_type: &str) -> String {
    format!("{version}_{}_idx", resource_type.to_ascii_lowercase())
}

/// The columns a singular parameter of this type needs, or `None` for a type
/// with no column representation. Derived from the same mapping the shared
/// tables use, so a parameter cannot get a column of one shape and rows of
/// another.
fn columns_for_type(base: &str, param_type: &BoundCode<SearchParamType>) -> Option<ParamColumns> {
    let part = |suffix: &str| format!("{base}{suffix}");

    Some(match SharedTable::for_param_type(param_type)? {
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

/// The definitions for a parameter's columns.
///
/// Only the half of a pair a query filters on is indexed: a token is looked up
/// by code and a reference by id, with the other half checked on the same row.
/// Indexing it too would mostly index a few repeated canonical URLs.
fn column_defs(columns: &ParamColumns) -> Vec<ColumnDef> {
    let text = |name: &String, index| ColumnDef {
        name: name.clone(),
        column_type: ColumnType::Text,
        index,
    };

    match columns {
        ParamColumns::String { value } => vec![text(value, IndexKind::LoweredPrefix)],
        ParamColumns::Uri { value } => vec![text(value, IndexKind::BTree)],
        ParamColumns::Number { value } => vec![ColumnDef {
            name: value.clone(),
            column_type: ColumnType::Double,
            index: IndexKind::BTree,
        }],
        ParamColumns::Token { system, code } => system
            .iter()
            .map(|system| text(system, IndexKind::None))
            .chain(std::iter::once(text(code, IndexKind::BTree)))
            .collect(),
        ParamColumns::Date { start, end } => vec![
            ColumnDef {
                name: start.clone(),
                column_type: ColumnType::BigInt,
                index: IndexKind::BTree,
            },
            // The only sort reading the period's end is a descending one.
            ColumnDef {
                name: end.clone(),
                column_type: ColumnType::BigInt,
                index: IndexKind::BTreeDescending,
            },
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
            ColumnDef {
                name: start.clone(),
                column_type: ColumnType::Double,
                index: IndexKind::BTree,
            },
            ColumnDef {
                name: end.clone(),
                column_type: ColumnType::Double,
                index: IndexKind::BTree,
            },
            text(system, IndexKind::None),
            text(code, IndexKind::None),
        ],
    }
}

/// The code, URL and columns `parameter` would occupy, or `None` when it can
/// have none: project-level, or with no expression or column representation.
/// The caller decides per resource type whether it actually gets them.
fn singular_columns(parameter: &ResolvedParameter) -> Option<(&str, &str, ParamColumns)> {
    if !matches!(parameter.level, ParameterLevel::System) {
        return None;
    }

    let search_parameter = &parameter.search_parameter;
    let code = search_parameter.code.value.as_deref()?;

    // Without an expression nothing is evaluated into the column.
    search_parameter
        .expression
        .as_ref()
        .and_then(|e| e.value.as_deref())?;

    let url = search_parameter.url.value.as_deref()?;
    let columns = columns_for_type(&code_to_column_base(code), &search_parameter.type_)?;
    Some((code, url, columns))
}

/// Lays out one table from its parameters, in order.
///
/// Taken names are seeded with the fixed columns. Two parameters can also
/// normalize to the same name; first writer wins and the loser resolves through
/// the shared tables, which have no such constraint.
fn build_table_schema(
    table_name: String,
    resource_type: String,
    entries: &[(String, ParamColumns)],
) -> ResourceTypeSchema {
    let mut schema = ResourceTypeSchema {
        table_name,
        resource_type,
        parameters: HashMap::new(),
        columns: Vec::new(),
    };

    let mut claimed: HashSet<String> = RESERVED_COLUMNS.iter().map(|c| (*c).to_string()).collect();

    for (code, columns) in entries {
        let names = columns.column_names();
        if names.iter().any(|name| claimed.contains(*name)) {
            tracing::warn!(
                "PG search: '{code}' on '{}' collides with an existing column; \
                 it resolves through the shared tables instead.",
                schema.table_name,
            );
            continue;
        }

        for name in names {
            claimed.insert(name.to_string());
        }

        schema.columns.extend(column_defs(columns));
        schema.parameters.insert(code.clone(), columns.clone());
    }

    schema.columns.sort_by(|a, b| a.name.cmp(&b.name));
    schema
}

/// A table per resource type from the singular system-level parameters, plus
/// the anchor's columns for the resource-level ones.
#[must_use]
pub fn generate_schemas(
    version: SupportedFHIRVersions,
    parameters: &[ResolvedParameter],
) -> SchemaRegistry {
    let mut universal: Vec<(String, ParamColumns)> = Vec::new();
    let mut per_type: HashMap<String, Vec<(String, ParamColumns)>> = HashMap::new();

    for parameter in parameters {
        let Some((code, url, columns)) = singular_columns(parameter) else {
            continue;
        };

        for base in parameter
            .search_parameter
            .base
            .iter()
            .filter_map(BoundCode::as_str)
        {
            // Gated per type: a shared parameter can repeat for one type and
            // not another, and a column is only safe where it cannot.
            if !search_parameter_cardinality::is_single_valued(url, base) {
                continue;
            }

            // `_id` is the anchor's own key, added below.
            if code == RESOURCE_ID_CODE && UNIVERSAL_BASES.contains(&base) {
                continue;
            }

            let entries = if UNIVERSAL_BASES.contains(&base) {
                &mut universal
            } else {
                per_type.entry(base.to_string()).or_default()
            };

            if !entries.iter().any(|(existing, _)| existing == code) {
                entries.push((code.to_string(), columns.clone()));
            }
        }
    }

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
            SharedTable::Token.table_name(&SupportedFHIRVersions::R4),
            "r4_param_token_idx"
        );
        assert_eq!(
            resource_table_name(&SupportedFHIRVersions::R4),
            "r4_resource_idx"
        );
    }

    /// Singular parameters get a scalar column, the reason for the split.
    #[tokio::test]
    async fn singular_parameters_get_scalar_columns() {
        let registry = registry().await;
        let patient = registry.get("Patient").expect("Patient schema");

        let birth_date = column(patient, "birthdate_start").expect("birthdate_start");
        assert_eq!(birth_date.column_type, ColumnType::BigInt);
        assert_eq!(birth_date.index, IndexKind::BTree);

        let gender = column(patient, "gender_code").expect("gender_code");
        assert_eq!(gender.column_type, ColumnType::Text);
    }

    /// `Patient.name` is `0..*`, so it resolves through the shared string
    /// table.
    #[tokio::test]
    async fn repeating_parameters_get_no_column() {
        let registry = registry().await;
        let patient = registry.get("Patient").expect("Patient schema");

        for code in ["name", "family", "given", "identifier", "address"] {
            assert!(
                patient.columns_for(code).is_none(),
                "'{code}' repeats and must not claim a column",
            );
        }
    }

    /// A case-insensitive prefix search needs an index over `LOWER(column)`; a
    /// plain B-tree cannot serve a predicate on an expression.
    #[tokio::test]
    async fn string_columns_index_the_lowered_value() {
        let registry = registry().await;

        let mut checked = 0;
        for schema in registry.iter() {
            for columns in schema.parameters.values() {
                if let ParamColumns::String { value } = columns {
                    assert_eq!(
                        column(schema, value).expect(value).index,
                        IndexKind::LoweredPrefix,
                        "'{value}' on '{}' is searched by prefix",
                        schema.table_name,
                    );
                    checked += 1;
                }
            }
        }

        assert!(checked > 0, "expected some string columns to check");
    }

    /// The half a query never filters on is checked on the row the indexed
    /// half already found.
    #[tokio::test]
    async fn only_the_filtered_half_of_a_pair_is_indexed() {
        let registry = registry().await;
        let observation = registry.get("Observation").expect("Observation schema");

        if let Some(ParamColumns::Reference {
            target_type,
            target_id,
        }) = observation.columns_for("subject")
        {
            assert_eq!(
                column(observation, target_id).unwrap().index,
                IndexKind::BTree
            );
            assert_eq!(
                column(observation, target_type).unwrap().index,
                IndexKind::None
            );
        } else {
            panic!("Observation.subject should be a singular reference column");
        }
    }

    /// Each resource type takes only its own branch of the union, so a single
    /// branch gets a column even when another type's repeats.
    #[tokio::test]
    async fn shared_parameters_get_columns_where_their_branch_is_single() {
        let registry = registry().await;
        let has = |resource_type: &str, code: &str| {
            registry
                .get(resource_type)
                .unwrap_or_else(|| panic!("{resource_type} schema"))
                .columns_for(code)
                .is_some()
        };

        assert!(has("Observation", "patient"));
        assert!(has("Observation", "encounter"));
        assert!(has("Condition", "patient"));
        assert!(has("Encounter", "date"));

        // `Observation.effective` may be a Timing, which is several dates.
        assert!(!has("Observation", "date"));
        // A CodeableConcept is one token per coding.
        assert!(!has("Observation", "code"));
        // `DocumentReference.context.encounter` repeats.
        assert!(!has("DocumentReference", "encounter"));
    }

    /// Resource-level parameters live on the anchor, which is also what lets a
    /// search naming no resource type read them.
    #[tokio::test]
    async fn resource_level_parameters_live_on_the_anchor() {
        let registry = registry().await;
        let anchor = registry.anchor();

        assert_eq!(anchor.table_name, "r4_resource_idx");
        assert!(
            anchor.columns_for("_lastUpdated").is_some(),
            "_lastUpdated belongs on the anchor",
        );

        let patient = registry.get("Patient").expect("Patient schema");
        assert!(
            patient.columns_for("_lastUpdated").is_none(),
            "_lastUpdated must not be replicated onto every resource type table",
        );
    }

    /// `_id` is answered from the anchor's `resource_id`, already indexed by
    /// its primary key, rather than a duplicate column.
    #[tokio::test]
    async fn id_is_answered_from_the_resource_key() {
        let registry = registry().await;
        let anchor = registry.anchor();

        assert_eq!(
            anchor.columns_for("_id"),
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

    /// The anchor's columns are indexed like any other, so a system-wide
    /// `_lastUpdated` filter or sort reads an index.
    #[tokio::test]
    async fn anchor_columns_are_indexed() {
        let registry = registry().await;
        let anchor = registry.anchor();

        let Some(ParamColumns::Date { start, end }) = anchor.columns_for("_lastUpdated") else {
            panic!("_lastUpdated should be a date column pair");
        };
        assert_eq!(column(anchor, start).unwrap().index, IndexKind::BTree);

        // `_sort=-_lastUpdated` reads the end, and stops at the page size only
        // if the index is already in that order.
        assert_eq!(
            column(anchor, end).unwrap().index,
            IndexKind::BTreeDescending
        );
    }

    #[tokio::test]
    async fn reserved_columns_are_never_generated() {
        let registry = registry().await;

        for schema in registry.iter() {
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

        for schema in registry.iter() {
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
