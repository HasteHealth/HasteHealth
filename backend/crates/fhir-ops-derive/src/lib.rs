use oxidized_fhir_model::r4::types::ResourceType;
use proc_macro::TokenStream;
use quote::{ToTokens, format_ident, quote};
use syn::{Attribute, Data, DeriveInput, Expr, Field, Lit, Meta, Type, parse_macro_input};

enum Direction {
    Input,
    Output,
}

fn get_direction(attrs: &[Attribute]) -> Direction {
    let direction = attrs.iter().find_map(|attr| match &attr.meta {
        Meta::NameValue(name_value) => {
            if name_value.path.is_ident("direction") {
                match &name_value.value {
                    Expr::Lit(lit) => match &lit.lit {
                        Lit::Str(lit) => Some(lit.value()),
                        _ => panic!("Expected a string literal"),
                    },
                    _ => panic!("Expected a string literal"),
                }
            } else {
                None
            }
        }
        _ => None,
    });

    match direction.as_ref().map(|s| s.as_str()) {
        Some("in") => Direction::Input,
        Some("out") => Direction::Output,
        _ => panic!("Direction attribute is required and must be either 'in' or 'out'."),
    }
}

fn get_field_type(field: &Field) -> proc_macro2::Ident {
    match &field.ty {
        Type::Path(path) => path.path.segments.first().unwrap().ident.clone(),
        _ => panic!("Unsupported field type for serialization"),
    }
}

fn is_optional<'a>(field: &'a Field) -> bool {
    let field_type = get_field_type(field);
    if field_type == "Option" { true } else { false }
}

fn is_vec<'a>(field: &'a Field) -> bool {
    match &field.ty {
        Type::Path(path) => {
            for segment in path.path.segments.iter() {
                if segment.ident == format_ident!("Vec") {
                    return true;
                }
            }
            false
        }
        _ => panic!("Unsupported field type for serialization"),
    }
}

/// Returns the inner type if it's between Options and Vecs etc..
fn inner_type(field: &Field) -> Option<proc_macro2::Ident> {
    match &field.ty {
        Type::Path(path) => {
            for segment in path.path.segments.iter() {
                if segment.ident != format_ident!("Option") && segment.ident != format_ident!("Vec")
                {
                    return Some(segment.ident.clone());
                }
            }
            None
        }
        _ => panic!("Unsupported field type for serialization"),
    }
}

/// Returns the inner type if it's between Option
fn get_optional_type(field: &Field) -> Option<proc_macro2::Ident> {
    match &field.ty {
        Type::Path(path) => {
            for segment in path.path.segments.iter() {
                if segment.ident != format_ident!("Option") {
                    return Some(segment.ident.clone());
                }
            }
            None
        }
        _ => panic!("Unsupported field type for serialization"),
    }
}

fn is_resource_type(field: &Field) -> bool {
    let Some(field_type) = inner_type(field) else {
        return false;
    };

    let res = ResourceType::try_from(field_type.to_string());
    if let Ok(_) = res {
        return true;
    } else {
        field_type == format_ident!("Resource")
    }
}

#[proc_macro_derive(ParametersParse, attributes(rename_field))]
pub fn oxidized_from_parameter(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    let direction = get_direction(&input.attrs);

    match input.data {
        Data::Struct(data) => {
            let struct_name = input.ident;
            let parameters_name = format_ident!("parameters");
            let current_parameter = format_ident!("param");

            // Declare all the fields on the struct.
            let declare_fields = data.fields.iter().map(|field| {
                let field_name = field.ident.as_ref().unwrap();
                let field_type_token = get_optional_type(field);

                quote! {
                    let mut #field_name: Option<#field_type_token> = None;
                }
            });

            let set_fields = data.fields.iter().map(|field| {
                let field_name = field.ident.as_ref().unwrap();
                let is_vector = is_vec(field);
                let value_type = inner_type(field);
                let expected_parameter_name = field_name.to_string();

                let get_value_from_param = if value_type == Some(format_ident!("Resource")) {
                    quote!{
                        Ok(param.resource)
                    }
                } else if is_resource_type(field) {
                    quote! {
                        if let Some(Resource::#value_type(resource)) = param.resource {
                            Ok(Some(resource))
                        } else {
                            return Err(OperatinOutcomeError(OperationOutcomeCodes::Invalid, format!("Parameter '{}' does not contain correct value type.", #expected_parameter_name)));
                        }
                    }
                } else {
                    quote! {
                        if let Some(oxidized_fhir_model::r4::types::ParametersParameterValueTypeChoice::#value_type(value)) = param.value {
                            Ok(Some(value))
                        } else {
                            return Err(OperatinOutcomeError(OperationOutcomeCodes::Invalid, format!("Parameter '{}' does not contain correct value type.", #expected_parameter_name)));
                        }
                    }
                };

                let setter = if is_vector {
                    quote! {
                        let tmp_value = #get_value_from_param;
                        let mut tmp_value_array = #field_name.unwrap_or_default();
                        tmp_value_array.push(tmp_value?);
                        #field_name = Some(tmp_value_array);

                    }
                } else {
                    quote! {
                        if #field_name.is_some(){
                            return Err(OperatinOutcomeError(OperationOutcomeCodes::Invalid, format!("Parameter '{}' is not allowed to be repeated.", #expected_parameter_name)));
                        }
                        let tmp_value = #get_value_from_param;
                        #field_name = Some(tmp_value);
                    }
                };

                quote!{
                    if #current_parameter.name.value.as_ref().map(|v| v.as_str()) == Some(#expected_parameter_name) {
                        #setter
                    }
                }
            });

            quote! {
                impl TryFrom<Vec<ParametersParameter>, Error = OperationOutcomeError> for #struct_name {
                    fn try_from(#parameters_name: Vec<ParametersParameter>) -> Result<Self, Self::Error> {
                        #(#declare_fields)*

                        for #current_parameter in #parameters_name {
                            #(#set_fields)*
                        }

                        todo!("Not implemented.");
                    }
                }
            }.into()
        }
        _ => panic!("From parameter deriviation is only supported for structs."),
    }
}
