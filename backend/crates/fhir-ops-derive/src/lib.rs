use oxidized_fhir_model::r4::types::ResourceType;
use proc_macro::TokenStream;
use quote::{ToTokens, format_ident, quote};
use syn::{
    parse_macro_input, Attribute, Data, DeriveInput, Field, Fields, PathArguments, PathSegment, Type
};

fn is_nested_parameter(attrs: &[Attribute]) -> bool {
    attrs
        .iter()
        .any(|attr| attr.path().is_ident("parameter_nested"))
}

fn determine_is_vector(field: &Field) -> bool {
    let inner_type = get_optional_type(field);

    inner_type.ident == format_ident!("Vec")
}

fn _inner_type(segment: &PathSegment, ignore_cases: Vec<String>) -> PathSegment {
    if ignore_cases.contains(&segment.ident.to_string()) {
        match &segment.arguments {
            PathArguments::AngleBracketed(args) => {
                if let Some(syn::GenericArgument::Type(Type::Path(inner_path))) = args.args.first()
                {
                    let k = inner_path.path.segments.first().unwrap();
                    _inner_type(k, ignore_cases)
                } else {
                    panic!("invalid");
                }
            }
            _ => panic!("invalid"),
        }
    } else {
        segment.clone()
    }
}

/// Returns the inner type if it's between Options and Vecs etc..
fn inner_type(field: &Field) -> PathSegment {
    match &field.ty {
        Type::Path(path) => {
            let type_ = path.path.segments.first().unwrap();
            _inner_type(type_, vec!["Option".to_string(), "Vec".to_string()])
        }
        _ => panic!("Unsupported field type for serialization"),
    }
}

/// Returns the inner type if it's between Option
fn get_optional_type(field: &Field) -> PathSegment {
    match &field.ty {
        Type::Path(path) => {
            let type_ = path.path.segments.first().unwrap();
            _inner_type(type_, vec!["Option".to_string()])
        }
        _ => panic!("Unsupported field type for serialization"),
    }
}

fn is_optional(field: &Field) -> bool {
    match &field.ty {
        Type::Path(path) => {
            let type_ = path.path.segments.first().unwrap();
            type_.ident == format_ident!("Option")
        }
        _ => panic!("Unsupported field type for serialization"),
    }
}


fn is_resource_type(field: &Field) -> bool {
    let field_type = inner_type(field).ident;

    let res = ResourceType::try_from(field_type.to_string());
    if let Ok(_) = res {
        return true;
    } else {
        return false;
    }
}

fn build_return_value(fields: &Fields) -> proc_macro2::TokenStream {
    
    let field_setters = fields.iter().map(|field| {
        let optional = is_optional(field);
        let field = field.ident.as_ref().unwrap();
        let field_name = field.to_string();

        if optional {
            quote!{ 
                #field: #field
            }
        } else {
            quote!{
                #field: #field.ok_or_else(|| 
                    OperationOutcomeError::error(
                        OperationOutcomeCodes::Invalid, format!("Field '{}' is required.", stringify!(#field_name))))?
             }
        }
        
    });

    quote! {
        Ok(Self {
            #(#field_setters),*
        })
    }
}

#[proc_macro_derive(ParametersParse, attributes(direction, parameter_nested))]
pub fn oxidized_from_parameter(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    match input.data {
        Data::Struct(data) => {
            let struct_name = input.ident;
            let parameters_name = format_ident!("parameters");
            let current_parameter = format_ident!("param");

            // Declare all the fields on the struct.
            let declare_fields = data.fields.iter().map(|field| {
                let field_name = field.ident.as_ref().unwrap();
                let field_type_token = get_optional_type(field).to_token_stream();

                quote! {
                    let mut #field_name: Option<#field_type_token> = None;
                }
            });

            let set_fields = data.fields.iter().map(|field| {
                let field_name = field.ident.as_ref().unwrap();
                let is_vector = determine_is_vector(field);
                let value_type = inner_type(field);
                let expected_parameter_name = field_name.to_string();

                let get_value_from_param = if is_nested_parameter(&field.attrs) {
                    quote!{ 
                        #value_type::try_from(#current_parameter.part.unwrap_or_default()).map(|v| Some(v))
                    }
                }else if value_type.ident == format_ident!("Resource") {
                    quote!{
                        Ok(#current_parameter.resource.map(|r| *r))
                    }
                }else  if value_type.ident == format_ident!("ParametersParameterValueTypeChoice") {
                    quote! {
                        Ok(#current_parameter.value)
                    }
                }
                 else if is_resource_type(field) {
                    quote! {
                        if let Some(Resource::#value_type(resource)) = #current_parameter.resource.map(|r| *r) {
                            Ok(Some(resource))
                        } else {
                            return Err(OperationOutcomeError::error(OperationOutcomeCodes::Invalid, format!("Parameter '{}' does not contain correct value type.", #expected_parameter_name)));
                        }
                    }
                } else {
                    // Need to remove the start FHIR on the primitives.
                    let removed_fhir = value_type.ident.to_string().replacen("FHIR", "", 1);
                    let parameter_value_type = format_ident!("{}", removed_fhir);

                    quote! {
                        if let Some(oxidized_fhir_model::r4::types::ParametersParameterValueTypeChoice::#parameter_value_type(value)) = #current_parameter.value {
                            Ok(Some(*value))
                        } else {
                            return Err(OperationOutcomeError::error(OperationOutcomeCodes::Invalid, format!("Parameter '{}' does not contain correct value type.", #expected_parameter_name)));
                        }
                    }
                };

                let setter = if is_vector {
                    quote! {
                        let tmp_value = #get_value_from_param;
                        if let Some(tmp_value) = tmp_value? {
                            if let Some(tmp_array) = #field_name.as_mut(){
                                tmp_array.push(tmp_value);
                            } else {
                                #field_name = Some(vec![tmp_value]);
                            }
                        }
                    }
                } else {
                    quote! {
                        if #field_name.is_some(){
                            return Err(OperationOutcomeError::error(OperationOutcomeCodes::Invalid, format!("Parameter '{}' is not allowed to be repeated.", #expected_parameter_name)));
                        }
                        let tmp_value = #get_value_from_param;
                        #field_name = tmp_value?;
                    }
                };

                quote!{
                    Some(#expected_parameter_name) =>  {
                        #setter
                    }
                }
            });

            let return_value = build_return_value(&data.fields);

            let try_from_code = quote! {
                impl TryFrom<Vec<ParametersParameter>> for #struct_name {
                    type Error = OperationOutcomeError;
                    fn try_from(#parameters_name: Vec<ParametersParameter>) -> Result<Self, Self::Error> {
                        #(#declare_fields)*

                        for #current_parameter in #parameters_name {
                            match #current_parameter.name.value.as_ref().map(|v| v.as_str()) {
                                #(#set_fields),*
                                Some(k) => {
                                    return Err(OperationOutcomeError::error(OperationOutcomeCodes::Invalid, format!("Parameter '{}' is not allowed.", k)));
                                },
                                None => {
                                    return Err(OperationOutcomeError::error(OperationOutcomeCodes::Invalid, format!("Parameter must have a name on it")));
                                }
                            }
                        }

                        #return_value
                    }
                }
            };

            // println!("{}", try_from_code.to_string());

            try_from_code.into()
        }
        _ => panic!("From parameter deriviation is only supported for structs."),
    }
}
