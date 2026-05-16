use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{Data, DeriveInput, Field, Type, Variant};

use crate::{
    DeserializeComplexType,
    utilities::{get_attribute_value, is_attribute_present},
};

fn get_field_type(field: &Field) -> proc_macro2::Ident {
    match &field.ty {
        Type::Path(path) => path.path.segments.first().unwrap().ident.clone(),
        _ => panic!("Unsupported field type for serialization"),
    }
}

#[allow(dead_code)]
fn is_optional_field(field: &Field) -> bool {
    let field_type = get_field_type(field);
    if field_type == "Option" { true } else { false }
}

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

pub fn complex_deserialization(
    input: DeriveInput,
    _deserialize_complex_type: DeserializeComplexType,
) -> TokenStream {
    let name = input.ident;
    match input.data {
        Data::Struct(_data) => {
            let visitor_name = format_ident!("{}Visitor", name);
            let deserialize_impl = quote! {
                impl<'de> serde::Deserialize<'de> for #name {
                    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
                        struct #visitor_name;
                            impl<'de> serde::de::Visitor<'de> for #visitor_name {
                                type Value = #name;
                                fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                                    todo!("Implement complex")
                                }
                                fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<#name, E> {
                                    todo!("Implement complex")
                                }
                            }
                        d.deserialize_str(#visitor_name)
                    }
                }
            };

            deserialize_impl.into()
        }
        _ => panic!("Only structs can be deserialized for complex deserializer."),
    }
}
