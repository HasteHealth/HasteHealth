use std::{borrow::Cow, path::Path};

use crate::utilities::{generate::capitalize, load};
use oxidized_fhir_model::r4::types::{OperationDefinition, OperationDefinitionParameter, Resource};
use proc_macro2::TokenStream;
use quote::quote;
use walkdir::WalkDir;

pub fn get_operation_definitions<'a>(
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

fn generate_parameter_type(parameters: &Vec<&OperationDefinitionParameter>) -> TokenStream {
    let types = vec![];
    for p in parameters.iter() {
        let is_array = p.max.value != Some("1".to_string());
        let required = p.min.value.unwrap_or(0) > 0;
        let field_name = p.name.value.as_ref().expect("Parameter must have a name");

        if let Some(type_) = p.type_.as_ref().and_then(|v| v.value.as_ref()) {
        } else {
        }
    }

    panic!();

    //   const fields = parameters
    //     .map((p) => {
    //       const isArray = p.max !== "1";
    //       const required = p.min > 0;

    //       const fieldName = `"${p.name}"${required ? "" : "?"}`;
    //       const type = p.type === "Any" ? "Resource" : p.type;

    //       const singularValue = p.type
    //         ? `fhirTypes.${type}`
    //         : generateParameterType(p.part || []);
    //       const value = isArray ? `Array<${singularValue}>` : singularValue;

    //       return `${fieldName}: ${value}`;
    //     })
    //     .join(",\n");
    //   return `{${fields}}`;
}

fn generate_output(parameters: &Cow<Vec<OperationDefinitionParameter>>) -> TokenStream {}

fn generate_input(parameters: &Cow<Vec<OperationDefinitionParameter>>) -> TokenStream {
    let input_parameters = parameters
        .iter()
        .filter(|p| {
            p.use_value
                .as_ref()
                .map(|u| u.as_str() == "in")
                .unwrap_or(false)
        })
        .collect::<Vec<_>>();

    generate_parameter_type(&input_parameters)
}

fn generate_operation_definition(file_path: &Path) -> Result<TokenStream, String> {
    let resource = load::load_from_file(file_path)?;
    let op_defs = get_operation_definitions(&resource)?;
    // Generate code for each operation definition
    let mut generated = quote! {};
    for op_def in op_defs {
        let name = get_name(op_def);
        let parameters = op_def
            .parameter
            .as_ref()
            .map(Cow::Borrowed)
            .unwrap_or(Cow::Owned(vec![]));

        let generate_input = generate_input(&parameters);
        let generate_output = generate_output(&parameters);

        generated.extend(quote! {
            mod #name {
                #generate_input
                #generate_output
            }
            // Code generation for each operation definition
        });
    }

    Ok(generated)
}

pub fn generate_operation_definitions(file_paths: &Vec<String>) -> Result<String, String> {
    let mut generated_code = quote! {
        #![allow(non_snake_case)]
        use oxidized_reflect::{MetaValue, derive::Reflect};
        use oxidized_fhir_serialization_json;
        use oxidized_fhir_serialization_json::FHIRJSONDeserializer;
        use thiserror::Error;
        use std::io::Write;
    };

    let mut resource_types: Vec<String> = vec![];

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
