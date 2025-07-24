use crate::{
    traversal,
    utilities::{
        RUST_KEYWORDS, conditionals,
        conversion::fhir_type_to_rust_type,
        extract,
        generate::{self, field_typename},
        load,
    },
};
use indexmap::IndexMap;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use serde_json::Value;

type NestedTypes = IndexMap<String, TokenStream>;

fn wrap_cardinality_and_optionality(element: &Value, field_value: TokenStream) -> TokenStream {
    let cardinality = extract::cardinality(element);

    // Check the cardinality.
    let field_value = match cardinality.1 {
        extract::Max::Unlimited => quote! {
            Vec<#field_value>
        },
        extract::Max::Fixed(1) => quote! {
            #field_value
        },
        extract::Max::Fixed(_n) => quote! {
            Vec<#field_value>
        },
    };

    // Check the Optionality
    if cardinality.0 == 0 {
        quote! {
            Option<#field_value>
        }
    } else {
        field_value
    }
}

fn get_struct_key_value(element: &Value, field_value_type_name: TokenStream) -> TokenStream {
    let description = extract::element_description(element);
    let field_name = extract::field_name(&extract::path(element));
    let field_name_ident = if RUST_KEYWORDS.contains(&field_name.as_str()) {
        format_ident!("{}_", field_name)
    } else {
        format_ident!("{}", field_name)
    };

    let reflect_attribute = if RUST_KEYWORDS.contains(&field_name.as_str()) {
        quote! {
            #[rename_field = #field_name]
        }
    } else {
        quote! {}
    };

    let type_choice_variants = if conditionals::is_typechoice(element) {
        let type_choice_variants = generate::create_type_choice_variants(element);
        let type_choice_primitives = generate::create_type_choice_primitive_variants(element);
        let type_choice_complex_variants = type_choice_variants
            .iter()
            .filter(|variant| !type_choice_primitives.contains(variant));

        quote! {
           #[type_choice_variants(complex = [#(#type_choice_complex_variants),*], primitive = [#(#type_choice_primitives),*])]
        }
    } else {
        quote! {}
    };

    let primitive_attribute = if conditionals::is_primitive(element) {
        quote! {
        #[primitive]
        }
    } else {
        quote! {}
    };

    let field_value = wrap_cardinality_and_optionality(element, field_value_type_name);

    quote! {
        #type_choice_variants
        #reflect_attribute
        #primitive_attribute
        #[doc = #description]
        pub #field_name_ident: #field_value
    }
}

fn resolve_content_reference<'a>(sd: &'a Value, element: &Value) -> &'a Value {
    let content_reference_id =
        element.get("contentReference").unwrap().as_str().unwrap()[1..].to_string();

    let content_reference_element: Vec<&Value> = sd
        .get("snapshot")
        .ok_or("StructureDefinition has no snapshot")
        .unwrap()
        .get("element")
        .ok_or("StructureDefinition has no elements")
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e.get("id").unwrap().as_str().unwrap() == &content_reference_id)
        .collect();

    if content_reference_element.len() != 1 {
        panic!(
            "Content reference element not found {}",
            content_reference_id
        );
    }

    let content_reference_element = content_reference_element[0];
    content_reference_element
}

fn create_type_choice(sd: &Value, element: &Value) -> TokenStream {
    let field_name = extract::field_name(&extract::path(element));
    let type_name = format_ident!("{}", generate::type_choice_name(sd, element));
    let types = extract::field_types(element);

    let enum_variants = types
        .iter()
        .map(|fhir_type| {
            let enum_name = format_ident!("{}", generate::capitalize(fhir_type));
            let rust_type = fhir_type_to_rust_type(element, fhir_type);

            quote! {
                #enum_name(#rust_type)
            }
        })
        .collect::<Vec<TokenStream>>();

    let default_enum = format_ident!("{}", generate::capitalize(&types[0].to_string()));
    let default_impl = if conditionals::should_be_boxed(&types[0].to_string()) {
        quote! {
            impl Default for #type_name {
                fn default() -> Self {
                    #type_name::#default_enum(Box::new(Default::default()))
                }
            }
        }
    } else {
        quote! {
            impl Default for #type_name {
                fn default() -> Self {
                    #type_name::#default_enum(Default::default())
                }
            }
        }
    };

    // fhir_serialization_json::derive::FHIRJSONDeserialize
    quote! {
        #[derive(Clone, Reflect, Debug, fhir_serialization_json::derive::FHIRJSONSerialize, fhir_serialization_json::derive::FHIRJSONDeserialize)]
        #[fhir_serialize_type = "typechoice"]
        #[type_choice_field_name = #field_name]
        pub enum #type_name {
            #(#enum_variants),*
        }
        #default_impl
    }
}

fn process_leaf(sd: &Value, element: &Value, types: &mut NestedTypes) -> TokenStream {
    if element.get("contentReference").is_some() {
        let content_reference_element = resolve_content_reference(sd, element);
        let field_type_name = field_typename(sd, content_reference_element);
        get_struct_key_value(element, field_type_name)
    } else if conditionals::is_typechoice(element) {
        let type_choice_name_ident = field_typename(sd, element);
        let type_choice = create_type_choice(sd, element);

        types.insert(type_choice_name_ident.to_string(), type_choice);

        get_struct_key_value(element, quote! {#type_choice_name_ident})
    } else {
        let fhir_type = extract::field_types(element)[0];
        let rust_type = fhir_type_to_rust_type(element, fhir_type);

        get_struct_key_value(element, rust_type)
    }
}

fn process_complex(
    sd: &Value,
    element: &Value,
    children: Vec<TokenStream>,
    types: &mut NestedTypes,
) -> TokenStream {
    let interface_name = generate::struct_name(sd, element);
    let i = format_ident!("{}", interface_name.clone());
    let description = extract::element_description(element);

    let derive = if conditionals::is_root(sd, element) && conditionals::is_primitive_sd(sd) {
        quote! {
           #[derive(Clone, Reflect, Debug, Default, fhir_serialization_json::derive::FHIRJSONSerialize, fhir_serialization_json::derive::FHIRJSONDeserialize)]
           #[fhir_serialize_type = "primitive"]
        }
    } else if conditionals::is_root(sd, element) && conditionals::is_resource_sd(sd) {
        quote! {
            #[derive(Clone, Reflect, Debug, Default, fhir_serialization_json::derive::FHIRJSONSerialize, fhir_serialization_json::derive::FHIRJSONDeserialize)]
            #[fhir_serialize_type = "resource"]
        }
    } else {
        quote! {
            #[derive(Clone, Reflect, Debug, Default, fhir_serialization_json::derive::FHIRJSONSerialize, fhir_serialization_json::derive::FHIRJSONDeserialize)]
            #[fhir_serialize_type = "complex"]
        }
    };

    let type_value = quote! {
        #derive
        #[doc = #description]
        pub struct #i {
            #(#children),*
        }
    };

    let i = interface_name.clone();
    types.insert(i, type_value);
    let i = format_ident!("{}", interface_name.clone());
    get_struct_key_value(element, quote! {#i})
}

fn generate_from_structure_definition(sd: &Value) -> Result<TokenStream, String> {
    let mut nested_types = IndexMap::<String, TokenStream>::new();

    let mut visitor = |element: &Value, children: Vec<TokenStream>, _index: usize| -> TokenStream {
        if children.len() == 0 {
            process_leaf(&sd, element, &mut nested_types)
        } else {
            process_complex(&sd, element, children, &mut nested_types)
        }
    };

    traversal::traversal(sd, &mut visitor)?;
    let types_generated = nested_types.values();

    let generated_code = quote! {
        #(#types_generated)*
    };

    Ok(generated_code)
}

pub struct GeneratedTypes {
    types: Vec<TokenStream>,
    resource_types: Vec<String>,
}

pub fn generate_fhir_types_from_file(
    file_path: &str,
    level: Option<&'static str>,
) -> Result<GeneratedTypes, String> {
    let json_data = load::load_from_file(file_path)?;
    // Extract StructureDefinitions
    let structure_definitions = load::get_structure_definitions(&json_data, level)
        .map_err(|e| format!("Failed to get structure definitions: {}", e))?;

    let mut generated_code = vec![];
    let mut resource_types: Vec<String> = vec![];

    for sd in structure_definitions {
        if conditionals::is_resource_sd(&sd) {
            resource_types.push(sd.get("id").and_then(|id| id.as_str()).unwrap().to_string());
        }
        generated_code.push(generate_from_structure_definition(sd)?);
    }

    Ok(GeneratedTypes {
        types: generated_code,
        resource_types: resource_types,
    })
}

fn generate_resource_type(resource_types: &Vec<String>) -> TokenStream {
    let count = resource_types.len();

    quote! {
        #[derive(Error, Debug)]
        pub enum ResourceTypeError {
            #[error("Invalid resource type: {0}")]
            Invalid(String),
        }

        static _RESOURCE_TYPES: [&str; #count] = [
            #(#resource_types),*
        ];

        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub struct ResourceType(String);
        impl ResourceType {
            pub fn new(s: String) -> Result<Self, ResourceTypeError> {
                if !_RESOURCE_TYPES.contains(&s.as_str()) {
                    Err(ResourceTypeError::Invalid(s))
                } else {
                    Ok(ResourceType(s))
                }
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }

            pub unsafe fn unchecked(s: String) -> Self {
                ResourceType(s)
            }
        }
    }
}

pub fn generate_fhir_types_from_files(
    file_paths: &Vec<String>,
    level: Option<&'static str>,
) -> Result<String, String> {
    let mut generated_code = quote! {
        #![allow(non_snake_case)]
        use reflect::{MetaValue, derive::Reflect};
        use fhir_serialization_json;
        use fhir_serialization_json::FHIRJSONDeserializer;
        use thiserror::Error;
        use std::io::Write;
    };

    let mut resource_types: Vec<String> = vec![];

    for file_path in file_paths {
        let generated_types = generate_fhir_types_from_file(file_path, level)?;
        let code = generated_types.types;
        resource_types.extend(generated_types.resource_types);

        generated_code = quote! {
            #generated_code
            #(#code)*
        }
    }

    let resource_type_enum_variant_idents = resource_types
        .iter()
        .map(|resource_name| format_ident!("{}", resource_name))
        .map(|variant| {
            let enum_variant = variant.clone();
            quote! {
                #enum_variant(#variant)
            }
        });

    let resource_enum = quote! {
        #[derive(Clone, Reflect, Debug, fhir_serialization_json::derive::FHIRJSONSerialize, fhir_serialization_json::derive::FHIRJSONDeserialize)]
        #[fhir_serialize_type = "enum-variant"]
        #[determine_by = "resourceType"]
        pub enum Resource {
            #(#resource_type_enum_variant_idents),*
        }
    };

    let resource_type_type = generate_resource_type(&resource_types);

    Ok(quote! {
        #generated_code
        #resource_enum
        #resource_type_type
    }
    .to_string())
}
