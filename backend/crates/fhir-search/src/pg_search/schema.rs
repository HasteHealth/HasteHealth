//! Per-resource-type table schema generation for the hybrid PG search backend.
//!
//! The PG backend mirrors the Elasticsearch split: HL7 base (system-level)
//! search parameters get dedicated, type-specific columns on a table named
//! after the resource type (`search_patient`, `search_observation`, ...),
//! while project-level (tenant custom) parameters share the EAV
//! `search_dynamic_*` tables keyed by `param_url`.
//!
//! Every value column is an array, since a single resource can produce many
//! values for one parameter. Multi-part types (token, date, reference,
//! quantity) use *parallel* arrays: index `i` of each column belongs to the
//! same logical value, which lets queries recombine them with
//! `unnest(a, b) WITH ORDINALITY`.

use std::collections::{HashMap, HashSet};

use haste_fhir_model::r4::generated::terminology::{BoundCode, SearchParamType};

use crate::{ParameterLevel, ResolvedParameter};

/// The PostgreSQL type of a generated value column.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColumnType {
    TextArray,
    BigIntArray,
    DoubleArray,
}

impl ColumnType {
    /// The SQL type name used in `CREATE TABLE` / `ADD COLUMN`.
    #[must_use]
    pub const fn sql_type(self) -> &'static str {
        match self {
            ColumnType::TextArray => "TEXT[]",
            ColumnType::BigIntArray => "BIGINT[]",
            ColumnType::DoubleArray => "DOUBLE PRECISION[]",
        }
    }
}

/// A single generated column on a per-resource-type table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColumnDef {
    pub name: String,
    pub column_type: ColumnType,
    /// Whether a GIN index should be created for this column.
    pub indexed: bool,
}

/// The set of columns backing one search parameter, grouped by the role each
/// column plays. Query builders match on this to know which columns to
/// `unnest` together.
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
    /// Every column name this parameter occupies, in insert order.
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

/// The generated schema for one FHIR resource type's search table.
#[derive(Debug, Clone, Default)]
pub struct ResourceTypeSchema {
    /// Resource type name, e.g. `"Patient"`.
    pub resource_type: String,
    /// Table name, e.g. `"search_patient"`.
    pub table_name: String,
    /// Search parameter `code` → the columns backing it.
    pub parameters: HashMap<String, ParamColumns>,
    /// All value columns, in a stable (sorted) order.
    pub columns: Vec<ColumnDef>,
}

impl ResourceTypeSchema {
    /// Looks up the columns backing a search parameter `code`.
    #[must_use]
    pub fn columns_for(&self, code: &str) -> Option<&ParamColumns> {
        self.parameters.get(code)
    }
}

/// All generated per-resource-type schemas, keyed by resource type name.
#[derive(Debug, Clone, Default)]
pub struct SchemaRegistry {
    schemas: HashMap<String, ResourceTypeSchema>,
}

impl SchemaRegistry {
    #[must_use]
    pub fn get(&self, resource_type: &str) -> Option<&ResourceTypeSchema> {
        self.schemas.get(resource_type)
    }

    /// Iterates every generated schema.
    pub fn iter(&self) -> impl Iterator<Item = &ResourceTypeSchema> {
        self.schemas.values()
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

/// Resource types whose parameters apply to *every* resource table rather than
/// getting a table of their own (`_lastUpdated`, `_tag`, `_profile`, ...).
const UNIVERSAL_BASES: [&str; 2] = ["Resource", "DomainResource"];

/// `_id` is already the `resource_id` primary key column, so it never needs a
/// generated column of its own.
const SKIPPED_CODES: [&str; 1] = ["_id"];

/// The fixed columns every per-resource-type table carries. A search parameter
/// whose generated name lands on one of these (ImplementationGuide's
/// `resource` reference becomes `resource_id`, for instance) can't have a
/// column of its own and falls back to the dynamic tables.
const RESERVED_COLUMNS: [&str; 5] = [
    "tenant",
    "project",
    "resource_id",
    "version_id",
    "resource_type",
];

/// Converts a SearchParameter `code` into a safe PostgreSQL column base name:
/// lowercased, with every character outside `[a-z0-9_]` folded to `_`.
///
/// FHIR codes are already restricted to a conservative character set, but
/// `_lastUpdated` (leading underscore, camelCase) and hyphenated codes like
/// `address-city` both need normalizing.
#[must_use]
pub fn code_to_column_base(code: &str) -> String {
    let mut out = String::with_capacity(code.len());
    for ch in code.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
        } else {
            out.push('_');
        }
    }
    out
}

/// Converts a FHIR resource type name into its search table name.
#[must_use]
pub fn resource_type_to_table(resource_type: &str) -> String {
    format!("search_{}", resource_type.to_ascii_lowercase())
}

/// Builds the columns for one parameter, or `None` if the type has no PG
/// representation (composite, special, ...).
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

/// The column definitions (name + SQL type + whether to index) for a
/// `ParamColumns`.
fn column_defs(columns: &ParamColumns) -> Vec<ColumnDef> {
    match columns {
        ParamColumns::String { value } | ParamColumns::Uri { value } => vec![ColumnDef {
            name: value.clone(),
            column_type: ColumnType::TextArray,
            indexed: true,
        }],
        ParamColumns::Number { value } => vec![ColumnDef {
            name: value.clone(),
            column_type: ColumnType::DoubleArray,
            indexed: true,
        }],
        ParamColumns::Token { system, code } => vec![
            ColumnDef {
                name: system.clone(),
                column_type: ColumnType::TextArray,
                // The code is the selective half of a token; indexing the
                // system as well would mostly index a handful of repeated
                // canonical URLs.
                indexed: false,
            },
            ColumnDef {
                name: code.clone(),
                column_type: ColumnType::TextArray,
                indexed: true,
            },
        ],
        ParamColumns::Date { start, end } => vec![
            ColumnDef {
                name: start.clone(),
                column_type: ColumnType::BigIntArray,
                indexed: true,
            },
            ColumnDef {
                name: end.clone(),
                column_type: ColumnType::BigIntArray,
                indexed: true,
            },
        ],
        ParamColumns::Reference {
            target_type,
            target_id,
        } => vec![
            ColumnDef {
                name: target_type.clone(),
                column_type: ColumnType::TextArray,
                indexed: false,
            },
            ColumnDef {
                name: target_id.clone(),
                column_type: ColumnType::TextArray,
                indexed: true,
            },
        ],
        ParamColumns::Quantity {
            start,
            end,
            system,
            code,
        } => vec![
            ColumnDef {
                name: start.clone(),
                column_type: ColumnType::DoubleArray,
                indexed: true,
            },
            ColumnDef {
                name: end.clone(),
                column_type: ColumnType::DoubleArray,
                indexed: true,
            },
            ColumnDef {
                name: system.clone(),
                column_type: ColumnType::TextArray,
                indexed: false,
            },
            ColumnDef {
                name: code.clone(),
                column_type: ColumnType::TextArray,
                indexed: false,
            },
        ],
    }
}

/// Derives per-resource-type table schemas from the system-level search
/// parameters.
///
/// Parameters based on `Resource`/`DomainResource` apply to every resource
/// type and are therefore replicated onto each generated table. Project-level
/// parameters are ignored here — they live in the `search_dynamic_*` tables.
#[must_use]
pub fn generate_schemas(parameters: &[ResolvedParameter]) -> SchemaRegistry {
    // (code, ParamColumns) pairs that belong on every table.
    let mut universal: Vec<(String, ParamColumns)> = Vec::new();
    // resource type → (code, ParamColumns) pairs specific to it.
    let mut per_type: HashMap<String, Vec<(String, ParamColumns)>> = HashMap::new();
    // Every resource type that has at least one parameter, universal-only
    // types included.
    let mut resource_types: HashSet<String> = HashSet::new();

    for parameter in parameters {
        if !matches!(parameter.level, ParameterLevel::System) {
            continue;
        }

        let search_parameter = &parameter.search_parameter;

        let Some(code) = search_parameter.code.value.as_deref() else {
            continue;
        };

        if SKIPPED_CODES.contains(&code) {
            continue;
        }

        // A parameter with no FHIRPath expression is never indexed, so it
        // would only ever produce an always-NULL column.
        if search_parameter
            .expression
            .as_ref()
            .and_then(|e| e.value.as_deref())
            .is_none()
        {
            continue;
        }

        let column_base = code_to_column_base(code);
        let Some(columns) = columns_for_type(&column_base, &search_parameter.type_) else {
            continue;
        };

        for base in &search_parameter.base {
            let Some(base) = base.as_str() else {
                continue;
            };

            if UNIVERSAL_BASES.contains(&base) {
                if !universal.iter().any(|(existing, _)| existing == code) {
                    universal.push((code.to_string(), columns.clone()));
                }
            } else {
                resource_types.insert(base.to_string());
                let entries = per_type.entry(base.to_string()).or_default();
                if !entries.iter().any(|(existing, _)| existing == code) {
                    entries.push((code.to_string(), columns.clone()));
                }
            }
        }
    }

    let mut schemas = HashMap::with_capacity(resource_types.len());

    for resource_type in resource_types {
        let mut schema = ResourceTypeSchema {
            table_name: resource_type_to_table(&resource_type),
            resource_type: resource_type.clone(),
            parameters: HashMap::new(),
            columns: Vec::new(),
        };

        // Column names already claimed on this table, seeded with the fixed
        // columns. Two parameters on the same resource type can also normalize
        // to colliding names (a `date` date parameter wanting `date_start`
        // alongside a hypothetical `date-start` string parameter). First
        // writer wins; the loser falls back to the dynamic EAV tables, which
        // have no such constraint.
        let mut claimed: HashSet<String> =
            RESERVED_COLUMNS.iter().map(|c| (*c).to_string()).collect();

        // Resource-level parameters first so they are stable across tables.
        for (code, columns) in universal.iter().chain(
            per_type
                .get(&resource_type)
                .map(Vec::as_slice)
                .unwrap_or_default(),
        ) {
            let names = columns.column_names();
            if names.iter().any(|name| claimed.contains(*name)) {
                tracing::warn!(
                    "PG search: skipping search parameter '{code}' on '{resource_type}' — \
                     column name collision; it will resolve through the dynamic tables.",
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
        schemas.insert(resource_type, schema);
    }

    SchemaRegistry { schemas }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SearchParameterResolve;
    use crate::memory::R4_SEARCH_PARAMETERS_INDEX;
    use haste_jwt::{ProjectId, TenantId};

    async fn patient_schema() -> ResourceTypeSchema {
        let parameters = R4_SEARCH_PARAMETERS_INDEX
            .all(&TenantId::System, &ProjectId::System)
            .await
            .expect("system parameters resolve");

        generate_schemas(&parameters)
            .get("Patient")
            .expect("Patient schema generated")
            .clone()
    }

    fn has_column(schema: &ResourceTypeSchema, name: &str, column_type: ColumnType) -> bool {
        schema
            .columns
            .iter()
            .any(|c| c.name == name && c.column_type == column_type)
    }

    #[test]
    fn code_to_column_base_normalizes() {
        assert_eq!(code_to_column_base("address-city"), "address_city");
        assert_eq!(code_to_column_base("_lastUpdated"), "_lastupdated");
        assert_eq!(code_to_column_base("name"), "name");
    }

    #[test]
    fn resource_type_to_table_lowercases() {
        assert_eq!(resource_type_to_table("Patient"), "search_patient");
        assert_eq!(
            resource_type_to_table("MedicationRequest"),
            "search_medicationrequest"
        );
    }

    #[tokio::test]
    async fn patient_has_expected_string_columns() {
        let schema = patient_schema().await;

        assert_eq!(schema.table_name, "search_patient");
        for column in ["name", "family", "given", "address", "address_city"] {
            assert!(
                has_column(&schema, column, ColumnType::TextArray),
                "expected TEXT[] column '{column}' on search_patient",
            );
        }
    }

    #[tokio::test]
    async fn patient_has_expected_token_and_date_columns() {
        let schema = patient_schema().await;

        for column in ["identifier_system", "identifier_code", "gender_code"] {
            assert!(
                has_column(&schema, column, ColumnType::TextArray),
                "expected TEXT[] column '{column}' on search_patient",
            );
        }

        for column in ["birthdate_start", "birthdate_end"] {
            assert!(
                has_column(&schema, column, ColumnType::BigIntArray),
                "expected BIGINT[] column '{column}' on search_patient",
            );
        }
    }

    #[tokio::test]
    async fn patient_has_reference_and_resource_level_columns() {
        let schema = patient_schema().await;

        for column in ["general_practitioner_type", "general_practitioner_id"] {
            assert!(
                has_column(&schema, column, ColumnType::TextArray),
                "expected TEXT[] column '{column}' on search_patient",
            );
        }

        // Resource-level parameters are replicated onto every table.
        for column in ["_lastupdated_start", "_lastupdated_end"] {
            assert!(
                has_column(&schema, column, ColumnType::BigIntArray),
                "expected BIGINT[] column '{column}' on search_patient",
            );
        }
        for column in ["_tag_code", "_profile", "_security_code", "_source"] {
            assert!(
                has_column(&schema, column, ColumnType::TextArray),
                "expected TEXT[] column '{column}' on search_patient",
            );
        }
    }

    #[tokio::test]
    async fn id_is_skipped_and_lookups_resolve_by_code() {
        let schema = patient_schema().await;

        assert!(schema.columns_for("_id").is_none());

        assert_eq!(
            schema.columns_for("birthdate"),
            Some(&ParamColumns::Date {
                start: "birthdate_start".to_string(),
                end: "birthdate_end".to_string(),
            })
        );
        assert_eq!(
            schema.columns_for("identifier"),
            Some(&ParamColumns::Token {
                system: "identifier_system".to_string(),
                code: "identifier_code".to_string(),
            })
        );
    }

    #[tokio::test]
    async fn registry_covers_many_resource_types() {
        let parameters = R4_SEARCH_PARAMETERS_INDEX
            .all(&TenantId::System, &ProjectId::System)
            .await
            .expect("system parameters resolve");
        let registry = generate_schemas(&parameters);

        assert!(registry.get("Observation").is_some());
        assert!(registry.get("Encounter").is_some());
        // Universal bases never get a table of their own.
        assert!(registry.get("Resource").is_none());
        assert!(registry.get("DomainResource").is_none());
    }

    #[tokio::test]
    async fn reserved_columns_are_never_generated() {
        let parameters = R4_SEARCH_PARAMETERS_INDEX
            .all(&TenantId::System, &ProjectId::System)
            .await
            .expect("system parameters resolve");
        let registry = generate_schemas(&parameters);

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

        // ImplementationGuide's `resource` reference is the concrete case:
        // it would generate `resource_id`, so it must route to the dynamic
        // tables instead of claiming a column.
        let ig = registry
            .get("ImplementationGuide")
            .expect("ImplementationGuide schema generated");
        assert!(ig.columns_for("resource").is_none());
    }

    #[tokio::test]
    async fn column_names_are_unique_per_table() {
        let parameters = R4_SEARCH_PARAMETERS_INDEX
            .all(&TenantId::System, &ProjectId::System)
            .await
            .expect("system parameters resolve");
        let registry = generate_schemas(&parameters);

        for schema in registry.iter() {
            let mut seen = HashSet::new();
            for column in &schema.columns {
                assert!(
                    seen.insert(column.name.clone()),
                    "duplicate column '{}' on table '{}'",
                    column.name,
                    schema.table_name,
                );
            }
        }
    }
}

#[cfg(test)]
mod sql_preview {
    use super::*;
    use crate::SearchParameterResolve;
    use crate::memory::R4_SEARCH_PARAMETERS_INDEX;
    use haste_jwt::{ProjectId, TenantId};

    #[tokio::test]
    #[ignore = "prints generated DDL for manual inspection"]
    async fn print_patient_ddl() {
        let parameters = R4_SEARCH_PARAMETERS_INDEX
            .all(&TenantId::System, &ProjectId::System)
            .await
            .unwrap();
        let registry = generate_schemas(&parameters);
        let schema = registry.get("Patient").unwrap();
        println!("{}", crate::pg_search::migration::preview_ddl(schema));
    }
}
