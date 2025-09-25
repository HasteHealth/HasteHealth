use std::{borrow::Cow, path::Path};

use crate::utilities::{FHIR_PRIMITIVES, RUST_KEYWORDS, generate::capitalize, load};
use oxidized_fhir_model::r4::generated::{
    resources::{OperationDefinition, OperationDefinitionParameter, Resource},
    terminology::OperationParameterUse,
};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use walkdir::WalkDir;

fn get_operation_definitions<'a>(
    resource: &'a Resource,
) -> Result<Vec<&'a OperationDefinition>, String> {
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
        Resource::OperationDefinition(op_def) => {
            let op_def = op_def;

            Ok(vec![op_def])
        }
        _ => Err("Resource is not a Bundle or OperationDefinition".to_string()),
    }
}

fn get_name(op_def: &OperationDefinition) -> String {
    let id = op_def
        .id
        .clone()
        .expect("Operation definition must have an id.");
    let interface_name = id
        .split("-")
        .into_iter()
        .map(|s| capitalize(s))
        .collect::<Vec<String>>()
        .join("");
    interface_name
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

    let type_ = if required {
        quote! { #type_ }
    } else {
        quote! {Option<#type_>}
    };

    type_
}

fn generate_parameter_type(
    name: &str,
    parameters: &Vec<&OperationDefinitionParameter>,
    direction: &Direction,
) -> Vec<TokenStream> {
    let mut types = vec![];
    let mut fields = vec![];

    for p in parameters.iter() {
        let is_array = p.max.value != Some("1".to_string());
        let required = p.min.value.unwrap_or(0) > 0;
        let field_name = p
            .name
            .value
            .as_ref()
            .expect("Parameter must have a name")
            .replace("-", "_");

        let field_ident = if RUST_KEYWORDS.contains(&field_name.as_str()) {
            format_ident!("{}_", field_name)
        } else {
            format_ident!("{}", field_name)
        };

        let attribute_rename = if RUST_KEYWORDS.contains(&field_name.as_str()) {
            quote! {  #[parameter_rename=#field_name] }
        } else {
            quote! {}
        };

        if let Some(type_) = p.type_.as_ref().and_then(|v| v.value.as_ref()) {
            let type_ = if type_ == "Any" { "Resource" } else { type_ };
            let type_ = create_field_value(type_, is_array, required);

            fields.push(quote! {
                #attribute_rename
                pub #field_ident: #type_
            })
        } else {
            let name = name.to_string() + &capitalize(field_name.as_str());
            let nested_types = generate_parameter_type(
                &name,
                &p.part
                    .as_ref()
                    .map(|v| v.iter().collect())
                    .unwrap_or(vec![]),
                direction,
            );
            types.extend(nested_types);

            let type_ = create_field_value(&name, is_array, required);
            fields.push(quote! {
                #attribute_rename
                #[parameter_nested]
                pub #field_ident: #type_
            })
        }
    }

    let struct_name = format_ident!("{}", name);

    let base_parameter_type = quote! {
        #[derive(Debug, ParametersParse)]
        pub struct #struct_name {
            #(#fields),*
        }
    };

    types.push(base_parameter_type);

    types
}

fn generate_output(parameters: &Cow<Vec<OperationDefinitionParameter>>) -> Vec<TokenStream> {
    let input_parameters = parameters
        .iter()
        .filter(|p| match p.use_.as_ref() {
            OperationParameterUse::Out(_) => true,
            _ => false,
        })
        .collect::<Vec<_>>();

    generate_parameter_type("Output", &input_parameters, &Direction::Output)
}

fn generate_input(parameters: &Cow<Vec<OperationDefinitionParameter>>) -> Vec<TokenStream> {
    let input_parameters = parameters
        .iter()
        .filter(|p| match p.use_.as_ref() {
            OperationParameterUse::In(_) => true,
            _ => false,
        })
        .collect::<Vec<_>>();

    generate_parameter_type("Input", &input_parameters, &Direction::Input)
}

enum Direction {
    Input,
    Output,
}

fn generate_operation_definition(file_path: &Path) -> Result<TokenStream, String> {
    let resource = load::load_from_file(file_path)?;
    let op_defs = get_operation_definitions(&resource)?;
    // Generate code for each operation definition
    let mut generated = quote! {};
    for op_def in op_defs {
        let name = format_ident!("{}", get_name(op_def));
        let parameters = op_def
            .parameter
            .as_ref()
            .map(Cow::Borrowed)
            .unwrap_or(Cow::Owned(vec![]));

        let generate_input = generate_input(&parameters);
        let generate_output = generate_output(&parameters);

        generated.extend(quote! {
            pub mod #name {
                use super::*;
                #(#generate_input)*
                #(#generate_output)*
            }
            // Code generation for each operation definition
        });
    }

    Ok(generated)
}

pub fn generate_operation_definitions_from_files(
    file_paths: &Vec<String>,
) -> Result<String, String> {
    let mut generated_code = quote! {
        #![allow(non_snake_case)]
        use oxidized_fhir_ops::derive::ParametersParse;
        use oxidized_fhir_model::r4::generated::types::*;
        use oxidized_fhir_model::r4::generated::resources::*;
        use oxidized_fhir_operation_error::*;
    };

    for dir_path in file_paths {
        let walker = WalkDir::new(dir_path).into_iter();
        for entry in walker
            .filter_map(|e| e.ok())
            .filter(|e| e.metadata().unwrap().is_file())
        {
            let generated_types = generate_operation_definition(entry.path())?;

            generated_code = quote! {
                #generated_code
                #generated_types
            }
        }
    }

    Ok(generated_code.to_string())
}
