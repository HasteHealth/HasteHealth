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
    cardinality: (usize, Max),
    field: String,
    schema: serde_json::Value,
}

static PRIMITIVE_TYPES: &[&str] = &[
    "http://hl7.org/fhirpath/System.String",
    "http://hl7.org/fhirpath/System.Time",
    "http://hl7.org/fhirpath/System.Date",
    "http://hl7.org/fhirpath/System.DateTime",
    "http://hl7.org/fhirpath/System.Instant",
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
        "http://hl7.org/fhirpath/System.String"
        | "http://hl7.org/fhirpath/System.Time"
        | "http://hl7.org/fhirpath/System.Date"
        | "http://hl7.org/fhirpath/System.DateTime"
        | "http://hl7.org/fhirpath/System.Instant"
        | "markdown"
        | "url"
        | "canonical"
        | "uuid"
        | "string"
        | "uri"
        | "code"
        | "id"
        | "oid"
        | "base64Binary"
        | "time"
        | "date"
        | "dateTime" => JSONSchemaType::String,
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

fn process_leaf(sd: &StructureDefinition, element: &ElementDefinition) -> Vec<Processed> {
    let cardinality = utilities::extract::cardinality(element);
    let base_schema = if is_typechoice(element) {
        element
            .type_
            .as_ref()
            .unwrap_or(&vec![])
            .iter()
            .map(|fhir_type| {
                let type_code = fhir_type
                    .code
                    .value
                    .as_ref()
                    .map(|s| s.as_str())
                    .unwrap_or_default();

                let field_name = utilities::generate::type_choice_variant_name(element, type_code);

                if is_fhir_primitive_type(type_code) {
                    Processed {
                        cardinality: (0, cardinality.1.clone()),
                        field: field_name,
                        schema: json!({
                            "type": fhir_primitive_type_to_json_schema_type(type_code)
                        }),
                    }
                } else {
                    Processed {
                        cardinality: (0, cardinality.1.clone()),
                        field: field_name,
                        schema: json!({"type": "object"}),
                    }
                }
            })
            .collect()
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
            vec![Processed {
                cardinality,
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
            }]
        } else {
            vec![Processed {
                cardinality,
                field: utilities::extract::field_name(
                    element
                        .path
                        .value
                        .as_ref()
                        .map(|s| s.as_str())
                        .unwrap_or(""),
                ),
                schema: json!({"type": "object"}),
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

    wrap_if_array(
        sd,
        element,
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
        },
    )
}

pub fn sd_to_json_schema(
    _primitive_sds: &Vec<StructureDefinition>,
    sd: &StructureDefinition,
) -> Result<serde_json::Value, OperationOutcomeError> {
    let mut visitor = |element: &ElementDefinition,
                       children: Vec<Vec<Processed>>,
                       _index: usize|
     -> Vec<Processed> {
        if children.len() == 0 {
            process_leaf(&sd, element)
        } else {
            vec![process_complex(
                &sd,
                element,
                children.into_iter().flatten().collect(),
            )]
        }
    };

    let mut result = traversal::traversal(sd, &mut visitor).map_err(|e| {
        OperationOutcomeError::error(
            IssueType::Exception(None),
            format!("Error traversing StructureDefinition: {}", e),
        )
    })?;

    if let Some(result) = result.pop() {
        Ok(result.schema)
    } else {
        Err(OperationOutcomeError::error(
            IssueType::Exception(None),
            "No schema generated from StructureDefinition".to_string(),
        ))
    }
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

        assert_eq!(
            "{\"additionalProperties\":true,\"properties\":{\"active\":{\"type\":\"boolean\"},\"address\":{\"items\":{\"type\":\"object\"},\"type\":\"array\"},\"birthDate\":{\"type\":\"string\"},\"communication\":{\"items\":{\"additionalProperties\":true,\"properties\":{\"extension\":{\"items\":{\"type\":\"object\"},\"type\":\"array\"},\"id\":{\"type\":\"string\"},\"language\":{\"type\":\"object\"},\"modifierExtension\":{\"items\":{\"type\":\"object\"},\"type\":\"array\"},\"preferred\":{\"type\":\"boolean\"}},\"required\":[\"language\"],\"type\":\"object\"},\"type\":\"array\"},\"contact\":{\"items\":{\"additionalProperties\":true,\"properties\":{\"address\":{\"type\":\"object\"},\"extension\":{\"items\":{\"type\":\"object\"},\"type\":\"array\"},\"gender\":{\"type\":\"string\"},\"id\":{\"type\":\"string\"},\"modifierExtension\":{\"items\":{\"type\":\"object\"},\"type\":\"array\"},\"name\":{\"type\":\"object\"},\"organization\":{\"type\":\"object\"},\"period\":{\"type\":\"object\"},\"relationship\":{\"items\":{\"type\":\"object\"},\"type\":\"array\"},\"telecom\":{\"items\":{\"type\":\"object\"},\"type\":\"array\"}},\"required\":[],\"type\":\"object\"},\"type\":\"array\"},\"contained\":{\"items\":{\"type\":\"object\"},\"type\":\"array\"},\"deceasedBoolean\":{\"type\":\"boolean\"},\"deceasedDateTime\":{\"type\":\"string\"},\"extension\":{\"items\":{\"type\":\"object\"},\"type\":\"array\"},\"gender\":{\"type\":\"string\"},\"generalPractitioner\":{\"items\":{\"type\":\"object\"},\"type\":\"array\"},\"id\":{\"type\":\"string\"},\"identifier\":{\"items\":{\"type\":\"object\"},\"type\":\"array\"},\"implicitRules\":{\"type\":\"string\"},\"language\":{\"type\":\"string\"},\"link\":{\"items\":{\"additionalProperties\":true,\"properties\":{\"extension\":{\"items\":{\"type\":\"object\"},\"type\":\"array\"},\"id\":{\"type\":\"string\"},\"modifierExtension\":{\"items\":{\"type\":\"object\"},\"type\":\"array\"},\"other\":{\"type\":\"object\"},\"type\":{\"type\":\"string\"}},\"required\":[\"other\",\"type\"],\"type\":\"object\"},\"type\":\"array\"},\"managingOrganization\":{\"type\":\"object\"},\"maritalStatus\":{\"type\":\"object\"},\"meta\":{\"type\":\"object\"},\"modifierExtension\":{\"items\":{\"type\":\"object\"},\"type\":\"array\"},\"multipleBirthBoolean\":{\"type\":\"boolean\"},\"multipleBirthInteger\":{\"type\":\"number\"},\"name\":{\"items\":{\"type\":\"object\"},\"type\":\"array\"},\"photo\":{\"items\":{\"type\":\"object\"},\"type\":\"array\"},\"resourceType\":{\"const\":\"Patient\",\"type\":\"string\"},\"telecom\":{\"items\":{\"type\":\"object\"},\"type\":\"array\"},\"text\":{\"type\":\"object\"}},\"required\":[\"resourceType\"],\"type\":\"object\"}",
            serde_json::to_string(&schema).unwrap()
        );
    }
}
