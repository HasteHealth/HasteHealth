use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{Data, DeriveInput, Field, Ident, Type, Variant};

use crate::{
    DeserializeComplexType,
    utilities::{
        CardinalityAttribute, TypeChoiceAttribute, get_attribute_value, get_cardinality_attributes,
        get_field_name, get_field_type, get_optional_inner_type, get_type_choice_attribute,
        is_attribute_present, is_optional_field, is_vector,
    },
};

// Generates code for deserializing the primtiive value.
// Note field, extension deserialization is handled on struct level (parent).
pub fn fhir_primitive_deserialization(input: DeriveInput) -> TokenStream {
    let name = input.ident;
    match input.data {
        Data::Struct(data) => {
            let value_field_found = data
                .fields
                .iter()
                .find(|f| f.ident == Some(format_ident!("value")));

            let value_type = get_field_type(value_field_found.unwrap());

            let deserialize_impl = quote! {
               impl<'de> serde::Deserialize<'de> for #name {
                    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                    where
                        D: serde::Deserializer<'de>,
                    {
                        let s = #value_type::deserialize(deserializer)?;
                        Ok(#name {
                            id: None,
                            extension: None,
                            value: s,
                        })
                    }
                }
            };

            deserialize_impl.into()
        }
        _ => panic!("Only structs can be serialized for primitive deserializer."),
    }
}

pub fn valueset_deserialization(input: DeriveInput) -> TokenStream {
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

            let variants_merge_element = data.variants.iter().map(|variant| {
                let variant_name = variant.ident.to_owned();
                quote! {
                    Self::#variant_name(inner) => {
                        *inner = Some(element);
                    }
                }
            });

            let visitor_name = format_ident!("{}Visitor", name);
            let name_str = name.to_string();

            let deserialize_impl = quote! {
                impl<'de> serde::Deserialize<'de> for #name {
                    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
                        struct #visitor_name;
                            impl<'de> serde::de::Visitor<'de> for #visitor_name {
                                type Value = #name;
                                fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                                    write!(f, "a string code for {}", #name_str)
                                }
                                fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<#name, E> {
                                    match v {
                                        #(#variants_deserialize_value),*,
                                        other => Err(E::custom(format!("Unknown code '{}' for {}", other, #name_str))),
                                    }
                                }
                            }
                        d.deserialize_str(#visitor_name)
                    }
                }

                impl #name {
                    pub fn merge_element(&mut self, element: Element) {
                        match self {
                            #(#variants_merge_element,)*
                        }
                    }
                }
            };

            deserialize_impl.into()
        }
        _ => panic!("Only enums can be serialized for value set deserializer."),
    }
}

pub fn typechoice_deserialization(input: DeriveInput) -> TokenStream {
    let type_choice_field_name = get_attribute_value(&input.attrs, "type_choice_field_name")
        .expect("type_choice_field_name attribute is required for typechoice deserialization");
    let name = input.ident;
    match input.data {
        Data::Enum(data) => {
            let (primitive_variants, complex_variants): (Vec<Variant>, Vec<Variant>) = data
                .variants
                .into_iter()
                .partition(|variant| is_attribute_present(&variant.attrs, "primitive"));

            let complex_variant_key_matches = complex_variants.iter().map(|variant| {
                let variant_ident = variant.ident.clone();
                let key = format!("{}{}", type_choice_field_name, variant_ident);
                quote! {
                    #key => {
                        Ok(Some(Self::#variant_ident(map.next_value()?)))
                    }
                }
            });

            let primitive_variant_key_matches = primitive_variants.iter().map(|variant| {
                let variant_ident = variant.ident.clone();
                let key = format!("{}{}", type_choice_field_name, variant_ident);
                quote! {
                    #key => {
                        Ok(Some(Self::#variant_ident(map.next_value()?)))
                    }
                }
            });

            let primitive_merge_matches = primitive_variants.iter().map(|variant| {
                let variant_ident = variant.ident.clone();
                let key = format!("_{}{}", type_choice_field_name, variant_ident);
                quote! {
                    (#key, Self::#variant_ident(v)) => {
                        v.extension = element.extension;
                        v.id = element.id;
                    }
                }
            });

            let deserialize_impl = quote! {
                impl #name {
                    // Returns Some(Self) if key matches any variant, None to skip unknown keys.
                    pub fn try_deserialize_from_key<'de, A: serde::de::MapAccess<'de>>(
                        key: &str,
                        map: &mut A,
                    ) -> Result<Option<Self>, A::Error> {
                        match key {
                            #(#complex_variant_key_matches,)*
                            #(#primitive_variant_key_matches,)*
                            _ => Ok(None),
                        }
                    }

                    // Merge a deferred element payload from _<choiceKey> into a primitive variant.
                    pub fn merge_element(&mut self, key: &str, element: Element) {
                        match (key, self) {
                            #(#primitive_merge_matches,)*
                            _ => {}
                        }
                    }
                }
            };

            deserialize_impl.into()
        }
        _ => panic!("Only enums can be deserialized for type choice deserializer."),
    }
}

// For primitives this is the value identifier.
fn value_ident(field_ident: &Ident) -> Ident {
    format_ident!("__{}_value", field_ident)
}

// For primitive extensions this is the extension identifier.
fn extension_ident(field_ident: &Ident) -> Ident {
    format_ident!("__{}_ext", field_ident)
}

fn typechoice_variant_found_ident(field_ident: &Ident) -> Ident {
    format_ident!("__{}_choice_variant_", field_ident)
}

fn create_complex_field_declaration(field: &FieldInformation) -> Vec<proc_macro2::TokenStream> {
    let field_ty = field.ty.clone();
    let value_ident = value_ident(&field.ident);

    match field.type_info {
        TypeInformation::Primitive => {
            let ext_ident = extension_ident(&field.ident);
            if field.is_vector {
                vec![
                    quote! { let mut #value_ident: Option<#field_ty> = None; },
                    quote! { let mut #ext_ident: Option<Vec<Option<Element>>> = None; },
                ]
            } else {
                vec![
                    quote! { let mut #value_ident: Option<#field_ty> = None; },
                    quote! { let mut #ext_ident: Option<Element> = None; },
                ]
            }
        }
        TypeInformation::TypeChoice(_) => {
            let ext_ident = extension_ident(&field.ident);
            let mut tc_declarations = if field.is_vector {
                vec![
                    quote! { let mut #value_ident: Option<#field_ty> = None; },
                    quote! { let mut #ext_ident: Option<Vec<Element>> = None; },
                ]
            } else {
                vec![
                    quote! { let mut #value_ident: Option<#field_ty> = None; },
                    quote! { let mut #ext_ident: Option<Element> = None; },
                ]
            };

            // We need to track if we've found a type choice variant for this field
            // To check that extension field aligns or field if primitive extension found first.
            let typechoice_variant_found_ident = typechoice_variant_found_ident(&field.ident);
            tc_declarations.push(quote! {
               let mut #typechoice_variant_found_ident: Option<&str> = None;
            });

            tc_declarations
        }
        TypeInformation::Complex => {
            vec![quote! { let mut #value_ident: Option<#field_ty> = None; }]
        }
    }
}

enum TypeInformation {
    Primitive,
    TypeChoice(TypeChoiceAttribute),
    Complex,
}

struct FieldInformation {
    ident: Ident,
    ty: Type,
    field_name: String,
    type_info: TypeInformation,
    is_vector: bool,
    is_optional: bool,
    #[allow(dead_code)]
    cardinality: Option<CardinalityAttribute>,
}

// Get the various metadata extracted from the field.
fn process_field(field: &Field) -> FieldInformation {
    let is_primitive = is_attribute_present(&field.attrs, "primitive");
    let type_choice_attr = get_type_choice_attribute(&field.attrs);
    let is_type_choice = type_choice_attr.is_some();

    FieldInformation {
        ident: field.ident.clone().unwrap(),
        ty: field.ty.clone(),
        field_name: get_field_name(field),
        is_vector: is_vector(field),
        is_optional: is_optional_field(field),
        cardinality: get_cardinality_attributes(&field.attrs),

        type_info: if is_primitive {
            TypeInformation::Primitive
        } else if is_type_choice {
            TypeInformation::TypeChoice(type_choice_attr.unwrap())
        } else {
            TypeInformation::Complex
        },
    }
}

// Handle optional vs non optional for typechoice setter.
fn typechoice_value_setter(
    field: &FieldInformation,
    value_ident: &Ident,
    field_name: &str,
) -> proc_macro2::TokenStream {
    let typechoice_type: Type = if field.is_optional {
        get_optional_inner_type(&field.ty).unwrap()
    } else {
        field.ty.clone()
    };

    if field.is_optional {
        quote! { #value_ident = Some(#typechoice_type::try_deserialize_from_key(#field_name, &mut map)?); }
    } else {
        quote! { #value_ident = #typechoice_type::try_deserialize_from_key(#field_name, &mut map)?; }
    }
}

pub fn complex_deserialization(
    input: DeriveInput,
    deserialize_complex_type: DeserializeComplexType,
) -> TokenStream {
    let name = input.ident;
    match input.data {
        Data::Struct(data) => {
            let visitor_name = format_ident!("{}Visitor", name);
            let name_str = name.to_string();
            let seen_resource_type_ident = format_ident!("__seen_resource_type");

            // Declare all fields for the given struct.
            // Make all fields optional at this stage to allow for partial construction during deserialization,
            // we'll validate required fields at the end.

            let field_meta = data.fields.iter().map(process_field).collect::<Vec<_>>();

            let field_declarations = field_meta
                .iter()
                .flat_map(|field| create_complex_field_declaration(field));

            let seen_resource_decl = if deserialize_complex_type == DeserializeComplexType::Resource
            {
                quote! { let mut #seen_resource_type_ident = false; }
            } else {
                quote! {}
            };

            let mut key_match_arms = Vec::new();

            if deserialize_complex_type == DeserializeComplexType::Resource {
                key_match_arms.push(quote! {
                    "resourceType" => {
                        if #seen_resource_type_ident {
                            return Err(serde::de::Error::duplicate_field("resourceType"));
                        }
                        let resource_type: String = map.next_value()?;
                        if resource_type != #name_str {
                            return Err(serde::de::Error::custom(format!(
                                "Invalid resourceType for {}: {}",
                                #name_str,
                                resource_type
                            )));
                        }
                        #seen_resource_type_ident = true;
                    }
                });
            }

            for field in field_meta.iter() {
                let value_ident = value_ident(&field.ident);
                let ext_ident = extension_ident(&field.ident);
                let value_field_name = &field.field_name;
                let ext_field_name = format!("_{}", field.field_name);

                match &field.type_info {
                    TypeInformation::Primitive => {
                        key_match_arms.push(quote! {
                            #value_field_name => {
                                if #value_ident.is_some() {
                                    return Err(serde::de::Error::duplicate_field(#value_field_name));
                                }
                                #value_ident = Some(map.next_value()?);
                            }
                        });
                        if field.is_vector {
                            key_match_arms.push(quote! {
                                #ext_field_name => {
                                    if #ext_ident.is_some() {
                                        return Err(serde::de::Error::duplicate_field(#ext_field_name));
                                    }
                                    #ext_ident = Some(map.next_value::<Vec<Option<Element>>>()?);
                                }
                            });
                        } else {
                            key_match_arms.push(quote! {
                                #ext_field_name => {
                                    if #ext_ident.is_some() {
                                        return Err(serde::de::Error::duplicate_field(#ext_field_name));
                                    }
                                    #ext_ident = Some(map.next_value::<Element>()?);
                                }
                            });
                        }
                    }
                    TypeInformation::TypeChoice(type_choice_attributes) => {
                        let complex_variants = &type_choice_attributes.complex_variants;
                        let primitives = &type_choice_attributes.primitive_variants;

                        for primitive_variant_fieldname in primitives {
                            let value_setter = typechoice_value_setter(
                                field,
                                &value_ident,
                                primitive_variant_fieldname,
                            );
                            key_match_arms.push(quote! {
                                #primitive_variant_fieldname => {
                                    if #value_ident.is_some() {
                                        return Err(serde::de::Error::duplicate_field(#ext_field_name));
                                    }
                                    #value_setter
                                }
                            });
                            key_match_arms.push(quote! {
                                #ext_field_name => {
                                    if #ext_ident.is_some() {
                                        return Err(serde::de::Error::duplicate_field(#ext_field_name));
                                    }
                                    #ext_ident = Some(map.next_value::<Element>()?);
                                }
                            });
                        }

                        for complex_variant_fieldname in complex_variants {
                            let value_setter = typechoice_value_setter(
                                field,
                                &value_ident,
                                complex_variant_fieldname,
                            );

                            key_match_arms.push(quote! {
                                #complex_variant_fieldname => {
                                    if #value_ident.is_some() {
                                        return Err(serde::de::Error::duplicate_field(#complex_variant_fieldname));
                                    }
                                    #value_setter
                                }
                            });
                        }
                    }
                    TypeInformation::Complex => {
                        key_match_arms.push(quote! {
                            #value_field_name => {
                                if #value_ident.is_some() {
                                    return Err(serde::de::Error::duplicate_field(#value_field_name));
                                }
                                #value_ident = Some(map.next_value()?);
                            }
                        });
                    }
                }
            }

            // let bind_fields = data.fields.iter().map(|field| {
            //     let field_ident = field.ident.as_ref().unwrap();
            //     let field_name = get_field_name(field);
            //     let value_ident = format_ident!("__{}_value", field_ident);
            //     if is_optional_field(field) {
            //         quote! { let #field_ident = #value_ident.and_then(|v| v); }
            //     } else {
            //         quote! {
            //             let #field_ident = #value_ident
            //                 .ok_or_else(|| serde::de::Error::missing_field(#field_name))?;
            //         }
            //     }
            // });

            // let field_names = data.fields.iter().map(|f| f.ident.as_ref().unwrap());

            // #(#bind_fields)*

            // Ok(#name {
            //     #(#field_names),*
            // })

            let required_resource_check =
                if deserialize_complex_type == DeserializeComplexType::Resource {
                    quote! {
                        if !#seen_resource_type_ident {
                            return Err(serde::de::Error::missing_field("resourceType"));
                        }
                    }
                } else {
                    quote! {}
                };

            let deserialize_impl = quote! {
                impl<'de> serde::Deserialize<'de> for #name {
                    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
                        struct #visitor_name;
                        impl<'de> serde::de::Visitor<'de> for #visitor_name {
                            type Value = #name;

                            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                                write!(f, "a JSON object for {}", #name_str)
                            }

                            fn visit_map<A>(self, mut map: A) -> Result<#name, A::Error>
                            where
                                A: serde::de::MapAccess<'de>,
                            {
                                #(#field_declarations)*
                                #seen_resource_decl

                                while let Some(key) = map.next_key::<String>()? {
                                    match key.as_str() {
                                        #(#key_match_arms)*
                                        _ => {
                                            return Err(serde::de::Error::unknown_field(key.as_str(), &[]));
                                        }
                                    }
                                }


                                #required_resource_check

                                todo!("Not implemented.")
                            }
                        }

                        d.deserialize_map(#visitor_name)
                    }
                }
            };

            // if name == "AuditEventEntityDetail" {
            //     println!("{}", deserialize_impl.to_string());
            // }

            deserialize_impl.into()
        }
        _ => panic!("Only structs can be deserialized for complex deserializer."),
    }
}
