use std::path::Path;

use crate::documentation::{format_documentation, generate_doc_attributes};
use crate::utilities::{FHIR_PRIMITIVES, RUST_KEYWORDS, generate::capitalize, load};
use haste_fhir_model::r4::generated::{
    resources::{OperationDefinition, OperationDefinitionParameter, Resource, ResourceType},
    terminology::{AllTypes, BoundCode, OperationParameterUse},
};
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use walkdir::WalkDir;

fn get_operation_definitions(resource: &Resource) -> Result<Vec<&OperationDefinition>, String> {
    match resource {
        Resource::Bundle(bundle) => Ok(bundle
            .entry
            .as_ref()
            .map(|entries| {
                entries
                    .iter()
                    .filter_map(|entry| entry.resource.as_ref())
                    .filter_map(|resource| match resource.as_ref() {
                        Resource::OperationDefinition(op_def) => Some(op_def),
                        _ => None,
                    })
                    .collect()
            })
            .unwrap_or_default()),
        Resource::OperationDefinition(op_def) => Ok(vec![op_def]),
        _ => Err("Resource is not a Bundle or OperationDefinition".to_string()),
    }
}

fn get_name(op_def: &OperationDefinition) -> String {
    op_def
        .id
        .clone()
        .expect("Operation definition must have an id.")
        .split('-')
        .map(capitalize)
        .collect()
}

fn parameter_type_name(type_: &str) -> &str {
    if let Some(primitive) = FHIR_PRIMITIVES.get(type_) {
        primitive.as_str()
    } else if type_ == "Element" {
        "ParametersParameterValueTypeChoice"
    } else {
        type_
    }
}

fn parameter_type_ident(type_: &str) -> Ident {
    format_ident!("{}", parameter_type_name(type_))
}

fn create_field_value(type_: &str, is_array: bool, required: bool) -> TokenStream {
    let type_ident = parameter_type_ident(type_);

    let field_type = if is_array {
        quote! { Vec<#type_ident> }
    } else {
        quote! { #type_ident }
    };

    if required {
        quote! { #field_type }
    } else {
        quote! { Option<#field_type> }
    }
}

fn is_resource_return(parameters: &[&OperationDefinitionParameter]) -> bool {
    parameters.len() == 1
        && parameters[0].name.value.as_deref() == Some("return")
        && parameters[0].type_.as_ref().is_some_and(|parameter_type| {
            parameter_type == &AllTypes::any()
                || ResourceType::try_from(parameter_type.as_str().unwrap_or_default()).is_ok()
        })
}

fn generate_parameter_type(
    name: &str,
    parameters: &[&OperationDefinitionParameter],
    is_base: bool,
) -> Vec<TokenStream> {
    let mut generated_types = Vec::new();
    let mut fields = Vec::with_capacity(parameters.len());

    for parameter in parameters {
        let (field_ident, attribute_rename) = process_field_names(parameter);

        let description = parameter
            .documentation
            .as_ref()
            .and_then(|documentation| documentation.value.as_deref())
            .map_or_else(|| field_ident.to_string(), format_documentation);

        let doc_attributes = generate_doc_attributes(&description);

        let is_array = parameter.max.value.as_deref() != Some("1");
        let required = parameter.min.value.unwrap_or_default() > 0;

        let (field_type, nested_attribute) = if let Some(type_) = parameter.type_.as_ref() {
            let type_name = if type_ == &AllTypes::any() {
                "Resource"
            } else {
                type_.as_str().unwrap_or_default()
            };

            (create_field_value(type_name, is_array, required), quote! {})
        } else {
            let nested_struct_name = format_nested_name(name, parameter);

            let nested_parameters = parameter
                .part
                .as_deref()
                .unwrap_or_default()
                .iter()
                .collect::<Vec<_>>();

            generated_types.extend(generate_parameter_type(
                &nested_struct_name,
                &nested_parameters,
                false,
            ));

            (
                create_field_value(&nested_struct_name, is_array, required),
                quote! {
                    #[parameter_nested]
                },
            )
        };

        fields.push(quote! {
            #doc_attributes
            #attribute_rename
            #nested_attribute
            pub #field_ident: #field_type
        });
    }

    generated_types.push(build_struct_tokens(name, parameters, &fields, is_base));

    generated_types
}

fn process_field_names(p: &OperationDefinitionParameter) -> (Ident, TokenStream) {
    let initial_name = p.name.value.as_deref().expect("Parameter must have a name");

    let replaced_name = initial_name.replace('-', "_");

    // A leading `_` is a FHIR naming convention and is not meaningful to the
    // generated Rust field name. Preserve the original FHIR name through
    // #[parameter_rename = "..."] instead.
    let formatted_name = replaced_name
        .strip_prefix('_')
        .unwrap_or(&replaced_name)
        .to_string();

    let is_rust_keyword = RUST_KEYWORDS.contains(&formatted_name.as_str());

    let field_ident = if is_rust_keyword {
        format_ident!("{}_", formatted_name)
    } else {
        format_ident!("{}", formatted_name)
    };

    let attribute_rename = if formatted_name != *initial_name || is_rust_keyword {
        quote! {
            #[parameter_rename = #initial_name]
        }
    } else {
        quote! {}
    };

    (field_ident, attribute_rename)
}

fn format_nested_name(parent_name: &str, parameter: &OperationDefinitionParameter) -> String {
    let initial_name = parameter
        .name
        .value
        .as_deref()
        .expect("Parameter must have a name");

    let formatted_name = initial_name.replace('-', "_");

    let capitalized_parts = formatted_name
        .split('_')
        .map(capitalize)
        .collect::<String>();

    format!("{parent_name}{capitalized_parts}")
}

fn resource_return_tokens(parameters: &[&OperationDefinitionParameter]) -> Option<TokenStream> {
    if !is_resource_return(parameters) {
        return None;
    }

    let parameter = parameters.first()?;

    let required = parameter.min.value.unwrap_or_default() > 0;

    let type_str = parameter
        .type_
        .as_ref()
        .and_then(BoundCode::as_str)
        .unwrap_or_default();

    let return_type = if type_str == "Any" {
        "Resource"
    } else {
        type_str
    };

    let return_type_ident = format_ident!("{}", return_type);

    let return_value = if required {
        quote! { value.return_ }
    } else {
        quote! { value.return_.unwrap_or_default() }
    };

    Some(if return_type == "Resource" {
        quote! { #return_value }
    } else {
        quote! { Resource::#return_type_ident(#return_value) }
    })
}

/// Constructs the final `TokenStream` (struct definition and `From`
/// implementation) by differentiating between base resource returns and
/// standard parameter wraps.
fn build_struct_tokens(
    name: &str,
    parameters: &[&OperationDefinitionParameter],
    fields: &[TokenStream],
    is_base: bool,
) -> TokenStream {
    let struct_name = format_ident!("{}", name);

    if is_base && let Some(returned_value) = resource_return_tokens(parameters) {
        return quote! {
            #[derive(Debug, FromParameters)]
            pub struct #struct_name {
                #(#fields),*
            }

            impl From<#struct_name> for Resource {
                fn from(value: #struct_name) -> Self {
                    #returned_value
                }
            }
        };
    }

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

fn generate_parameters(
    parameters: &[OperationDefinitionParameter],
    parameter_use: &BoundCode<OperationParameterUse>,
    name: &str,
) -> Vec<TokenStream> {
    let parameters = parameters
        .iter()
        .filter(|parameter| parameter.use_ == *parameter_use)
        .collect::<Vec<_>>();

    generate_parameter_type(name, &parameters, true)
}

fn generate_output(parameters: &[OperationDefinitionParameter]) -> Vec<TokenStream> {
    generate_parameters(parameters, &OperationParameterUse::out(), "Output")
}

fn generate_input(parameters: &[OperationDefinitionParameter]) -> Vec<TokenStream> {
    generate_parameters(parameters, &OperationParameterUse::in_(), "Input")
}

struct OperationImports {
    resources: Vec<Ident>,
    types: Vec<Ident>,
}

impl OperationImports {
    fn new() -> Self {
        Self {
            resources: Vec::new(),
            types: vec![format_ident!("FHIRString")],
        }
    }

    fn add_resource(&mut self, name: &str) {
        self.resources.push(format_ident!("{}", name));
    }

    fn add_type(&mut self, name: &str) {
        self.types.push(format_ident!("{}", name));
    }

    fn sort_and_dedup(&mut self) {
        Self::sort_and_dedup_idents(&mut self.resources);
        Self::sort_and_dedup_idents(&mut self.types);
    }

    fn sort_and_dedup_idents(idents: &mut Vec<Ident>) {
        idents.sort_by_key(std::string::ToString::to_string);
        idents.dedup_by_key(|ident| ident.to_string());
    }
}

fn add_parameter_type(imports: &mut OperationImports, parameter: &OperationDefinitionParameter) {
    if let Some(type_) = parameter.type_.as_ref() {
        let type_name = if type_ == &AllTypes::any() {
            "Resource"
        } else {
            type_.as_str().unwrap_or_default()
        };

        if type_name == "Resource" {
            imports.add_resource("Resource");
        } else if type_name == "Element" {
            imports.add_resource("ParametersParameterValueTypeChoice");
        } else if let Some(primitive) = FHIR_PRIMITIVES.get(type_name) {
            imports.add_type(primitive);
        } else if ResourceType::try_from(type_name).is_ok() {
            imports.add_resource(type_name);
        } else {
            imports.add_type(type_name);
        }
    }

    for part in parameter.part.as_deref().unwrap_or_default() {
        add_parameter_type(imports, part);
    }
}

fn collect_imports(parameters: &[OperationDefinitionParameter]) -> OperationImports {
    let mut imports = OperationImports::new();

    imports.add_resource("Parameters");
    imports.add_resource("ParametersParameter");
    imports.add_resource("Resource");

    for parameter in parameters {
        add_parameter_type(&mut imports, parameter);
    }

    imports.sort_and_dedup();
    imports
}

/* ------------------------------------------------------------------------- */
/* Operation generation                                                       */
/* ------------------------------------------------------------------------- */

fn generate_operation_definition(file_path: &Path) -> Result<TokenStream, String> {
    let resource = load::load_from_file(file_path)?;
    let op_defs = get_operation_definitions(&resource)?;

    let mut generated = quote! {};

    for op_def in op_defs {
        let name = format_ident!("{}", get_name(op_def));

        let op_code = op_def
            .code
            .value
            .as_ref()
            .expect("Operation must have a code.");

        let parameters = op_def.parameter.as_deref().unwrap_or_default();

        let operation_description = op_def
            .description
            .as_ref()
            .and_then(|description| description.value.as_deref())
            .map(format_documentation)
            .unwrap_or_default();

        let operation_doc_attributes = generate_doc_attributes(&operation_description);

        let mut imports = collect_imports(parameters);

        if name == "ActivityDefinitionDataRequirements" || name == "PlanDefinitionDataRequirements"
        {
            imports.types.retain(|ident| ident != "FHIRString");
        }

        let generated_input = generate_input(parameters);
        let generated_output = generate_output(parameters);

        let resource_imports = &imports.resources;
        let type_imports = &imports.types;

        let resource_use = if resource_imports.is_empty() {
            quote! {}
        } else {
            quote! {
                use haste_fhir_model::r4::generated::resources::{
                    #(#resource_imports),*
                };
            }
        };

        let type_use = if type_imports.is_empty() {
            quote! {}
        } else {
            quote! {
                use haste_fhir_model::r4::generated::types::{
                    #(#type_imports),*
                };
            }
        };

        generated.extend(quote! {
            #operation_doc_attributes
            pub mod #name {
                #resource_use
                #type_use

                use haste_fhir_operation_error::OperationOutcomeError;
                use haste_fhir_ops::derive::{FromParameters, ToParameters};

                pub const CODE: &str = #op_code;

                #(#generated_input)*
                #(#generated_output)*
            }
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
        #![doc = " @generated by `bash scripts/operation_build.sh` - do not edit."]
    };

    for dir_path in file_paths {
        let walker = WalkDir::new(dir_path).sort_by_file_name().into_iter();

        for entry in walker
            .filter_map(std::result::Result::ok)
            .filter(|entry| entry.metadata().is_ok_and(|metadata| metadata.is_file()))
            .filter(|entry| {
                entry
                    .path()
                    .extension()
                    .is_some_and(|extension| extension == "json")
            })
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
