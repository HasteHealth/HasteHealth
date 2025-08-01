#![allow(unused)]
use std::collections::{HashMap, HashSet};

use once_cell::sync::Lazy;

/// Some of these keywords are present as properties in the FHIR spec.
/// We need to prefix them with an underscore to avoid conflicts.
/// And use an attribute to rename the field in the generated code.
pub static RUST_KEYWORDS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    let mut m = HashSet::new();
    m.insert("self");
    m.insert("Self");
    m.insert("super");
    m.insert("type");
    m.insert("use");
    m.insert("identifier");
    m.insert("abstract");
    m.insert("for");
    m.insert("if");
    m.insert("else");
    m.insert("match");
    m.insert("while");
    m.insert("loop");
    m.insert("break");
    m.insert("continue");
    m.insert("ref");
    m
});

pub static RUST_PRIMITIVES: Lazy<HashMap<String, String>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert(
        "http://hl7.org/fhirpath/System.String".to_string(),
        "String".to_string(),
    );
    m.insert(
        "http://hl7.org/fhirpath/System.Decimal".to_string(),
        "f64".to_string(),
    );
    m.insert(
        "http://hl7.org/fhirpath/System.Boolean".to_string(),
        "bool".to_string(),
    );
    m.insert(
        "http://hl7.org/fhirpath/System.Integer".to_string(),
        "i64".to_string(),
    );
    m.insert(
        "http://hl7.org/fhirpath/System.Time".to_string(),
        "oxidized_fhir_datetime::Time".to_string(),
    );
    m.insert(
        "http://hl7.org/fhirpath/System.Date".to_string(),
        "oxidized_fhir_datetime::Date".to_string(),
    );
    m.insert(
        "http://hl7.org/fhirpath/System.DateTime".to_string(),
        "oxidized_fhir_datetime::DateTime".to_string(),
    );
    m
});

pub static FHIR_PRIMITIVES: Lazy<HashMap<String, String>> = Lazy::new(|| {
    let mut m = HashMap::new();
    // bool type
    m.insert("boolean".to_string(), "FHIRBoolean".to_string());

    // f64 type
    m.insert("decimal".to_string(), "FHIRDecimal".to_string());

    // i64 type
    m.insert("integer".to_string(), "FHIRInteger".to_string());
    // u64 type
    m.insert("positiveInt".to_string(), "FHIRPositiveInt".to_string());
    m.insert("unsignedInt".to_string(), "FHIRUnsignedInt".to_string());

    // String type
    m.insert("base64Binary".to_string(), "FHIRBase64Binary".to_string());
    m.insert("canonical".to_string(), "FHIRString".to_string());
    m.insert("code".to_string(), "FHIRCode".to_string());
    m.insert("id".to_string(), "FHIRId".to_string());
    m.insert("markdown".to_string(), "FHIRMarkdown".to_string());
    m.insert("oid".to_string(), "FHIROid".to_string());
    m.insert("string".to_string(), "FHIRString".to_string());
    m.insert("uri".to_string(), "FHIRUri".to_string());
    m.insert("url".to_string(), "FHIRUrl".to_string());
    m.insert("uuid".to_string(), "FHIRUuid".to_string());
    m.insert("xhtml".to_string(), "FHIRXhtml".to_string());

    // Date and Time types
    m.insert("instant".to_string(), "FHIRInstant".to_string());
    m.insert("date".to_string(), "FHIRDate".to_string());
    m.insert("dateTime".to_string(), "FHIRDateTime".to_string());
    m.insert("time".to_string(), "FHIRTime".to_string());

    m
});

pub static FHIR_PRIMITIVE_VALUE_TYPE: Lazy<HashMap<String, String>> = Lazy::new(|| {
    let mut m = HashMap::new();
    // bool type
    m.insert("boolean".to_string(), "bool".to_string());

    // f64 type
    m.insert("decimal".to_string(), "f64".to_string());

    // i64 type
    m.insert("integer".to_string(), "i64".to_string());
    // u64 type
    m.insert("positiveInt".to_string(), "u64".to_string());
    m.insert("unsignedInt".to_string(), "u64".to_string());

    // String type
    m.insert("base64Binary".to_string(), "String".to_string());
    m.insert("canonical".to_string(), "String".to_string());
    m.insert("code".to_string(), "String".to_string());
    m.insert("date".to_string(), "String".to_string());
    m.insert("dateTime".to_string(), "String".to_string());
    m.insert("id".to_string(), "String".to_string());
    m.insert("instant".to_string(), "String".to_string());
    m.insert("markdown".to_string(), "String".to_string());
    m.insert("oid".to_string(), "String".to_string());
    m.insert("string".to_string(), "String".to_string());
    m.insert("time".to_string(), "String".to_string());
    m.insert("uri".to_string(), "String".to_string());
    m.insert("url".to_string(), "String".to_string());
    m.insert("uuid".to_string(), "String".to_string());
    m.insert("xhtml".to_string(), "String".to_string());

    m
});

pub mod conversion {
    use super::{FHIR_PRIMITIVES, RUST_PRIMITIVES};
    use proc_macro2::TokenStream;
    use quote::{format_ident, quote};
    use serde_json::Value;

    pub fn fhir_type_to_rust_type(element: &Value, fhir_type: &str) -> TokenStream {
        let path = element.get("path").and_then(|p| p.as_str());

        match path {
            Some("unsignedInt.value") | Some("positiveInt.value") => {
                let k = format_ident!("{}", "u64");
                quote! {
                    #k
                }
            }

            _ => {
                if let Some(rust_primitive) = RUST_PRIMITIVES.get(fhir_type) {
                    let k = rust_primitive.parse::<TokenStream>().unwrap();
                    quote! {
                        #k
                    }
                } else if let Some(primitive) = FHIR_PRIMITIVES.get(fhir_type) {
                    let k = format_ident!("{}", primitive.clone());
                    quote! {
                        Box<#k>
                    }
                } else {
                    let k = format_ident!("{}", fhir_type.to_string());
                    quote! {
                        Box<#k>
                    }
                }
            }
        }
    }
}

pub mod extract {
    use serde_json::Value;
    pub fn field_types<'a>(element: &Value) -> Vec<&str> {
        let types = element.get("type").and_then(|t| t.as_array());
        if let Some(types) = types {
            types
                .into_iter()
                .map(|t| t.get("code").unwrap().as_str().unwrap())
                .collect()
        } else {
            vec![]
        }
    }

    pub fn field_name(path: &str) -> String {
        let field_name: String = path
            .split('.')
            .last()
            .unwrap_or("")
            .chars()
            .enumerate()
            .map(|(i, c)| {
                if i == 0 {
                    c.to_lowercase().next().unwrap_or(c)
                } else {
                    c
                }
            })
            .collect();
        let removed_x = if field_name.ends_with("[x]") {
            field_name.replace("[x]", "")
        } else {
            field_name.clone()
        };

        removed_x
    }

    pub fn is_abstract(sd: &Value) -> bool {
        let is_abstract = sd
            .get("abstract")
            .and_then(|r| r.as_bool())
            .unwrap_or(false);
        is_abstract
    }

    pub fn path(element: &Value) -> String {
        element
            .get("path")
            .and_then(|p| p.as_str())
            .unwrap_or("")
            .to_string()
    }
    pub fn element_description(element: &Value) -> String {
        element
            .get("definition")
            .and_then(|d| d.as_str())
            .map(|s| s.to_string())
            .unwrap_or("".to_string())
    }

    pub enum Max {
        Unlimited,
        Fixed(usize),
    }

    pub fn cardinality(element: &Value) -> (usize, Max) {
        let min = element.get("min").and_then(|m| m.as_u64()).unwrap_or(0) as usize;

        let max = element.get("max").and_then(|m| m.as_str()).and_then(|s| {
            if s == "*" {
                Some(Max::Unlimited)
            } else {
                s.parse::<usize>().ok().and_then(|i| Some(Max::Fixed(i)))
            }
        });

        (min, max.unwrap_or_else(|| Max::Fixed(1)))
    }
}

pub mod generate {
    use proc_macro2::TokenStream;
    use quote::{format_ident, quote};
    use serde_json::Value;

    use crate::utilities::{FHIR_PRIMITIVES, conditionals, conversion, extract};

    /// Capitalize the first character in s.
    pub fn capitalize(s: &str) -> String {
        let mut c = s.chars();
        match c.next() {
            None => String::new(),
            Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        }
    }

    pub fn struct_name(sd: &Value, element: &Value) -> String {
        if conditionals::is_root(sd, element) {
            let mut interface_name: String =
                capitalize(sd.get("id").and_then(|p| p.as_str()).unwrap());
            if conditionals::is_primitive_sd(sd) {
                interface_name = "FHIR".to_owned() + &interface_name;
            }
            interface_name
        } else {
            element
                .get("id")
                .and_then(|p| p.as_str())
                .map(|p| p.split("."))
                .map(|p| p.map(capitalize).collect::<Vec<String>>().join(""))
                .unwrap()
                .replace("[x]", "")
        }
    }

    pub fn type_choice_name(sd: &Value, element: &Value) -> String {
        let name = struct_name(sd, element);
        name + "TypeChoice"
    }

    pub fn type_choice_variant_name(element: &Value, fhir_type: &str) -> String {
        let field_name = extract::field_name(&extract::path(element));
        format!("{:0}{:1}", field_name, capitalize(fhir_type))
    }

    pub fn create_type_choice_variants(element: &Value) -> Vec<String> {
        extract::field_types(element)
            .into_iter()
            .map(|fhir_type| type_choice_variant_name(element, fhir_type))
            .collect()
    }
    pub fn create_type_choice_primitive_variants(element: &Value) -> Vec<String> {
        extract::field_types(element)
            .into_iter()
            .filter(|fhir_type| FHIR_PRIMITIVES.contains_key(*fhir_type))
            .map(|fhir_type| type_choice_variant_name(element, fhir_type))
            .collect()
    }

    pub fn field_typename(sd: &Value, element: &Value) -> TokenStream {
        let field_value_type_name = if conditionals::is_typechoice(element) {
            let k = format_ident!("{}", type_choice_name(sd, element));
            quote! {
                #k
            }
        } else if conditionals::is_nested_complex(element) {
            let k = format_ident!("{}", struct_name(sd, element));
            quote! {
                #k
            }
        } else {
            let fhir_type = element.get("type").and_then(|v| v.as_array()).unwrap()[0]
                .get("code")
                .and_then(|v| v.as_str())
                .unwrap();

            conversion::fhir_type_to_rust_type(element, fhir_type)
        };

        field_value_type_name
    }
}

pub mod conditionals {
    use serde_json::Value;

    use crate::utilities::{FHIR_PRIMITIVES, RUST_PRIMITIVES, extract};

    pub fn is_root(sd: &Value, element: &Value) -> bool {
        element.get("path") == sd.get("id")
    }

    pub fn is_resource_sd(sd: &Value) -> bool {
        sd.get("kind").and_then(|k| k.as_str()) == Some("resource")
    }

    pub fn is_primitive(element: &Value) -> bool {
        let types = extract::field_types(element);
        types.len() == 1 && FHIR_PRIMITIVES.contains_key(types[0])
    }

    pub fn is_nested_complex(element: &Value) -> bool {
        let types = extract::field_types(element);
        // Backbone or Typechoice elements Have inlined types created.
        types.len() > 1 || types[0] == "BackboneElement" || types[0] == "Element"
    }

    // All structs should be boxed if they are not rust primitive types.
    pub fn should_be_boxed(fhir_type: &str) -> bool {
        !RUST_PRIMITIVES.contains_key(fhir_type)
    }

    pub fn is_primitive_sd(sd: &Value) -> bool {
        sd.get("kind")
            .and_then(|k| k.as_str())
            .unwrap_or("resource")
            == "primitive-type"
    }

    pub fn is_typechoice(element: &Value) -> bool {
        extract::field_types(element).len() > 1
    }
}

pub mod load {
    use serde_json::Value;

    use crate::utilities::extract;

    pub fn load_from_file(file_path: &str) -> Result<Value, String> {
        let data = std::fs::read_to_string(file_path)
            .map_err(|e| format!("Failed to read file: {}", e))?;

        let json_data: Value =
            serde_json::from_str(&data).map_err(|e| format!("Failed to parse JSON: {}", e))?;

        Ok(json_data)
    }

    pub fn get_structure_definitions<'a>(
        json_data: &'a Value,
        level: Option<&'static str>,
    ) -> Result<impl Iterator<Item = &'a Value>, String> {
        // Extract StructureDefinitions
        let resources = json_data
            .get("entry")
            .and_then(|e| e.as_array())
            .ok_or("No entries found")?
            .into_iter()
            .map(|e| e.get("resource").unwrap())
            .filter(|sd| {
                sd.get("derivation")
                    .and_then(|d| d.as_str())
                    .unwrap_or("specialization")
                    == "specialization"
            })
            .filter(|sd| {
                let resource_type = sd.get("resourceType").and_then(|rt| rt.as_str());
                resource_type == Some("StructureDefinition")
            })
            .filter(|sd| {
                // Filter out the abstract resources particularly Resource + DomainResource
                !(extract::is_abstract(sd)
                    && sd.get("kind").and_then(|k| k.as_str()) == Some("resource"))
            })
            .filter(move |sd| {
                if let Some(level) = level {
                    let kind = sd
                        .get("kind")
                        .and_then(|k| k.as_str())
                        .unwrap_or("resource");

                    kind == level
                } else {
                    true
                }
            });

        Ok(resources)
    }
}
