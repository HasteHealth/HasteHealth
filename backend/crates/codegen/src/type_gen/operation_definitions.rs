use std::{borrow::Cow, path::Path};

use crate::utilities::{FHIR_PRIMITIVES, RUST_KEYWORDS, generate::capitalize, load};
use haste_fhir_model::r4::generated::{
    resources::{OperationDefinition, OperationDefinitionParameter, Resource, ResourceType},
    terminology::{AllTypes, OperationParameterUse},
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

/// If param is return and type is a resource, you can return resource directly from field.
fn is_resource_return(parameters: &Vec<&OperationDefinitionParameter>) -> bool {
    // Need special handling for single "return" parameter of type Any or a Resource type
    if parameters.len() == 1
        && parameters[0].name.value.as_deref() == Some("return")
        && let Some(parameter_type) = parameters[0]
            .type_
            .as_ref()
        
        && (std::mem::discriminant(&**parameter_type) == std::mem::discriminant(&AllTypes::Any(None)) 
            || ResourceType::try_from(Into::<Option<String>>::into(&**parameter_type).unwrap_or_default()).is_ok())
    {
        true
    } else {
        false
    }
}

fn generate_parameter_type(
    name: &str,
    parameters: &Vec<&OperationDefinitionParameter>,
    direction: &Direction,
    is_base: bool,
) -> Vec<TokenStream> {
    let mut types = vec![];
    let mut fields = vec![];

    for p in parameters.iter() {
        let is_array = p.max.value != Some("1".to_string());
        let required = p.min.value.unwrap_or(0) > 0;
        let initial_field_name = p.name.value.as_ref().expect("Parameter must have a name");
        let formatted_field_name = initial_field_name.replace("-", "_");

        let field_ident = if RUST_KEYWORDS.contains(&formatted_field_name.as_str()) {
            format_ident!("{}_", formatted_field_name)
        } else {
            format_ident!("{}", formatted_field_name)
        };

        let attribute_rename = if RUST_KEYWORDS.contains(&formatted_field_name.as_str())
            || formatted_field_name != *initial_field_name
        {
            quote! {  #[parameter_rename=#initial_field_name] }
        } else {
            quote! {}
        };

        if let Some(type_) = p.type_.as_ref() {
            let type_ = if std::mem::discriminant(&**type_) == std::mem::discriminant(&AllTypes::Any(None)) { AllTypes::Resource(None) } else { *type_.clone() };
            let field = create_field_value(Into::<Option<String>>::into(&type_).unwrap_or_default().as_str(), is_array, required);

            fields.push(quote! {
                #attribute_rename
                pub #field_ident: #field
            })
        } else {
            let name = name.to_string()
                + formatted_field_name
                    .split("_")
                    .map(|s| capitalize(s))
                    .collect::<String>()
                    .as_str();
            let nested_types = generate_parameter_type(
                &name,
                &p.part
                    .as_ref()
                    .map(|v| v.iter().collect())
                    .unwrap_or(vec![]),
                direction,
                false,
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

    let base_parameter_type = if is_base && is_resource_return(parameters) {
        let required = parameters.get(0).and_then(|p| p.min.value).unwrap_or(0) > 0;
        let type_ = parameters
            .get(0)
            .and_then(|p| {
                p.type_
                    .as_ref()
                    .and_then(|v| Into::<Option<String>>::into(&**v))
            })
            .unwrap_or_default();

        let return_type = if type_ == "Any" { "Resource" } else { type_.as_str() };
        let return_type_ident = format_ident!("{}", return_type);

        let return_v = if required {
            quote! {
                value.return_
            }
        } else {
            quote! {
               value.return_.unwrap_or_default()
            }
        };

        let returned_value = if return_type == "Resource" {
            quote! {#return_v}
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

    generate_parameter_type("Output", &input_parameters, &Direction::Output, true)
}

fn generate_input(parameters: &Cow<Vec<OperationDefinitionParameter>>) -> Vec<TokenStream> {
    let input_parameters = parameters
        .iter()
        .filter(|p| match p.use_.as_ref() {
            OperationParameterUse::In(_) => true,
            _ => false,
        })
        .collect::<Vec<_>>();

    generate_parameter_type("Input", &input_parameters, &Direction::Input, true)
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
        let op_code = op_def
            .code
            .value
            .as_ref()
            .expect("Operation must have a code.");
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
                pub const CODE: &str = #op_code;
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
        use haste_fhir_ops::derive::{FromParameters, ToParameters};
        use haste_fhir_model::r4::generated::types::*;
        use haste_fhir_model::r4::generated::resources::*;
        use haste_fhir_operation_error::*;
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
