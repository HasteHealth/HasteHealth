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
    m.insert("return");
    m.insert("async");
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
        "crate::r4::datetime::Time".to_string(),
    );
    m.insert(
        "http://hl7.org/fhirpath/System.Date".to_string(),
        "crate::r4::datetime::Date".to_string(),
    );
    m.insert(
        "http://hl7.org/fhirpath/System.DateTime".to_string(),
        "crate::r4::datetime::DateTime".to_string(),
    );
    m.insert(
        "http://hl7.org/fhirpath/System.Instant".to_string(),
        "crate::r4::datetime::Instant".to_string(),
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
    use std::collections::HashMap;

    use super::{FHIR_PRIMITIVES, RUST_PRIMITIVES};
    use oxidized_fhir_model::r4::generated::types::ElementDefinition;
    use proc_macro2::TokenStream;
    use quote::{format_ident, quote};

    pub fn fhir_type_to_rust_type(
        element: &ElementDefinition,
        fhir_type: &str,
        inlined_terminology: &HashMap<String, String>,
    ) -> TokenStream {
        let path = element.path.value.as_ref().map(|p| p.as_str());

        match path {
            Some("unsignedInt.value") | Some("positiveInt.value") => {
                let k = format_ident!("{}", "u64");
                quote! {
                    #k
                }
            }

            _ => {
                if let Some(rust_primitive) = RUST_PRIMITIVES.get(fhir_type) {
                    // Special handling for instance which should use instant type,
                    let path = path.unwrap();
                    if path == "instant.value" {
                        let k = RUST_PRIMITIVES
                            .get("http://hl7.org/fhirpath/System.Instant")
                            .unwrap()
                            .parse::<TokenStream>()
                            .unwrap();

                        quote! {
                            #k
                        }
                    } else {
                        let k = rust_primitive.parse::<TokenStream>().unwrap();
                        quote! {
                            #k
                        }
                    }
                } else if let Some(primitive) = FHIR_PRIMITIVES.get(fhir_type) {
                    // Support for inlined types.
                    // inlined could be a url | version for canonical.
                    // Only do inlined if the binding is required and exists as inlined terminology.
                    if let Some("required") = element
                        .binding
                        .as_ref()
                        .map(|b| &b.strength)
                        .and_then(|b| b.value.as_ref())
                        .map(|s| s.as_str())
                        && let Some(canonical_string) = element
                            .binding
                            .as_ref()
                            .and_then(|b| b.valueSet.as_ref())
                            .and_then(|b| b.value.as_ref())
                            .map(|u| u.as_str())
                        && let Some(url) = canonical_string.split('|').next()
                        && let Some(inlined) = inlined_terminology.get(url)
                    {
                        let inline_type = format_ident!("{}", inlined);
                        quote! {
                            Box<terminology::#inline_type>
                        }
                    } else {
                        let k = format_ident!("{}", primitive.clone());
                        quote! {
                            Box<#k>
                        }
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
    use oxidized_fhir_model::r4::generated::resources::StructureDefinition;
    use oxidized_fhir_model::r4::generated::types::ElementDefinition;
    pub fn field_types<'a>(element: &ElementDefinition) -> Vec<&str> {
        let codes = element
            .type_
            .as_ref()
            .map(|types| {
                types
                    .iter()
                    .filter_map(|t| t.code.value.as_ref().map(|v| v.as_str()))
                    .collect()
            })
            .unwrap_or_else(|| vec![]);
        codes
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

    pub fn is_abstract(sd: &StructureDefinition) -> bool {
        sd.abstract_.value == Some(true)
    }

    pub fn path(element: &ElementDefinition) -> String {
        element.path.value.clone().unwrap_or_else(|| "".to_string())
    }
    pub fn element_description(element: &ElementDefinition) -> String {
        element
            .definition
            .as_ref()
            .and_then(|d| d.value.as_ref())
            .cloned()
            .unwrap_or_else(|| "".to_string())
    }

    pub enum Max {
        Unlimited,
        Fixed(usize),
    }

    pub fn cardinality(element: &ElementDefinition) -> (usize, Max) {
        let min = element.min.as_ref().and_then(|m| m.value).map_or(0, |m| m) as usize;

        let max = element
            .max
            .as_ref()
            .and_then(|m| m.value.as_ref())
            .map(|v| v.as_str())
            .and_then(|s| {
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
    use std::collections::HashMap;

    use oxidized_fhir_model::r4::generated::{
        resources::StructureDefinition, types::ElementDefinition,
    };
    use proc_macro2::TokenStream;
    use quote::{format_ident, quote};

    use crate::utilities::{FHIR_PRIMITIVES, conditionals, conversion, extract};

    /// Capitalize the first character in s.
    pub fn capitalize(s: &str) -> String {
        let mut c = s.chars();
        match c.next() {
            None => String::new(),
            Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        }
    }

    pub fn struct_name(sd: &StructureDefinition, element: &ElementDefinition) -> String {
        if conditionals::is_root(sd, element) {
            let mut interface_name: String = capitalize(sd.id.as_ref().unwrap());
            if conditionals::is_primitive_sd(sd) {
                interface_name = "FHIR".to_owned() + &interface_name;
            }
            interface_name
        } else {
            element
                .id
                .as_ref()
                .map(|p| p.split("."))
                .map(|p| p.map(capitalize).collect::<Vec<String>>().join(""))
                .unwrap()
                .replace("[x]", "")
        }
    }

    pub fn type_choice_name(sd: &StructureDefinition, element: &ElementDefinition) -> String {
        let name = struct_name(sd, element);
        name + "TypeChoice"
    }

    pub fn type_choice_variant_name(element: &ElementDefinition, fhir_type: &str) -> String {
        let field_name = extract::field_name(&extract::path(element));
        format!("{:0}{:1}", field_name, capitalize(fhir_type))
    }

    pub fn create_type_choice_variants(element: &ElementDefinition) -> Vec<String> {
        extract::field_types(element)
            .into_iter()
            .map(|fhir_type| type_choice_variant_name(element, fhir_type))
            .collect()
    }
    pub fn create_type_choice_primitive_variants(element: &ElementDefinition) -> Vec<String> {
        extract::field_types(element)
            .into_iter()
            .filter(|fhir_type| FHIR_PRIMITIVES.contains_key(*fhir_type))
            .map(|fhir_type| type_choice_variant_name(element, fhir_type))
            .collect()
    }

    pub fn field_typename(
        sd: &StructureDefinition,
        element: &ElementDefinition,
        inlined_terminology: &HashMap<String, String>,
    ) -> TokenStream {
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
            let fhir_type = element.type_.as_ref().unwrap()[0]
                .code
                .as_ref()
                .value
                .as_ref()
                .unwrap();

            conversion::fhir_type_to_rust_type(element, fhir_type, inlined_terminology)
        };

        field_value_type_name
    }
}

pub mod conditionals {
    use oxidized_fhir_model::r4::generated::{
        resources::StructureDefinition, types::ElementDefinition,
    };

    use crate::utilities::{FHIR_PRIMITIVES, RUST_PRIMITIVES, extract};

    pub fn is_root(sd: &StructureDefinition, element: &ElementDefinition) -> bool {
        element.path.value == sd.id
    }

    pub fn is_resource_sd(sd: &StructureDefinition) -> bool {
        sd.kind.value == Some("resource".to_string())
    }

    pub fn is_primitive(element: &ElementDefinition) -> bool {
        let types = extract::field_types(element);
        types.len() == 1 && FHIR_PRIMITIVES.contains_key(types[0])
    }

    pub fn is_nested_complex(element: &ElementDefinition) -> bool {
        let types = extract::field_types(element);
        // Backbone or Typechoice elements Have inlined types created.
        types.len() > 1 || types[0] == "BackboneElement" || types[0] == "Element"
    }

    // All structs should be boxed if they are not rust primitive types.
    pub fn should_be_boxed(fhir_type: &str) -> bool {
        !RUST_PRIMITIVES.contains_key(fhir_type)
    }

    pub fn is_primitive_sd(sd: &StructureDefinition) -> bool {
        sd.kind.value == Some("primitive-type".to_string())
    }

    pub fn is_typechoice(element: &ElementDefinition) -> bool {
        extract::field_types(element).len() > 1
    }
}

pub mod load {
    use std::path::Path;

    use oxidized_fhir_model::r4::generated::resources::{Resource, StructureDefinition};

    use crate::utilities::extract;

    pub fn load_from_file(file_path: &Path) -> Result<Resource, String> {
        let data = std::fs::read_to_string(file_path)
            .map_err(|e| format!("Failed to read file: {}", e))?;

        let resource = oxidized_fhir_serialization_json::from_str::<Resource>(&data)
            .map_err(|e| format!("Failed to parse JSON: {}", e))?;

        Ok(resource)
    }

    pub fn get_structure_definitions<'a>(
        resource: &'a Resource,
        level: Option<&'static str>,
    ) -> Result<Vec<&'a StructureDefinition>, String> {
        match resource {
            Resource::Bundle(bundle) => {
                if let Some(entries) = bundle.entry.as_ref() {
                    let sds = entries
                        .iter()
                        .filter_map(|e| e.resource.as_ref())
                        .filter_map(|sd| match sd.as_ref() {
                            Resource::StructureDefinition(sd) => Some(sd),
                            _ => None,
                        });

                    let filtered_sds = sds.filter(move |sd| {
                        if let Some(level) = level {
                            let kind = sd
                                .kind
                                .as_ref()
                                .value
                                .as_ref()
                                .map(|v| v.as_str())
                                .unwrap_or("resource");
                            kind == level
                        } else {
                            true
                        }
                    });

                    Ok(filtered_sds.collect())
                } else {
                    Ok(vec![])
                }
            }
            Resource::StructureDefinition(sd) => {
                let resources = std::iter::once(sd);
                let filtered_resources = resources.filter(|sd| {
                    if let Some(level) = level {
                        let kind = sd
                            .kind
                            .as_ref()
                            .value
                            .as_ref()
                            .map(|v| v.as_str())
                            .unwrap_or("resource");
                        kind == level
                    } else {
                        true
                    }
                });

                Ok(filtered_resources.collect())
            }
            _ => Err("Resource is not a Bundle or StructureDefinition".to_string()),
        }
    }
}
