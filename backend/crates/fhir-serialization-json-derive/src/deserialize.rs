use crate::{utilities::{ get_attribute_value, get_type_choice_attribute, is_attribute_present}, DeserializeComplexType};
use core::panic;

use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};
use syn::{Data, DeriveInput, Field, Fields, Ident, Type};

fn is_type_choice_field(field: &Field) -> bool {
    is_attribute_present(&field.attrs, "type_choice_variants")
}

fn get_field_type(field: &Field) -> proc_macro2::Ident {
    match &field.ty {
        Type::Path(path) => path.path.segments.first().unwrap().ident.clone(),
        _ => panic!("Unsupported field type for serialization"),
    }
}

fn is_optional_field(field: &Field) -> bool {
    let field_type = get_field_type(field);
    if field_type == "Option" { true } else { false }
}

fn handle_optional_require_field(
    field: &Field,
    value_identifier: proc_macro2::Ident,
) -> TokenStream {
    let value_string_name = value_identifier.to_string();
    if is_optional_field(field) {
        // Safe to unwrap as nested option.
        quote! {
            #value_identifier.and_then(|v| v)
        }
    } else {
        // If not optional that means it's a required field so we should unwrap it here.
        quote! {
            #value_identifier.ok_or_else(|| {
                oxidized_fhir_serialization_json::errors::DeserializeError::MissingRequiredField(
                    #value_string_name.to_string(),
                )
            })?
        }
    }
}

pub fn primitive_deserialization(input: DeriveInput) -> TokenStream {
    let name = input.ident;

    match input.data {
        Data::Struct(data) => {
            let value_field_found = data
                .fields
                .iter()
                .find(|f| f.ident == Some(format_ident!("value")));

            let value_type = get_field_type(value_field_found.unwrap());

            let unwrap_required =
                handle_optional_require_field(value_field_found.unwrap(), format_ident!("value"));

            let expanded = quote! {
                impl FHIRJSONDeserializer for #name {
                    fn from_json_str(s: &str) -> Result<Self, oxidized_fhir_serialization_json::errors::DeserializeError> {
                        let json = serde_json::from_str(s)?;
                        Self::from_serde_value(&json, oxidized_fhir_serialization_json::Context::AsValue)
                    }

                    fn from_serde_value(json: &serde_json::Value, context: oxidized_fhir_serialization_json::Context) -> Result<Self, oxidized_fhir_serialization_json::errors::DeserializeError> {
                        match context {
                            oxidized_fhir_serialization_json::Context::AsField(context) => {
                                let mut value = None;
                                let mut extensions = None;
                                let mut id = None;

                                if let Some(json_value) = json.get(context.field){
                                    value = Some(#value_type::from_serde_value(json_value, oxidized_fhir_serialization_json::Context::AsValue)?);
                                }

                                if let Some(json_element_fields) = json.get(&("_".to_string() + context.field)) {
                                    if !json_element_fields.is_object() {
                                        return Err(oxidized_fhir_serialization_json::errors::DeserializeError::InvalidType(
                                            "Expected an object for element fields".to_string(),
                                        ));
                                    }
                                    extensions = Option::from_serde_value(json_element_fields, ("extension", false).into())?;
                                    id = Option::from_serde_value(json_element_fields, ("id", false).into())?;
                                }

                                Ok(Self {
                                    value: #unwrap_required,
                                    extension: extensions,
                                    id: id,
                                })
                            }
                            oxidized_fhir_serialization_json::Context::AsValue => {
                                let value = #value_type::from_serde_value(json, oxidized_fhir_serialization_json::Context::AsValue)?;
                                let mut parsed = Self::default();
                                parsed.value = value;
                                Ok(parsed)
                            }
                        }
                    }
                }
            };

            // println!("{}", expanded.to_string());

            expanded.into()
        }
        _ => panic!("Only structs can be serialized for primitive serializer."),
    }
}

pub fn deserialize_valueset(input: DeriveInput) -> TokenStream {
   let name = input.ident;
   match input.data {
        Data::Enum(data) => {
            let variants_deserialize_value = data.variants.iter().filter_map(|variant| {
                let variant_name = variant.ident.to_owned();
                let code = get_attribute_value(&variant.attrs, "code");
                if let Some(code) = code {
                    Some(quote! {
                        #code =>  Ok(#name::#variant_name(None))
                    })
                } else {
                    None
                }
            });

            let variants_deserialize_value_with_element = data.variants.iter().filter_map(|variant| {
                let variant_name = variant.ident.to_owned();
                let code = get_attribute_value(&variant.attrs, "code");
                if let Some(code) = code {
                    Some(quote! {
                        #code =>  Ok(#name::#variant_name(element))
                    })
                } else {
                    None
                }
            });

            let expanded: TokenStream = quote! {
                impl oxidized_fhir_serialization_json::FHIRJSONDeserializer for #name {
                    fn from_json_str(s: &str) -> Result<Self, oxidized_fhir_serialization_json::errors::DeserializeError> {
                        let json = serde_json::from_str(s)?;
                        Self::from_serde_value(&json, oxidized_fhir_serialization_json::Context::AsValue)
                    }

                    fn from_serde_value(json: &serde_json::Value, context: oxidized_fhir_serialization_json::Context) -> Result<Self, oxidized_fhir_serialization_json::errors::DeserializeError> {
                        match context {
                            oxidized_fhir_serialization_json::Context::AsField(context) => {
                                let mut element = None;

                                if let Some(json_element_fields) = json.get(&("_".to_string() + context.field)) {
                                    if !json_element_fields.is_object() {
                                        return Err(oxidized_fhir_serialization_json::errors::DeserializeError::InvalidType(
                                            "Expected an object for element fields".to_string(),
                                        ));
                                    }
                                    element = Some(Element::from_serde_value(json_element_fields, oxidized_fhir_serialization_json::Context::AsValue)?);
                                }
                                match json.get(context.field) {
                                    Some(serde_json::Value::String(s)) => {
                                        match s.as_str(){
                                            #(#variants_deserialize_value_with_element),*,
                                            _ => Err(oxidized_fhir_serialization_json::errors::DeserializeError::InvalidType(
                                                "Expected a string for value set enum".to_string(),
                                            )),
                                        }
                                    },
                                    None => {
                                        Ok(Self::Null(element))
                                    },
                                    _ => return Err(oxidized_fhir_serialization_json::errors::DeserializeError::InvalidType(
                                        "Expected a string for value set enum".to_string(),
                                    )),
                                }
                            }
                            oxidized_fhir_serialization_json::Context::AsValue => {
                                match json {
                                    serde_json::Value::String(s) => {
                                        match s.as_str() {
                                            #(#variants_deserialize_value),*,
                                            _ => Err(oxidized_fhir_serialization_json::errors::DeserializeError::InvalidType(
                                                "Expected a string for value set enum".to_string(),
                                            )),
                                        }
                                    },
                                    _ => return Err(oxidized_fhir_serialization_json::errors::DeserializeError::InvalidType(
                                        "Expected a string for value set enum".to_string(),
                                    )),
                                }
                            }
                        }
                    }
                }
            };

            //println!("{}", expanded.to_string());

            expanded.into()
        }
         _ => panic!("Value set serialization only works for enums"),
    }

}

pub fn deserialize_typechoice(input: DeriveInput) -> TokenStream {
    let name = input.ident;

    let typechoice_name = get_attribute_value(&input.attrs, "type_choice_field_name").unwrap();

    match input.data {
        Data::Enum(data) => {

            let serialize_by_name_matches = data.variants.iter().map(|variant| {
                let name = variant.ident.to_owned();
                let field_name = format!("{}{}", typechoice_name, name);
                let field: &Field = variant.fields.iter().next().unwrap();
                
                let variant_type = get_field_type(field);

                quote! {
                    #field_name => {
                        Ok(Self::#name(#variant_type::from_serde_value(json, oxidized_fhir_serialization_json::Context::AsField(context))?))
                    }
                }
            });

            let expanded = quote! {
                impl oxidized_fhir_serialization_json::FHIRJSONDeserializer for #name {
                    fn from_json_str(s: &str) -> Result<Self, oxidized_fhir_serialization_json::errors::DeserializeError> {
                        Err(oxidized_fhir_serialization_json::errors::DeserializeError::CannotDeserializeTypeChoiceAsValue)
                    }

                    fn from_serde_value(json: &serde_json::Value, context: oxidized_fhir_serialization_json::Context) -> Result<Self, oxidized_fhir_serialization_json::errors::DeserializeError> {
                        match context {
                            oxidized_fhir_serialization_json::Context::AsField(context) => {
                                // Handle deserialization for each variant
                                match context.field {
                                    #(#serialize_by_name_matches),*,
                                    _ => Err(oxidized_fhir_serialization_json::errors::DeserializeError::InvalidTypeChoiceVariant(context.field.to_string())),
                                }
                            }
                            oxidized_fhir_serialization_json::Context::AsValue => {
                                Err(oxidized_fhir_serialization_json::errors::DeserializeError::CannotDeserializeTypeChoiceAsValue)
                            }
                        }
                    }
                }
            };

            expanded.into()
        }
        _ => panic!("Only enums can be deserialized for typechoice serializer."),
    }
}

/// Use rename_field attribute if present else use the struct name
fn get_field_name(field: &Field) -> String {
    get_attribute_value(&field.attrs, "rename_field")
        .unwrap_or_else(|| field.ident.as_ref().unwrap().to_string())
}

fn create_primitive_struct_handler(
    found_fields_variable: &Ident,
    obj_variable: &Ident,
    field_variable: &Ident,
    field: &Field,
) -> TokenStream {
    let field_ident = field.ident.as_ref().unwrap();
    let field_str = get_field_name(field);
    let extension_str = format!("_{}", field_str);
    let field_type = get_field_type(field);

    quote! {
        if #field_variable == #field_str || #field_variable == #extension_str {
           #found_fields_variable.insert(#field_str);
           #found_fields_variable.insert(#extension_str);
           #field_ident = Some(#field_type::from_serde_value(#obj_variable, (#field_str, true).into())?);
        }
    }
}

fn create_type_choice_struct_handler(
    found_fields_variable: &Ident,
    obj_variable: &Ident,
    field_variable: &Ident,
    field: &Field,
) -> TokenStream {
    let field_ident = field.ident.as_ref().unwrap();
    let field_type = get_field_type(field);
    let type_choice_variants = get_type_choice_attribute(&field.attrs).unwrap();
    let all_type_choice_variants = type_choice_variants.all();
    let primitive_variants = type_choice_variants.primitive_variants;

    // For each individual primitve variant also check the extension
    let primitive_checks = primitive_variants.iter().map(|primitive_variant| {
        let extension_variant = format!("_{}", primitive_variant);
        quote!{
            if(#primitive_variant == #field_variable || #extension_variant == #field_variable) {
                #found_fields_variable.insert(#primitive_variant);
                #found_fields_variable.insert(#extension_variant);
                #field_ident = Some(#field_type::from_serde_value(#obj_variable, (#primitive_variant, true).into())?);
            }
        }
    });

    quote! {
        if [#(#all_type_choice_variants),*].contains(&#field_variable.as_str()) {
            if let Some(existing_type_choice) = #field_ident {
                return Err(oxidized_fhir_serialization_json::errors::DeserializeError::DuplicateTypeChoiceVariant(
                    #field_variable.to_string(),
                ));
            }
            #(#primitive_checks)else *
            else {
                #field_ident = Some(#field_type::from_serde_value(#obj_variable, (#field_variable, false).into())?);
            }
        }
    }
}

fn create_complex_struct_handler(
    found_fields_variable: &Ident,
    obj_variable: &Ident,
    field_variable: &Ident,
    field: &Field,
) -> TokenStream {
    let field_ident = field.ident.as_ref().unwrap();
    let field_type = get_field_type(field);
    let field_str = get_field_name(field);

    quote! {
        if #field_variable == #field_str {
          #found_fields_variable.insert(#field_str);
          #field_ident = Some(#field_type::from_serde_value(#obj_variable.get(#field_str).unwrap(), oxidized_fhir_serialization_json::Context::AsValue)?);
        }
    }
}

fn set_struct_field(found_fields_variable: &Ident, obj_variable: &Ident, field_variable: &Ident, field: &Field) -> TokenStream {
    if is_attribute_present(&field.attrs, "primitive") {
        create_primitive_struct_handler(found_fields_variable, obj_variable, field_variable, field)
    } else if is_type_choice_field(field) {
        create_type_choice_struct_handler(found_fields_variable, obj_variable, field_variable, field)
    } else {
        create_complex_struct_handler(found_fields_variable, obj_variable, field_variable, field)
    }
}

fn create_struct_item(fields: &Fields) -> TokenStream {
    let mut required_fields: Vec<&Field> = vec![];
    let mut optional_fields: Vec<&Field> = vec![];

    for field in fields {
        if is_optional_field(field) {
            optional_fields.push(field);
        } else {
            required_fields.push(field);
        }
    }

    let optional_fields_instantiate = optional_fields.iter().map(|field| {
        let field_name = field.ident.as_ref().unwrap();
        // We can just unwrap with an and_then to remove the nested option.
        quote! {
            #field_name: #field_name.and_then(|v| v),
        }
    });

    let required_fields_instantiate = required_fields.iter().map(|field| {
        let field_name = field.ident.as_ref().unwrap();
        let field_name_str = field_name.to_string();

        quote! {
          #field_name: #field_name.ok_or_else(|| {
            oxidized_fhir_serialization_json::errors::DeserializeError::MissingRequiredField(
                #field_name_str.to_string()
            )})?,
        }
    });
    quote! {
        Self{
            #(#required_fields_instantiate)*
            #(#optional_fields_instantiate)*
        }
    }
}

pub fn deserialize_complex(input: DeriveInput, deserialize_complex_type: DeserializeComplexType) -> TokenStream {
    let name = input.ident;
    let name_string = name.to_string();

    match input.data {
        Data::Struct(data) => {
            // Declare all the fields on the struct.
            let declare_fields = data.fields.iter().map(|field| {
                let field_name = field.ident.as_ref().unwrap();
                let field_type_token = field.ty.to_token_stream();
                quote! {
                    let mut #field_name: Option<#field_type_token> = None;
                }
            });

            let field_variable = format_ident!("_field_");
            let obj_variable = format_ident!("obj");

            let found_fields_ident = format_ident!("found_fields");

            let check_resource_type = match deserialize_complex_type {
                DeserializeComplexType::Resource => {
                    quote! {
                        if let Some(resource_type_json) = #obj_variable.get("resourceType") && let Some(resource_type) = resource_type_json.as_str() {
                                if resource_type == #name_string {
                                    #found_fields_ident.insert("resourceType");
                                } else {
                                    return Err(oxidized_fhir_serialization_json::errors::DeserializeError::InvalidResourceType(
                                        #name_string.to_string(),
                                        resource_type.to_string(),
                                    ));
                                }
                        } 
                        else {
                            return Err(oxidized_fhir_serialization_json::errors::DeserializeError::MissingRequiredField("resourceType".to_string()));
                        }


                    }
                },
                DeserializeComplexType::Complex => {
                    quote!{}
                }
            };
            
            let set_value = data
                .fields
                .iter()
                .map(|field| set_struct_field(&found_fields_ident, &obj_variable, &field_variable, field));

            let return_val = create_struct_item(&data.fields);

            let expanded = quote! {
                impl oxidized_fhir_serialization_json::FHIRJSONDeserializer for #name {
                    fn from_json_str(s: &str) -> Result<Self, oxidized_fhir_serialization_json::errors::DeserializeError> {
                        let json = serde_json::from_str(s)?;
                        Self::from_serde_value(&json, oxidized_fhir_serialization_json::Context::AsValue)
                    }

                    fn from_serde_value(#obj_variable: &serde_json::Value, context: oxidized_fhir_serialization_json::Context) -> Result<Self, oxidized_fhir_serialization_json::errors::DeserializeError> {
                        let #obj_variable = {
                            match context {
                                oxidized_fhir_serialization_json::Context::AsValue => {
                                   Ok(#obj_variable)
                                }
                                oxidized_fhir_serialization_json::Context::AsField(context) => {
                                    #obj_variable.get(context.field)
                                        .ok_or_else(|| oxidized_fhir_serialization_json::errors::DeserializeError::MissingRequiredField(context.field.to_string()))
                                }
                            }
                        }?;

                        let mut #found_fields_ident = std::collections::HashSet::new();
                        if let Some(json_obj) = #obj_variable.as_object() {
                            #check_resource_type
                            #(#declare_fields)*
                            for #field_variable in json_obj.keys() {
                                if !#found_fields_ident.contains(#field_variable.as_str()){
                                  #(#set_value)else *
                                  else {
                                    return Err(oxidized_fhir_serialization_json::errors::DeserializeError::UnknownField(
                                        format!("{}: '{}'", #name_string, #field_variable.to_string())
                                    ));
                                  }
                                }
                            }
                            Ok(#return_val)
                        } else {
                            Err(oxidized_fhir_serialization_json::errors::DeserializeError::InvalidType(
                                "Expected an object".to_string(),
                            ))
                        }
                    }
                }
            };

            // println!("{}", expanded.to_string());
            expanded.into()
        }
        _ => panic!("Only enums can be deserialized for typechoice serializer."),
    }
}



pub fn enum_variant_deserialization(input: DeriveInput) -> TokenStream {
    let name = input.ident;
    let determine_by = get_attribute_value(&input.attrs, "determine_by").unwrap();

    match input.data {
        Data::Enum(data) => {
            let determine_by_value = format_ident!("_determine_by_");
            let serialize_by_name_matches = data.variants.iter().map(|variant| {
                let name = variant.ident.to_owned();
                let field_name = name.to_string();
                let field: &Field = variant.fields.iter().next().unwrap();
                let variant_type = get_field_type(field);
    

                quote! {
                    #field_name => {
                        Ok(Self::#name(#variant_type::from_serde_value(json, oxidized_fhir_serialization_json::Context::AsValue)?))
                    }
                }
            });

            let expanded = quote!{
                impl oxidized_fhir_serialization_json::FHIRJSONDeserializer for #name {
                    fn from_json_str(s: &str) -> Result<Self, oxidized_fhir_serialization_json::errors::DeserializeError> {
                        let json = serde_json::from_str(s)?;
                        Self::from_serde_value(&json, oxidized_fhir_serialization_json::Context::AsValue)
                    }

                    fn from_serde_value(json: &serde_json::Value, context: oxidized_fhir_serialization_json::Context) -> Result<Self, oxidized_fhir_serialization_json::errors::DeserializeError> {
                        let json = {
                            match &context {
                                oxidized_fhir_serialization_json::Context::AsValue => {
                                   Ok(json)
                                }
                                oxidized_fhir_serialization_json::Context::AsField(context) => {
                                    json.get(context.field)
                                        .ok_or_else(|| oxidized_fhir_serialization_json::errors::DeserializeError::MissingRequiredField(context.field.to_string()))
                                }
                            }
                        }?;

                        if let Some(json_v) = json.get(#determine_by) && let Some(#determine_by_value) = json_v.as_str()  {
                            match #determine_by_value {
                                #(#serialize_by_name_matches),*
                                field => Err(oxidized_fhir_serialization_json::errors::DeserializeError::InvalidEnumVariant(
                                    #determine_by.to_string(), field.to_string()
                                )),
                            }                            
                        } else {
                            Err(oxidized_fhir_serialization_json::errors::DeserializeError::MissingRequiredField(
                                #determine_by.to_string(),
                            ))
                        }
                    }
                }
            };

            // println!("{}", expanded.to_string());

            expanded.into()
        }
        _ => panic!("Only enums can be deserialized for enum serializer."),
    }
}
