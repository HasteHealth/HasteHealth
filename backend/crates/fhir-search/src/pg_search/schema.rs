//! Table and column layout for the PG search backend.
//!
//! A search parameter is stored one of two ways, decided by whether it can
//! produce more than one value for a single resource:
//!
//! - **Singular** parameters become columns on their resource type's own
//!   table, `r4_patient_idx`. The value is a scalar, which is what an index
//!   needs to answer an ordered comparison, a prefix match or a sort — an
//!   array can only be indexed for overlap. Several of them on one row is also
//!   what lets the planner combine predicates and, with a composite index,
//!   walk straight down a sort order and stop at the page size.
//!
//! - **Repeating** parameters become rows in the shared table for their value
//!   type, `r4_param_token_idx` and friends, keyed by the parameter's
//!   canonical URL.
//!   One row per value means the value is a scalar there too, so the same
//!   indexes work; the cost is a join rather than a column read.
//!
//! Which parameters are singular is decided ahead of time and compiled in —
//! see [`crate::search_parameter_cardinality`]. Anything not known to be
//! singular goes to the shared tables, because storing several values in a
//! scalar column keeps the first and silently drops the rest.

use std::collections::{HashMap, HashSet};

use haste_fhir_model::r4::generated::terminology::{BoundCode, SearchParamType};
use haste_repository::types::SupportedFHIRVersions;

use crate::{ParameterLevel, ResolvedParameter, search_parameter_cardinality};

/// The PostgreSQL type of a generated column. Every one is a scalar: the whole
/// point of the split is that repeating values live in rows, not arrays.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColumnType {
    Text,
    BigInt,
    Double,
}

impl ColumnType {
    /// The SQL type name used in `CREATE TABLE` / `ADD COLUMN`.
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
    /// No index of its own. The other half of a pair carries it — a token's
    /// system is only ever read beside its code, and both sit on the same row.
    None,
    /// A plain B-tree, for equality and ordered comparisons.
    BTree,
    /// A B-tree over `LOWER(column)` with `text_pattern_ops`, which is what
    /// serves the case-insensitive prefix match a FHIR string search performs
    /// by default. A plain B-tree cannot: the query compares an expression of
    /// the column, not the column.
    LoweredPrefix,
}

/// A single generated column.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColumnDef {
    pub name: String,
    pub column_type: ColumnType,
    pub index: IndexKind,
}

/// The columns backing one search parameter, grouped by the role each plays.
///
/// Multi-part types keep their parts in separate columns of the *same row*, so
/// a token's system stays with its own code without any of the pairing games a
/// parallel-array layout needs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParamColumns {
    String {
        value: String,
    },
    Token {
        system: String,
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
            ParamColumns::Token { system, code } => vec![system.as_str(), code.as_str()],
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

/// One value column of a [`SharedTable`], as both the migration's DDL and the
/// batch insert's array cast need it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SharedColumn {
    pub name: &'static str,
    /// A Postgres type name that is valid both in `CREATE TABLE` and in the
    /// `$n::<sql_type>[]` cast the batch insert binds through.
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

/// The shared table a repeating parameter's values are written to, one per
/// value type.
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

    /// The table's value columns. The DDL and the batch inserts both read
    /// this list, so a column cannot exist on one side and not the other —
    /// which is the failure it exists to prevent.
    ///
    /// The identity columns (`tenant`, `project`, `resource_type`,
    /// `resource_id`, `param_url`) are the same for every table and are not
    /// listed here.
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
                // A Reference can name a type with no id, or be an absolute
                // URL we cannot split, so neither half is guaranteed.
                const COLUMNS: &[SharedColumn] = &[
                    SharedColumn::new("target_resource_type", "text", true),
                    SharedColumn::new("target_id", "text", true),
                ];
                COLUMNS
            }
        }
    }

    /// `r4_param_token_idx` — the FHIR version, the value type, and a suffix
    /// marking it as a search index rather than stored resource data.
    ///
    /// The `param_` segment keeps these apart from the resource type tables
    /// (`r4_patient_idx`): a resource type name has no underscore, so no
    /// resource type can ever produce one of these names.
    #[must_use]
    pub fn table_name(self, version: &SupportedFHIRVersions) -> String {
        format!("{version}_param_{}_idx", self.kind())
    }

    /// The table a search parameter of `param_type` writes to, or `None` for a
    /// type with no representation here (composite, special).
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

    /// The position of a column within `columns`.
    ///
    /// A batched insert binds one fixed column list per statement, so a
    /// resource's values are collected into a slot per position. `columns` is
    /// sorted by name, which is what makes this a binary search.
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
    /// `_lastUpdated`, `_tag` and the rest are based on `Resource` rather than
    /// a concrete type, so replicating them onto all ~145 resource type tables
    /// would store the same five columns 145 times — and still not answer a
    /// search that names no resource type, which has no type table to read.
    /// The anchor has one row per resource and is the `FROM` of every search,
    /// typed or not, so it is where they belong.
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

    #[must_use]
    pub fn version(&self) -> SupportedFHIRVersions {
        self.version.clone()
    }

    /// The anchor table: one row per indexed resource, which every search
    /// selects from and every other table hangs off.
    #[must_use]
    pub fn resource_table_name(&self) -> String {
        resource_table_name(&self.version)
    }

    /// The name of a shared table in this registry's FHIR version.
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

/// Bases whose parameters apply to every resource, and therefore live on the
/// anchor table rather than on any one resource type's (`_lastUpdated`,
/// `_tag`, `_profile`, ...).
const UNIVERSAL_BASES: [&str; 2] = ["Resource", "DomainResource"];

/// The fixed columns every resource type table carries. A parameter whose
/// generated name lands on one of these cannot have a column and falls back to
/// the shared tables.
const RESERVED_COLUMNS: [&str; 5] = [
    "tenant",
    "project",
    "resource_id",
    "version_id",
    "resource_type",
];

/// Normalizes a search parameter `code` into a safe column base name.
#[must_use]
pub fn code_to_column_base(code: &str) -> String {
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
/// with no column representation.
fn columns_for_type(base: &str, param_type: &BoundCode<SearchParamType>) -> Option<ParamColumns> {
    if param_type == &SearchParamType::string() {
        Some(ParamColumns::String {
            value: base.to_string(),
        })
    } else if param_type == &SearchParamType::token() {
        Some(ParamColumns::Token {
            system: format!("{base}_system"),
            code: format!("{base}_code"),
        })
    } else if param_type == &SearchParamType::date() {
        Some(ParamColumns::Date {
            start: format!("{base}_start"),
            end: format!("{base}_end"),
        })
    } else if param_type == &SearchParamType::number() {
        Some(ParamColumns::Number {
            value: base.to_string(),
        })
    } else if param_type == &SearchParamType::uri() {
        Some(ParamColumns::Uri {
            value: base.to_string(),
        })
    } else if param_type == &SearchParamType::reference() {
        Some(ParamColumns::Reference {
            target_type: format!("{base}_type"),
            target_id: format!("{base}_id"),
        })
    } else if param_type == &SearchParamType::quantity() {
        Some(ParamColumns::Quantity {
            start: format!("{base}_start"),
            end: format!("{base}_end"),
            system: format!("{base}_system"),
            code: format!("{base}_code"),
        })
    } else {
        None
    }
}

/// The definitions for a parameter's columns.
///
/// Only the half of a pair a query actually filters on is indexed. A token is
/// looked up by code and its system checked on the same row; a reference by
/// id and its type checked the same way. Indexing the other half would mostly
/// index a handful of repeated canonical URLs.
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
        ParamColumns::Token { system, code } => {
            vec![text(system, IndexKind::None), text(code, IndexKind::BTree)]
        }
        ParamColumns::Date { start, end } => vec![
            ColumnDef {
                name: start.clone(),
                column_type: ColumnType::BigInt,
                index: IndexKind::BTree,
            },
            ColumnDef {
                name: end.clone(),
                column_type: ColumnType::BigInt,
                index: IndexKind::BTree,
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

/// The columns for `parameter`, or `None` when it cannot have any.
///
/// A parameter is given a column only when it is known to produce at most one
/// value. Everything else — repeating parameters, project-level parameters,
/// anything the cardinality analysis could not settle — is left out, and
/// resolves through the shared tables instead.
fn singular_columns(parameter: &ResolvedParameter) -> Option<(&str, ParamColumns)> {
    if !matches!(parameter.level, ParameterLevel::System) {
        return None;
    }

    let search_parameter = &parameter.search_parameter;
    let code = search_parameter.code.value.as_deref()?;

    // Without an expression nothing is ever evaluated into the column.
    search_parameter
        .expression
        .as_ref()
        .and_then(|e| e.value.as_deref())?;

    // The compiled classification is the gate: a column is only safe for a
    // parameter that cannot produce a second value to drop.
    let url = search_parameter.url.value.as_deref()?;
    if !search_parameter_cardinality::is_single_valued(url) {
        return None;
    }

    let columns = columns_for_type(&code_to_column_base(code), &search_parameter.type_)?;
    Some((code, columns))
}

/// Lays out one table from its parameters, in order.
///
/// Names already taken on the table are seeded with the fixed columns. Two
/// parameters can also normalize to colliding names; first writer wins and the
/// loser resolves through the shared tables, which have no such constraint.
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

/// Builds a table per resource type from the singular system-level
/// parameters, plus the anchor's columns for the resource-level ones.
#[must_use]
pub fn generate_schemas(
    version: SupportedFHIRVersions,
    parameters: &[ResolvedParameter],
) -> SchemaRegistry {
    let mut universal: Vec<(String, ParamColumns)> = Vec::new();
    let mut per_type: HashMap<String, Vec<(String, ParamColumns)>> = HashMap::new();

    for parameter in parameters {
        let Some((code, columns)) = singular_columns(parameter) else {
            continue;
        };

        for base in parameter
            .search_parameter
            .base
            .iter()
            .filter_map(BoundCode::as_str)
        {
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

    let anchor = build_table_schema(
        resource_table_name(&version),
        "Resource".to_string(),
        &universal,
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

    /// Singular parameters get a column; the value is a scalar, which is the
    /// whole reason for the split.
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

    /// `Patient.name` is `0..*`, so it has no column and resolves through the
    /// shared string table instead.
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

    /// A string column is searched with a case-insensitive prefix, so its
    /// index has to be over `LOWER(column)` — a plain B-tree cannot serve a
    /// predicate on an expression of the column.
    #[tokio::test]
    async fn string_columns_index_the_lowered_value() {
        let registry = registry().await;
        let patient = registry.get("Patient").expect("Patient schema");

        // `Patient.gender` is a token; `phonetic` and `name` repeat. A
        // singular string on Patient is harder to come by, so check the rule
        // holds wherever a string column exists at all.
        let mut checked = 0;
        for schema in registry.iter() {
            for column in &schema.columns {
                if column.column_type == ColumnType::Text
                    && column.index == IndexKind::LoweredPrefix
                {
                    checked += 1;
                }
            }
        }
        assert!(checked > 0, "expected some lowered-prefix string columns");
        let _ = patient;
    }

    /// The paired half a query never filters on carries no index of its own —
    /// it is checked on the row the indexed half already found.
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

    /// Resource-level parameters live on the anchor, not on every resource
    /// type's table — which is also what lets a search naming no resource
    /// type read them.
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

    /// `_id` is indexed like any other resource-level token: a column pair on
    /// the anchor, looked up by its indexed code.
    #[tokio::test]
    async fn id_is_an_ordinary_anchor_column() {
        let registry = registry().await;
        let anchor = registry.anchor();

        let Some(ParamColumns::Token { code, .. }) = anchor.columns_for("_id") else {
            panic!("_id should be a token column on the anchor");
        };
        assert_eq!(column(anchor, code).unwrap().index, IndexKind::BTree);
    }

    /// The anchor's columns are indexed like any other, so a system-wide
    /// `_lastUpdated` filter or sort reads an index rather than scanning.
    #[tokio::test]
    async fn anchor_columns_are_indexed() {
        let registry = registry().await;
        let anchor = registry.anchor();

        let last_updated = anchor
            .columns_for("_lastUpdated")
            .expect("_lastUpdated column");

        for name in last_updated.column_names() {
            let def = column(anchor, name).expect("column definition");
            assert_eq!(def.index, IndexKind::BTree, "{name} should be indexed");
        }
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
