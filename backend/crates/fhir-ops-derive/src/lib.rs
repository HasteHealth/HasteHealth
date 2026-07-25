use haste_fhir_model::r4::generated::resources::ResourceType;
use proc_macro2::{Ident, TokenStream};
use quote::{ToTokens, format_ident, quote};
use syn::{
    Attribute, Data, DataStruct, DeriveInput, Expr, Field, Fields, Lit, Meta, PathArguments,
    PathSegment, Type, parse_macro_input,
};

fn get_attribute_value(attrs: &[Attribute], attribute: &str) -> Option<String> {
    attrs.iter().find_map(|attr| match &attr.meta {
        Meta::NameValue(name_value) => {
            if name_value.path.is_ident(attribute) {
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
    })
}

fn get_parameter_name(field: &Field) -> String {
    if let Some(rename) = get_attribute_value(&field.attrs, "parameter_rename") {
        rename
    } else {
        field.ident.as_ref().unwrap().to_string()
    }
}

fn is_nested_parameter(attrs: &[Attribute]) -> bool {
    attrs
        .iter()
        .any(|attr| attr.path().is_ident("parameter_nested"))
}

fn determine_is_vector(field: &Field) -> bool {
    let inner_type = get_optional_type(field);

    inner_type.ident == format_ident!("Vec")
}

fn strip_type_wrappers(segment: &PathSegment, ignore_cases: Vec<String>) -> PathSegment {
    if ignore_cases.contains(&segment.ident.to_string()) {
        match &segment.arguments {
            PathArguments::AngleBracketed(args) => {
                if let Some(syn::GenericArgument::Type(Type::Path(inner_path))) = args.args.first()
                {
                    let k = inner_path.path.segments.first().unwrap();
                    strip_type_wrappers(k, ignore_cases)
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
fn field_inner_type(field: &Field) -> PathSegment {
    match &field.ty {
        Type::Path(path) => {
            let type_ = path.path.segments.first().unwrap();
            strip_type_wrappers(type_, vec!["Option".to_string(), "Vec".to_string()])
        }
        _ => panic!("Unsupported field type for serialization"),
    }
}

/// Returns the inner type if it's between Option
fn get_optional_type(field: &Field) -> PathSegment {
    match &field.ty {
        Type::Path(path) => {
            let type_ = path.path.segments.first().unwrap();
            strip_type_wrappers(type_, vec!["Option".to_string()])
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
    let field_type = field_inner_type(field).ident;

    ResourceType::try_from(field_type.to_string()).is_ok()
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
                        haste_fhir_model::r4::generated::terminology::IssueType::invalid(), format!("Field '{}' is required.", stringify!(#field_name))))?
             }
        }

    });

    quote! {
        Ok(Self {
            #(#field_setters),*
        })
    }
}

#[proc_macro_derive(ToParameters, attributes(parameter_rename, parameter_nested))]
pub fn haste_to_parameters(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match input.data {
        Data::Struct(data) => derive_to_parameters(&input.ident, &data).into(),
        _ => syn::Error::new_spanned(input.ident, "ToParameters can only be derived for structs")
            .to_compile_error()
            .into(),
    }
}

fn derive_to_parameters(struct_name: &Ident, data: &DataStruct) -> TokenStream {
    let parameters_name = format_ident!("parameters");
    let var_name = format_ident!("s");

    let to_parameters = match data
        .fields
        .iter()
        .map(generate_parameter_code)
        .collect::<syn::Result<Vec<_>>>()
    {
        Ok(tokens) => tokens,
        Err(err) => return err.to_compile_error(),
    };

    quote! {
        impl From<#struct_name> for Vec<ParametersParameter> {
            fn from(#var_name: #struct_name) -> Self {
                let mut #parameters_name = vec![];
                #(#to_parameters)*
                #parameters_name
            }
        }
    }
}

fn generate_parameter_code(field: &Field) -> syn::Result<proc_macro2::TokenStream> {
    let field_name = field.ident.as_ref().ok_or_else(|| {
        syn::Error::new_spanned(field, "ToParameters only supports named struct fields")
    })?;

    let tmp_name = format_ident!("tmp");
    let var_name = format_ident!("s");

    let mut body = generate_parameter_push(field, &tmp_name);

    if determine_is_vector(field) {
        body = quote! {
            for #tmp_name in #tmp_name.into_iter() {
                #body
            }
        };
    }

    let tokens = if is_optional(field) {
        quote! {
            if let Some(#tmp_name) = #var_name.#field_name {
                #body
            }
        }
    } else {
        quote! {
            let #tmp_name = #var_name.#field_name;
            #body
        }
    };

    Ok(tokens)
}

fn generate_parameter_push(field: &Field, value: &Ident) -> proc_macro2::TokenStream {
    let value_type = field_inner_type(field);
    let parameter_name = get_parameter_name(field);

    if is_nested_parameter(&field.attrs) {
        quote! {
            parameters.push(ParametersParameter {
                name: Box::new(FHIRString {
                    value: Some(#parameter_name.to_string()),
                    ..Default::default()
                }),
                part: Some(#value.into()),
                ..Default::default()
            });
        }
    } else if value_type.ident == format_ident!("Resource") {
        quote! {
            parameters.push(ParametersParameter {
                name: Box::new(FHIRString {
                    value: Some(#parameter_name.to_string()),
                    ..Default::default()
                }),
                resource: Some(Box::new(#value)),
                ..Default::default()
            });
        }
    } else if value_type.ident == format_ident!("ParametersParameterValueTypeChoice") {
        quote! {
            parameters.push(ParametersParameter {
                name: Box::new(FHIRString {
                    value: Some(#parameter_name.to_string()),
                    ..Default::default()
                }),
                value: Some(#value),
                ..Default::default()
            });
        }
    } else if is_resource_type(field) {
        quote! {
            parameters.push(ParametersParameter {
                name: Box::new(FHIRString {
                    value: Some(#parameter_name.to_string()),
                    ..Default::default()
                }),
                resource: Some(Box::new(Resource::#value_type(#value))),
                ..Default::default()
            });
        }
    } else {
        let primitive = value_type.ident.to_string().replacen("FHIR", "", 1);
        let parameter_value_type = format_ident!("{}", primitive);

        quote! {
            parameters.push(ParametersParameter {
                name: Box::new(FHIRString {
                    value: Some(#parameter_name.to_string()),
                    ..Default::default()
                }),
                value: Some(
                    haste_fhir_model::r4::generated::resources::ParametersParameterValueTypeChoice::#parameter_value_type(
                        Box::new(#value)
                    )
                ),
                ..Default::default()
            });
        }
    }
}

#[proc_macro_derive(FromParameters, attributes(parameter_rename, parameter_nested))]
pub fn haste_from_parameters(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match input.data {
        Data::Struct(data) => derive_from_parameters(&input.ident, &data).into(),
        _ => syn::Error::new_spanned(
            input.ident,
            "FromParameters can only be derived for structs",
        )
        .to_compile_error()
        .into(),
    }
}

fn derive_from_parameters(struct_name: &Ident, data: &DataStruct) -> proc_macro2::TokenStream {
    let parameters_name = format_ident!("parameters");
    let current_parameter = format_ident!("param");

    let declare_fields = match data
        .fields
        .iter()
        .map(generate_field_declaration)
        .collect::<syn::Result<Vec<_>>>()
    {
        Ok(fields) => fields,
        Err(err) => return err.to_compile_error(),
    };

    let set_fields = match data
        .fields
        .iter()
        .map(generate_parameter_match_arm)
        .collect::<syn::Result<Vec<_>>>()
    {
        Ok(fields) => fields,
        Err(err) => return err.to_compile_error(),
    };

    let return_value = build_return_value(&data.fields);

    quote! {
        impl TryFrom<Vec<ParametersParameter>> for #struct_name {
            type Error = OperationOutcomeError;

            fn try_from(#parameters_name: Vec<ParametersParameter>) -> Result<Self, Self::Error> {
                #(#declare_fields)*

                for #current_parameter in #parameters_name {
                    match #current_parameter.name.value.as_ref().map(|v| v.as_str()) {
                        #(#set_fields),*

                        Some(k) => {
                            return Err(OperationOutcomeError::error(
                                haste_fhir_model::r4::generated::terminology::IssueType::invalid(),
                                format!("Parameter '{}' is not allowed.", k),
                            ));
                        }

                        None => {
                            return Err(OperationOutcomeError::error(
                                haste_fhir_model::r4::generated::terminology::IssueType::invalid(),
                                "Parameter must have a name on it".to_string(),
                            ));
                        }
                    }
                }

                #return_value
            }
        }
    }
}

fn generate_field_declaration(field: &Field) -> syn::Result<proc_macro2::TokenStream> {
    let field_name = field.ident.as_ref().ok_or_else(|| {
        syn::Error::new_spanned(field, "FromParameters only supports named struct fields")
    })?;

    let field_type = get_optional_type(field).to_token_stream();

    Ok(quote! {
        let mut #field_name: Option<#field_type> = None;
    })
}

fn generate_parameter_match_arm(field: &Field) -> syn::Result<proc_macro2::TokenStream> {
    let field_name = field.ident.as_ref().ok_or_else(|| {
        syn::Error::new_spanned(field, "FromParameters only supports named struct fields")
    })?;
    let is_vector = determine_is_vector(field);
    let expected_parameter_name = get_parameter_name(field);

    let value_from_param = generate_value_extraction(field);

    let setter = if is_vector {
        quote! {
            let tmp_value: Result<_, OperationOutcomeError> = #value_from_param;

            if let Some(tmp_value) = tmp_value? {
                if let Some(tmp_array) = #field_name.as_mut() {
                    tmp_array.push(tmp_value);
                } else {
                    #field_name = Some(vec![tmp_value]);
                }
            }
        }
    } else {
        quote! {
            if #field_name.is_some() {
                return Err(OperationOutcomeError::error(
                    haste_fhir_model::r4::generated::terminology::IssueType::invalid(),
                    format!("Parameter '{}' is not allowed to be repeated.", #expected_parameter_name)
                ));
            }

            let tmp_value: Result<_, OperationOutcomeError> = #value_from_param;
            #field_name = tmp_value?;
        }
    };

    Ok(quote! {
        Some(#expected_parameter_name) => {
            #setter
        }
    })
}

fn generate_value_extraction(field: &Field) -> proc_macro2::TokenStream {
    let value_type = field_inner_type(field);
    let expected_parameter_name = get_parameter_name(field);
    let current_parameter = format_ident!("param");

    if is_nested_parameter(&field.attrs) {
        quote! {
            #value_type::try_from(
                #current_parameter.part.unwrap_or_default()
            )
            .map(Some)
        }
    } else if value_type.ident == format_ident!("Resource") {
        quote! {
            Ok(#current_parameter.resource.map(|r| *r))
        }
    } else if value_type.ident == format_ident!("ParametersParameterValueTypeChoice") {
        quote! {
            Ok(#current_parameter.value)
        }
    } else if is_resource_type(field) {
        quote! {
            if let Some(Resource::#value_type(resource)) =
                #current_parameter.resource.map(|r| *r)
            {
                Ok(Some(resource))
            } else {
                Err(OperationOutcomeError::error(
                    haste_fhir_model::r4::generated::terminology::IssueType::invalid(),
                    format!(
                        "Parameter '{}' does not contain correct value type.",
                        #expected_parameter_name
                    )
                ))
            }
        }
    } else {
        let primitive = value_type.ident.to_string().replacen("FHIR", "", 1);
        let parameter_value_type = format_ident!("{}", primitive);

        quote! {
            if let Some(
                haste_fhir_model::r4::generated::resources::ParametersParameterValueTypeChoice::#parameter_value_type(value)
            ) = #current_parameter.value {
                Ok(Some(*value))
            } else {
                Err(OperationOutcomeError::error(
                    haste_fhir_model::r4::generated::terminology::IssueType::invalid(),
                    format!(
                        "Parameter '{}' does not contain correct value type.",
                        #expected_parameter_name
                    )
                ))
            }
        }
    }
}
