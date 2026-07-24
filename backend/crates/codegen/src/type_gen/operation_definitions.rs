use std::path::Path;

use crate::utilities::{FHIR_PRIMITIVES, RUST_KEYWORDS, generate::capitalize, load};
use haste_fhir_model::r4::generated::{
    resources::{OperationDefinition, OperationDefinitionParameter, Resource, ResourceType},
    terminology::{AllTypes, OperationParameterUse},
};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use walkdir::WalkDir;

fn get_operation_definitions(resource: &Resource) -> Result<Vec<&OperationDefinition>, String> {
    match resource {
        Resource::Bundle(bundle) => {
            if let Some(entries) = bundle.entry.as_ref() {
                let op_defs = entries
                    .iter()
                    .filter_map(|e| e.resource.as_ref())
                    .filter_map(|sd| match sd.as_ref() {
                        Resource::OperationDefinition(op_def) => Some(op_def),
                        _ => None,
                    });
                Ok(op_defs.collect())
            } else {
                Ok(vec![])
            }
        }
        Resource::OperationDefinition(op_def) => Ok(vec![op_def]),
        _ => Err("Resource is not a Bundle or OperationDefinition".to_string()),
    }
}

fn get_name(op_def: &OperationDefinition) -> String {
    let id = op_def
        .id
        .clone()
        .expect("Operation definition must have an id.");
    id.split('-').map(capitalize).collect::<String>()
}

fn create_field_value(type_: &str, is_array: bool, required: bool) -> TokenStream {
    let base_type = if let Some(primitive) = FHIR_PRIMITIVES.get(type_) {
        primitive.as_str()
    }
    // For element move to ParametersParameterValueTypeChoice
    // This sets it as parameter.parameter.value where it would be pulled from.
    else if type_ == "Element" {
        "ParametersParameterValueTypeChoice"
    } else {
        type_
    };

    let type_ = format_ident!("{}", base_type);

    let type_ = if is_array {
        quote! {Vec<#type_>}
    } else {
        quote! {#type_}
    };

    if required {
        quote! { #type_ }
    } else {
        quote! {Option<#type_>}
    }
}

/// If param is return and type is a resource, you can return resource directly from field.
fn is_resource_return(parameters: &[&OperationDefinitionParameter]) -> bool {
    // Need special handling for single "return" parameter of type Any or a Resource type
    if parameters.len() == 1
        && parameters[0].name.value.as_deref() == Some("return")
        && let Some(parameter_type) = parameters[0].type_.as_ref()
        && (parameter_type == &AllTypes::any()
            || ResourceType::try_from(parameter_type.as_str().unwrap_or_default()).is_ok())
    {
        true
    } else {
        false
    }
}

fn generate_parameter_type(
    name: &str,
    parameters: &Vec<&OperationDefinitionParameter>,
    is_base: bool,
) -> Vec<TokenStream> {
    let mut generated_types = vec![];
    let mut fields = vec![];

    for p in parameters {
        let (field_ident, attribute_rename) = process_field_names(p);
        let description = p
            .documentation
            .as_ref()
            .and_then(|d| d.value.clone())
            .unwrap_or_default();
        let is_array = p.max.value != Some("1".to_string());
        let required = p.min.value.unwrap_or(0) > 0;

        if let Some(type_) = p.type_.as_ref() {
            // Handle parameters with primitive or base types
            let type_str = if type_ == &AllTypes::any() {
                "Resource"
            } else {
                type_.as_str().unwrap_or_default()
            };
            let field_type = create_field_value(type_str, is_array, required);

            fields.push(quote! {
                #[doc = #description]
                #attribute_rename
                pub #field_ident: #field_type
            });
        } else {
            // Handle nested parameters (sub-properties)
            let nested_struct_name = format_nested_name(name, p);
            let nested_types = generate_parameter_type(
                &nested_struct_name,
                &p.part
                    .as_ref()
                    .map(|v| v.iter().collect())
                    .unwrap_or(vec![]),
                false,
            );
            generated_types.extend(nested_types);

            let field_type = create_field_value(&nested_struct_name, is_array, required);
            fields.push(quote! {
                #[doc = #description]
                #attribute_rename
                #[parameter_nested]
                pub #field_ident: #field_type
            });
        }
    }

    let base_type = build_struct_tokens(name, parameters, &fields, is_base);
    generated_types.push(base_type);

    generated_types
}

/// Formats the field name and generates rename attributes, handling reserved Rust keywords.
fn process_field_names(p: &OperationDefinitionParameter) -> (proc_macro2::Ident, TokenStream) {
    let initial_name = p.name.value.as_ref().expect("Parameter must have a name");
    let formatted_name = initial_name.replace('-', "_");

    let field_ident = if RUST_KEYWORDS.contains(&formatted_name.as_str()) {
        format_ident!("{}_", formatted_name)
    } else {
        format_ident!("{}", formatted_name)
    };

    let attribute_rename =
        if RUST_KEYWORDS.contains(&formatted_name.as_str()) || formatted_name != *initial_name {
            quote! { #[parameter_rename=#initial_name] }
        } else {
            quote! {}
        };

    (field_ident, attribute_rename)
}

/// Generates a deterministic nested struct name by combining parent and child names.
fn format_nested_name(parent_name: &str, p: &OperationDefinitionParameter) -> String {
    let initial_name = p.name.value.as_ref().expect("Parameter must have a name");
    let formatted_name = initial_name.replace('-', "_");

    let capitalized_parts = formatted_name
        .split('_')
        .map(capitalize)
        .collect::<String>();

    format!("{parent_name}{capitalized_parts}")
}

/// Constructs the final [`TokenStream`]
/// (Struct definition and From trait implementation)
/// by differentiating between base resource returns and standard parameter wraps.
fn build_struct_tokens(
    name: &str,
    parameters: &Vec<&OperationDefinitionParameter>,
    fields: &[TokenStream],
    is_base: bool,
) -> TokenStream {
    let struct_name = format_ident!("{}", name);

    if is_base && is_resource_return(parameters) {
        let required = parameters.first().and_then(|p| p.min.value).unwrap_or(0) > 0;
        let type_str = parameters
            .first()
            .and_then(|p| {
                p.type_
                    .as_ref()
                    .and_then(haste_fhir_model::r4::generated::terminology::BoundCode::as_str)
            })
            .unwrap_or_default();

        let return_type = if type_str == "Any" {
            "Resource"
        } else {
            type_str
        };
        let return_type_ident = format_ident!("{}", return_type);

        let return_v = if required {
            quote! { value.return_ }
        } else {
            quote! { value.return_.unwrap_or_default() }
        };

        let returned_value = if return_type == "Resource" {
            quote! { #return_v }
        } else {
            quote! { Resource::#return_type_ident(#return_v) }
        };

        quote! {
            #[derive(Debug, FromParameters)]
            pub struct #struct_name {
                #(#fields),*
            }

            impl From<#struct_name> for Resource {
                fn from(value: #struct_name) -> Self {
                    // Special handling for single "return" parameter of type Any or a Resource type
                    #returned_value
                }
            }
        }
    } else {
        quote! {
            #[derive(Debug, FromParameters, ToParameters)]
            pub struct #struct_name {
                #(#fields),*
            }

            impl From<#struct_name> for Resource {
                fn from(value: #struct_name) -> Self {
                    let parameters: Vec<ParametersParameter> = value.into();
                    Resource::Parameters(Parameters {
                        parameter: Some(parameters),
                        ..Default::default()
                    })
                }
            }
        }
    }
}

fn generate_output(parameters: &[OperationDefinitionParameter]) -> Vec<TokenStream> {
    let input_parameters = parameters
        .iter()
        .filter(|p| matches!(&p.use_, use_ if use_ == &OperationParameterUse::out()))
        .collect::<Vec<_>>();

    generate_parameter_type("Output", &input_parameters, true)
}

fn generate_input(parameters: &[OperationDefinitionParameter]) -> Vec<TokenStream> {
    let input_parameters = parameters
        .iter()
        .filter(|p| matches!(&p.use_, use_ if use_ == &OperationParameterUse::in_()))
        .collect::<Vec<_>>();

    generate_parameter_type("Input", &input_parameters, true)
}

fn generate_operation_definition(file_path: &Path) -> Result<TokenStream, String> {
    let resource = load::load_from_file(file_path)?;
    let op_defs = get_operation_definitions(&resource)?;
    // Generate code for each operation definition
    let mut generated = quote! {};
    for op_def in op_defs {
        let name = format_ident!("{}", get_name(op_def));
        let op_code = op_def
            .code
            .value
            .as_ref()
            .expect("Operation must have a code.");
        let parameters: &[OperationDefinitionParameter] =
            op_def.parameter.as_deref().unwrap_or_default();

        let operation_description = op_def
            .description
            .as_ref()
            .and_then(|d| d.value.clone())
            .unwrap_or_default();

        let generate_input = generate_input(parameters);
        let generate_output = generate_output(parameters);

        generated.extend(quote! {
            #[doc = #operation_description]
            pub mod #name {
                use super::*;
                pub const CODE: &str = #op_code;
                #(#generate_input)*
                #(#generate_output)*
            }
            // Code generation for each operation definition
        });
    }

    Ok(generated)
}

/// Generates operation definitions from JSON files in the provided directories.
///
/// # Errors
///
/// Returns an error if an operation definition cannot be generated from one of
/// the input files.
pub fn generate_operation_definitions_from_files(file_paths: &[String]) -> Result<String, String> {
    let mut generated_code = quote! {
        #![allow(non_snake_case)]
        use haste_fhir_ops::derive::{FromParameters, ToParameters};
        use haste_fhir_model::r4::generated::types::*;
        use haste_fhir_model::r4::generated::resources::*;
        use haste_fhir_operation_error::*;
    };

    for dir_path in file_paths {
        let walker = WalkDir::new(dir_path).sort_by_file_name().into_iter();

        for entry in walker
            .filter_map(std::result::Result::ok)
            .filter(|e| e.metadata().is_ok_and(|metadata| metadata.is_file()))
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "json"))
        {
            let generated_types = generate_operation_definition(entry.path())?;

            generated_code = quote! {
                #generated_code
                #generated_types
            };
        }
    }

    Ok(generated_code.to_string())
}
