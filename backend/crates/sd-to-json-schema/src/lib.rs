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

struct JSONSchema {}

struct Processed {
    cardinality: (usize, Max),
    field: String,
    schema: serde_json::Value,
}

static PRIMITIVE_TYPES: &[&str] = &[
    "boolean", "string", "code", "id", "uri", "dateTime", "date", "instant", "markdown", "oid",
    "uuid", "xhtml", "integer", "decimal",
];

fn fhir_primitive_type_to_json_schema_type(fhir_type: &str) -> JSONSchemaType {
    match fhir_type {
        "boolean" => JSONSchemaType::Boolean,
        "string" | "code" | "id" | "uri" | "dateTime" | "date" | "instant" | "markdown" | "oid"
        | "uuid" | "xhtml" => JSONSchemaType::String,
        "integer" | "decimal" => JSONSchemaType::Number,
        _ => JSONSchemaType::String,
    }
}

fn is_fhir_primitive_type(fhir_type: &str) -> bool {
    PRIMITIVE_TYPES.contains(&fhir_type)
}

fn process_leaf(_sd: &StructureDefinition, element: &ElementDefinition) -> Processed {
    if is_typechoice(element) {
        Processed {
            cardinality: utilities::extract::cardinality(element),
            field: utilities::extract::field_name(
                element
                    .path
                    .value
                    .as_ref()
                    .map(|s| s.as_str())
                    .unwrap_or(""),
            ),
            schema: json!({}),
        }
    } else {
        let type_ = element
            .type_
            .as_ref()
            .and_then(|t| t.first())
            .map(|t| t.code.as_ref())
            .and_then(|c| c.value.as_ref())
            .map(|s| s.as_str())
            .unwrap_or_default();

        if is_fhir_primitive_type(type_) {
            Processed {
                cardinality: utilities::extract::cardinality(element),
                field: utilities::extract::field_name(
                    element
                        .path
                        .value
                        .as_ref()
                        .map(|s| s.as_str())
                        .unwrap_or(""),
                ),
                schema: json!({
                    "type": fhir_primitive_type_to_json_schema_type(type_)
                }),
            }
        } else {
            Processed {
                cardinality: utilities::extract::cardinality(element),
                field: utilities::extract::field_name(
                    element
                        .path
                        .value
                        .as_ref()
                        .map(|s| s.as_str())
                        .unwrap_or(""),
                ),
                schema: json!({"type": "object"}),
            }
        }
    }
}

fn process_complex(
    sd: &StructureDefinition,
    element: &ElementDefinition,
    children: Vec<Processed>,
    // nested_types: &mut Vec<StructureDefinition>,
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
    };

    for child in children.into_iter() {
        if child.cardinality.0 > 0 {
            required_properties.push(child.field.clone());
        }
        properties.insert(child.field, child.schema);
    }

    Processed {
        cardinality: utilities::extract::cardinality(element),
        field: utilities::extract::field_name(
            element
                .path
                .value
                .as_ref()
                .map(|s| s.as_str())
                .unwrap_or(""),
        ),
        schema: json!({
            "type": "object",
            "properties": properties,
            "required": required_properties,
            "additionalProperties": true,
        }),
    }
}

pub fn sd_to_json_schema(
    primitive_sds: &Vec<StructureDefinition>,
    sd: &StructureDefinition,
) -> Result<serde_json::Value, OperationOutcomeError> {
    let mut visitor =
        |element: &ElementDefinition, children: Vec<Processed>, _index: usize| -> Processed {
            if children.len() == 0 {
                process_leaf(&sd, element)
            } else {
                process_complex(&sd, element, children)
            }
        };

    let result = traversal::traversal(sd, &mut visitor).map_err(|e| {
        OperationOutcomeError::error(
            IssueType::Exception(None),
            format!("Error traversing StructureDefinition: {}", e),
        )
    })?;

    Ok(result.schema)
}

#[cfg(test)]
mod test {
    use std::sync::LazyLock;

    use haste_fhir_model::r4::generated::resources::Bundle;

    use super::*;

    static RESOURCE_SDS: LazyLock<Vec<StructureDefinition>> = LazyLock::new(|| {
        let sd_str =
            include_str!("../../artifacts/artifacts/r4/hl7/minified/profiles-resources.min.json");

        let bundle: Bundle = haste_fhir_serialization_json::from_str(sd_str)
            .expect("Failed to parse StructureDefinitions");

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

    #[test]
    fn test_sd_to_json_schema() {
        let patient_sd = RESOURCE_SDS
            .iter()
            .find(|v| v.type_.value.as_ref().map(|s| s.as_str()) == Some("Patient"))
            .unwrap();

        let schema = sd_to_json_schema(&vec![], patient_sd).unwrap();

        println!("{:#?}", schema);

        assert_eq!(json!({}), schema);
    }
}
