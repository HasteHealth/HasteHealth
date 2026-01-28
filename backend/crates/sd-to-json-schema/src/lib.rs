use std::collections::HashMap;

use haste_codegen::{traversal, utilities};
use haste_fhir_model::r4::generated::{
    resources::StructureDefinition, terminology::IssueType, types::ElementDefinition,
};
use haste_fhir_operation_error::OperationOutcomeError;
use serde_json::json;

enum JSONSchemaType {
    Object,
    Boolean,
    String,
    Number,
    Array,
}

struct JSONSchema {}

struct Processed {
    field: String,
    schema: serde_json::Value,
}

fn process_leaf(_sd: &StructureDefinition, _element: &ElementDefinition) -> serde_json::Value {
    json!({})
}

fn process_complex(
    sd: &StructureDefinition,
    element: &ElementDefinition,
    _children: Vec<serde_json::Value>,
    // nested_types: &mut Vec<StructureDefinition>,
) -> serde_json::Value {
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

    json!({
        "type": "object",
        "properties": properties,
        "required": required_properties,
        "additionalProperties": true,
    })
}

pub fn sd_to_json_schema(
    primitive_sds: &Vec<StructureDefinition>,
    sd: &StructureDefinition,
) -> Result<JSONSchema, OperationOutcomeError> {
    let mut visitor = |element: &ElementDefinition,
                       children: Vec<serde_json::Value>,
                       _index: usize|
     -> serde_json::Value {
        if children.len() == 0 {
            process_leaf(&sd, element)
        } else {
            process_complex(&sd, element, children)
        }
    };

    let result = traversal::traversal(sd, &mut visitor);

    Err(OperationOutcomeError::error(
            IssueType::Invalid(None),
            "StructureDefinition does not have a snapshot. This is required for conversion to JSON Schema.".to_string(),
        ))
}
