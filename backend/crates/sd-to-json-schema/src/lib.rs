use std::collections::HashMap;

use haste_codegen::{
    traversal,
    utilities::{self, conditionals::is_typechoice, extract::Max},
};
use haste_fhir_model::r4::generated::{
    resources::StructureDefinition, terminology::IssueType, types::ElementDefinition,
};
use haste_fhir_operation_error::OperationOutcomeError;
use serde_json::json;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
enum JSONSchemaType {
    Object,
    Boolean,
    String,
    Number,
    Array,
}

#[allow(dead_code)]
struct JSONSchema {}

struct Processed {
    cardinality: (u64, Max),
    field: String,
    schema: serde_json::Value,
}

static PRIMITIVE_TYPES: &[&str] = &[
    "http://hl7.org/fhirpath/System.String",
    "http://hl7.org/fhirpath/System.Time",
    "http://hl7.org/fhirpath/System.Date",
    "http://hl7.org/fhirpath/System.DateTime",
    "http://hl7.org/fhirpath/System.Instant",
    "xhtml",
    "markdown",
    "url",
    "canonical",
    "uuid",
    "string",
    "uri",
    "code",
    "id",
    "oid",
    "base64Binary",
    "time",
    "date",
    "dateTime",
    "instant",
    "http://hl7.org/fhirpath/System.Boolean",
    "boolean",
    "http://hl7.org/fhirpath/System.Integer",
    "http://hl7.org/fhirpath/System.Decimal",
    "decimal",
    "integer",
    "unsignedInt",
    "positiveInt",
];

fn fhir_primitive_type_to_json_schema_type(fhir_type: &str) -> JSONSchemaType {
    match fhir_type {
        "http://hl7.org/fhirpath/System.Boolean" | "boolean" => JSONSchemaType::Boolean,
        "http://hl7.org/fhirpath/System.Integer"
        | "http://hl7.org/fhirpath/System.Decimal"
        | "decimal"
        | "integer"
        | "unsignedInt"
        | "positiveInt" => JSONSchemaType::Number,
        _ => JSONSchemaType::String,
    }
}

fn is_fhir_primitive_type(fhir_type: &str) -> bool {
    PRIMITIVE_TYPES.contains(&fhir_type)
}

fn wrap_if_array(
    sd: &StructureDefinition,
    element: &ElementDefinition,
    base: Processed,
) -> Processed {
    match base.cardinality.1 {
        Max::Unlimited if !utilities::conditionals::is_root(sd, element) => Processed {
            cardinality: base.cardinality,
            field: base.field,
            schema: json!({
                "type": "array",
                "items": base.schema,
            }),
        },
        Max::Fixed(n) if n > 1 && !utilities::conditionals::is_root(sd, element) => Processed {
            cardinality: base.cardinality,
            field: base.field,
            schema: json!({
                "type": "array",
                "items": base.schema,
            }),
        },
        _ => base,
    }
}

// Generate a JSON Schema reference for a FHIR type
// If it's a Resource or DomainResource, we return a generic object schema.
fn datatype_reference_schema(schema_loc: &str, fhir_type: &str) -> serde_json::Value {
    match fhir_type {
        "DomainResource" | "Resource" => json!({
            "type": "object",
             "additionalProperties": true,
        }),
        _ => json!({
            "$ref": format!("{}/{}", schema_loc, fhir_type)
        }),
    }
}

fn process_leaf(
    schema_loc: &str,
    sd: &StructureDefinition,
    element: &ElementDefinition,
) -> Vec<Processed> {
    let cardinality = utilities::extract::cardinality(element);
    let base_schema = if is_typechoice(element) {
        element
            .type_
            .as_ref()
            .unwrap_or(&vec![])
            .iter()
            .flat_map(|fhir_type| {
                let type_code = fhir_type.code.value.as_deref().unwrap_or_default();

                let field_name = utilities::generate::type_choice_variant_name(element, type_code);

                if is_fhir_primitive_type(type_code) {
                    vec![
                        Processed {
                            cardinality: (0, cardinality.1),
                            field: format!("_{field_name}"),
                            schema: datatype_reference_schema(schema_loc, "Element"),
                        },
                        Processed {
                            cardinality: (0, cardinality.1),
                            field: field_name,
                            schema: json!({
                                "type": fhir_primitive_type_to_json_schema_type(type_code)
                            }),
                        },
                    ]
                } else {
                    vec![Processed {
                        cardinality: (0, cardinality.1),
                        field: field_name,
                        schema: datatype_reference_schema(schema_loc, type_code),
                    }]
                }
            })
            .collect()
    } else {
        let type_code = element
            .type_
            .as_ref()
            .and_then(|t| t.first())
            .map(|t| t.code.as_ref())
            .and_then(|c| c.value.as_ref())
            .map(std::string::String::as_str)
            .unwrap_or_default();
        let field_name =
            utilities::extract::field_name(element.path.value.as_deref().map_or("", |s| s));

        if is_fhir_primitive_type(type_code) {
            vec![
                Processed {
                    cardinality: (0, cardinality.1),
                    field: format!("_{field_name}"),
                    schema: datatype_reference_schema(schema_loc, "Element"),
                },
                Processed {
                    cardinality,
                    field: field_name,
                    schema: json!({
                        "type": fhir_primitive_type_to_json_schema_type(type_code)
                    }),
                },
            ]
        } else {
            vec![Processed {
                cardinality,
                field: field_name,
                schema: datatype_reference_schema(schema_loc, type_code),
            }]
        }
    };

    base_schema
        .into_iter()
        .map(|schema| wrap_if_array(sd, element, schema))
        .collect()
}

fn process_complex(
    sd: &StructureDefinition,
    element: &ElementDefinition,
    children: Vec<Processed>,
) -> Processed {
    let mut required_properties = vec![];
    let mut properties: HashMap<String, serde_json::Value> = HashMap::new();
    if utilities::conditionals::is_root(sd, element) && utilities::conditionals::is_resource_sd(sd)
    {
        properties.insert(
            "resourceType".to_string(),
            json!({
                "type": "string",
                "const": sd.type_.value.as_ref().unwrap_or(&"Unknown".to_string()),
            }),
        );
        required_properties.push("resourceType".to_string());
    }

    for child in children {
        if child.cardinality.0 > 0 {
            required_properties.push(child.field.clone());
        }
        properties.insert(child.field, child.schema);
    }

    wrap_if_array(
        sd,
        element,
        Processed {
            cardinality: utilities::extract::cardinality(element),
            field: utilities::extract::field_name(element.path.value.as_deref().map_or("", |s| s)),
            schema: json!({
                "type": "object",
                "properties": properties,
                "required": required_properties,
                "additionalProperties": false,
            }),
        },
    )
}

/// Generates an isolated JSON Schema from a FHIR [`StructureDefinition`].
///
/// Traverses the structure definition and converts leaf elements into
/// primitive schemas and complex elements into nested schemas. Returns an
/// [`OperationOutcomeError`] if traversal fails or no schema can be generated.
///
/// # Arguments
///
/// * `schema_loc` - The location used when generating the schema.
/// * `sd` - The FHIR [`StructureDefinition`] to convert.
///
/// # Errors
///
/// Returns an [`OperationOutcomeError`] if the structure definition cannot
/// be traversed or does not produce a schema.
pub fn isolated_schema(
    schema_loc: &str,
    sd: &StructureDefinition,
) -> Result<serde_json::Value, OperationOutcomeError> {
    let mut visitor = |element: &ElementDefinition,
                       children: Vec<Vec<Processed>>,
                       _index: usize|
     -> Vec<Processed> {
        if children.is_empty() {
            process_leaf(schema_loc, sd, element)
        } else {
            vec![process_complex(
                sd,
                element,
                children.into_iter().flatten().collect(),
            )]
        }
    };

    let mut result = traversal::traversal(sd, &mut visitor).map_err(|e| {
        OperationOutcomeError::error(
            IssueType::exception(),
            format!("Error traversing StructureDefinition: {e}"),
        )
    })?;

    if let Some(result) = result.pop() {
        Ok(result.schema)
    } else {
        Err(OperationOutcomeError::error(
            IssueType::exception(),
            "No schema generated from StructureDefinition".to_string(),
        ))
    }
}

// Creates a type schema for a bundle of resources
#[must_use]
pub fn bundle_of_resource(resource_schema: &serde_json::Value) -> serde_json::Value {
    json!({
        "type": "object",
        "properties": {
            "resourceType": {
                "type": "string",
                "const": "Bundle"
            },
            "type": {
                "enum": ["collection", "searchset", "history"]
            },
            "entry": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "resource": resource_schema
                    },
                    "required": ["resource"],
                    "additionalProperties": true
                }
            }
        },
        "required": ["resourceType", "type", "entry"],
        "additionalProperties": false
    })
}

#[cfg(test)]
mod test {
    use std::sync::LazyLock;

    use haste_fhir_model::r4::generated::{
        resources::{Bundle, Patient},
        terminology::StructureDefinitionKind,
        types::{FHIRString, HumanName},
    };

    use super::*;

    static RESOURCE_SDS: LazyLock<Vec<StructureDefinition>> = LazyLock::new(|| {
        let sd_str =
            include_str!("../../artifacts/artifacts/r4/hl7/minified/profiles-resources.min.json");

        let bundle: Bundle =
            serde_json::from_str(sd_str).expect("Failed to parse StructureDefinitions");

        bundle
            .entry
            .unwrap_or_default()
            .into_iter()
            .filter_map(|entry| entry.resource)
            .filter_map(|resource| {
                if let haste_fhir_model::r4::generated::resources::Resource::StructureDefinition(
                    sd,
                ) = *resource
                {
                    Some(sd)
                } else {
                    None
                }
            })
            .collect()
    });

    /// Test-only stand-in for the real `{schema_base_url}` complex types are
    /// referenced against in production.
    const TEST_SCHEMA_BASE_URL: &str = "https://example.com/schemas/fhir";

    /// Complex type schemas, keyed by the full external URL they're
    /// referenced by (matching what `isolated_schema` generates), so tests
    /// can resolve them the same way production does - via `$ref`, not `$defs`.
    pub static FHIR_COMPLEX_TYPE_DEFINITIONS: LazyLock<HashMap<String, serde_json::Value>> =
        LazyLock::new(|| {
            let sd_str =
                include_str!("../../artifacts/artifacts/r4/hl7/minified/profiles-types.min.json");

            let bundle: Bundle =
                serde_json::from_str(sd_str).expect("Failed to parse StructureDefinitions");

            bundle
            .entry
            .unwrap_or_default()
            .into_iter()
            .filter_map(|entry| entry.resource)
            .filter_map(|resource| {
                if let haste_fhir_model::r4::generated::resources::Resource::StructureDefinition(
                    sd,
                ) = *resource
                {
                    Some(sd)
                } else {
                    None
                }
            })
            .filter(|sd|
                sd.kind  == StructureDefinitionKind::complex_type()
            )
            .map(|sd| {
                let type_name = sd.type_.value.clone().unwrap();
                (
                    format!("{TEST_SCHEMA_BASE_URL}/{type_name}"),
                    isolated_schema(TEST_SCHEMA_BASE_URL, &sd).unwrap(),
                )
            })
            .collect::<HashMap<String, _>>()
        });

    /// Resolves the external `$ref`s complex types are generated with by
    /// looking them up in an in-memory map instead of making real HTTP calls.
    struct TestSchemaRetriever;

    impl jsonschema::Retrieve for TestSchemaRetriever {
        fn retrieve(
            &self,
            uri: &jsonschema::Uri<String>,
        ) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
            FHIR_COMPLEX_TYPE_DEFINITIONS
                .get(uri.as_str())
                .cloned()
                .ok_or_else(|| format!("Unknown schema: {uri}").into())
        }
    }

    #[test]
    fn test_sd_to_json_schema() {
        let patient_sd = RESOURCE_SDS
            .iter()
            .find(|v| v.type_.value.as_deref() == Some("Patient"))
            .unwrap();

        let schema = isolated_schema(TEST_SCHEMA_BASE_URL, patient_sd).unwrap();

        println!("{}", serde_json::to_string_pretty(&schema).unwrap());

        assert!(!serde_json::to_string(&schema).unwrap().is_empty());
    }

    #[test]
    fn patient_sd_test() {
        let patient_sd = RESOURCE_SDS
            .iter()
            .find(|v| v.type_.value.as_deref() == Some("Patient"))
            .unwrap();

        let schema = isolated_schema(TEST_SCHEMA_BASE_URL, patient_sd).unwrap();

        let validator = jsonschema::options()
            .with_retriever(TestSchemaRetriever)
            .build(&schema)
            .unwrap();

        let patient_data = serde_json::to_string(&Patient {
            name: Some(vec![HumanName {
                family: Some(Box::new(FHIRString {
                    value: Some("Doe".to_string()),
                    ..Default::default()
                })),
                given: Some(vec![FHIRString {
                    value: Some("John".to_string()),
                    ..Default::default()
                }]),
                ..Default::default()
            }]),
            ..Default::default()
        })
        .unwrap();

        let mut patient_json = serde_json::from_str(&patient_data).unwrap();
        let result = validator.validate(&patient_json);
        assert!(result.is_ok());

        patient_json["name"][0]["_given"] = json!("This is not a valid value");
        let result = validator.validate(&patient_json);
        assert!(result.is_err());

        patient_json["name"][0]["_given"] = json!([{"id": "1"}]);
        let result = validator.validate(&patient_json);
        println!("{result:?}");
        assert!(result.is_ok());

        patient_json["name"] = json!("This is not a valid value");
        let result = validator.validate(&patient_json);

        assert!(result.is_err());
    }
}
