use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{Data, DeriveInput};

use crate::{
    DeserializeComplexType,
    utilities::{
        TypeInformation, get_attribute_value, is_attribute_present, is_optional_field,
        is_type_string, process_field,
    },
};

fn extension_derive() -> proc_macro2::TokenStream {
    quote! {
         struct Companion<'a, Ext: serde::Serialize> {
            id: &'a Option<String>,
            extension: &'a Option<Vec<Box<Ext>>>,
        }

        impl<'a, Ext: serde::Serialize> serde::Serialize for Companion<'a, Ext> {
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                use serde::ser::SerializeMap;
                let mut m = serializer.serialize_map(None)?;
                if let Some(id) = self.id {
                    m.serialize_entry("id", id)?;
                }
                if let Some(ext) = self.extension {
                    m.serialize_entry("extension", ext)?;
                }
                m.end()
            }
        }
    }
}

pub fn fhir_primitive_serialization(input: DeriveInput) -> TokenStream {
    let name = input.ident;

    let extension_derive = extension_derive();

    match input.data {
        Data::Struct(data) => {
            let value_field = data
                .fields
                .iter()
                .find(|f| f.ident.as_ref().unwrap() == "value")
                .unwrap();
            // Value could be optional or not depending on SD.
            let function_to_check_empty = if is_optional_field(&value_field) {
                // Special handling for string types: empty string is considered empty even if the field is present.
                if is_type_string(&value_field.ty) {
                    quote! {
                        as_ref().map(|s| s.is_empty()).unwrap_or(true)
                    }
                } else {
                    quote! {
                        is_none()
                    }
                }
            // Markdown has a non-optional value field, but empty string is considered empty.
            // Should only be string that hits this.
            } else {
                quote! {
                    is_empty()
                }
            };

            let serialize = quote! {
                impl serde::Serialize for #name {
                    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                        where
                            S: serde::Serializer,
                        {
                        self.value.serialize(serializer)
                    }
                }

                impl #name {
                    pub fn serialize_as_field<M: serde::ser::SerializeMap>(&self, field_name: &str, serializer: &mut M) -> Result<(), M::Error> {
                        if !self.value.#function_to_check_empty {
                            serializer.serialize_entry(field_name, &self.value)?;
                        }

                        if self.extension.is_some() || self.id.is_some() {
                            let element_key = format!("_{}", field_name);

                            // Inline companion serializer so we don't depend on Element type here.
                            #extension_derive

                            serializer.serialize_entry(
                                &element_key,
                                &Companion { id: &self.id, extension: &self.extension },
                            )?;
                        }

                        Ok(())
                    }

                    pub fn serialize_as_vector<M: serde::ser::SerializeMap>(field_name: &str, values: &[Box<Self>], serializer: &mut M) -> Result<(), M::Error> {
                        let has_extensions = values.iter().any(|item| item.extension.is_some() || item.id.is_some());

                        if has_extensions {
                            let element_key = format!("_{}", field_name);

                            // Inline companion serializer so we don't depend on Element type here.
                            #extension_derive

                            let extension_serializations: Vec<Option<_>> = values
                                .iter()
                                .map(|item| {
                                    if item.extension.is_some() || item.id.is_some() {
                                        Some(Companion { id: &item.id, extension: &item.extension })
                                    } else {
                                        None
                                    }
                                })
                                .collect();

                            serializer.serialize_entry(&element_key, &extension_serializations)?;
                        }

                        let value_array: Vec<_> = values.iter().map(|v| &v.value).collect();

                        if value_array.iter().any(|v| !v.#function_to_check_empty) {
                            serializer.serialize_entry(field_name, &value_array)?;
                        }



                        Ok(())
                    }
                }
            };

            serialize.into()
        }
        _ => panic!("FHIR primitives must be structs with a single value field."),
    }
}

pub fn complex_serialization(
    input: DeriveInput,
    deserialize_complex_type: DeserializeComplexType,
) -> TokenStream {
    let name = input.ident;
    let name_string = name.to_string();
    match input.data {
        Data::Struct(data) => {
            let field_information = data.fields.iter().map(process_field).collect::<Vec<_>>();
            let map_serializer = format_ident!("___serializer");

            let serialize_resourcetype =
                if deserialize_complex_type == DeserializeComplexType::Resource {
                    quote! {
                        #map_serializer.serialize_entry("resourceType", #name_string)?;
                    }
                } else {
                    quote! {}
                };

            let instantiation_serialize = field_information.iter().map(|field| {
                let field_ident = &field.ident;
                let field_name = &field.field_name;
                let field_type = &field.field_type;

                let serialize_field = match field.type_info {
                    TypeInformation::Primitive => {
                        if field.is_vector {
                            quote!{
                                #field_type::serialize_as_vector(#field_name, #field_ident.as_slice(), &mut #map_serializer)?;
                            }
                        } else {
                            quote!{
                                #field_ident.serialize_as_field(#field_name, &mut #map_serializer)?;
                            }
                        }
                    }
                    TypeInformation::Complex => {
                        quote! {
                            #map_serializer.serialize_entry(#field_name, #field_ident)?;
                        }
                    }
                    TypeInformation::TypeChoice(_) => {
                        quote! {
                            #field_ident.serialize_as_field(#field_name, &mut #map_serializer)?;
                        }
                    }
                };

                if field.is_optional {
                    quote! {
                        if let Some(#field_ident) = &self.#field_ident {
                            #serialize_field
                        }
                    }
                } else {
                    quote! {
                        let #field_ident = &self.#field_ident;
                        #serialize_field
                    }
                }
            });

            let serialize = quote! {
                impl serde::Serialize for #name {
                    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                    where
                        S: serde::Serializer,
                    {
                        use serde::ser::SerializeMap;
                        let mut #map_serializer = serializer.serialize_map(None)?;
                        #serialize_resourcetype
                        #(#instantiation_serialize)*
                        #map_serializer.end()
                    }
                }
            };

            // if name.to_string() == "ExampleScenario" {
            //     println!("{}", serialize.to_string());
            // }

            serialize.into()
        }
        _ => panic!("Complex types must be structs."),
    }
}

pub fn valueset_serialization(input: DeriveInput) -> TokenStream {
    let name = input.ident;
    match input.data {
        Data::Enum(_data) => {
            // let serialize_field_variants = data.variants.iter().map(|v| {
            //     let variant_name = &v.ident;
            // });

            let serialize = quote! {
                impl serde::Serialize for #name {
                    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                    where
                        S: serde::Serializer,
                    {
                        let s: Option<String> = self.into();
                        s.serialize(serializer)
                    }
                }

                impl #name {
                    pub fn serialize_as_field<M: serde::ser::SerializeMap>(&self, field_name: &str, serializer: &mut M) -> Result<(), M::Error> {
                        let s: Option<String> = self.into();
                        let element = self.element();

                        if let Some(value) = s {
                            serializer.serialize_entry(field_name, &value)?;
                        }

                        if let Some(element) = element {
                            let element_key = format!("_{}", field_name);
                            serializer.serialize_entry(&element_key, element)?;
                        }

                        Ok(())
                    }

                    pub fn serialize_as_vector<M: serde::ser::SerializeMap>(field_name: &str, values: &[Box<Self>], serializer: &mut M) -> Result<(), M::Error> {
                        let value_array: Vec<Option<String>> = values.iter().map(|v| v.as_ref().into()).collect();
                        let element_array: Vec<Option<_>> = values.iter().map(|v| v.element()).collect();

                        if value_array.iter().any(|v| v.is_some()) {
                            serializer.serialize_entry(field_name, &value_array)?;
                        }

                        if element_array.iter().any(|e| e.is_some()) {
                            let element_key = format!("_{}", field_name);
                            let element_array: Vec<Option<_>> = values.iter().map(|v| v.element()).collect();
                            serializer.serialize_entry(&element_key, &element_array)?;
                        }

                        Ok(())
                    }
                }
            };

            serialize.into()
        }
        _ => panic!("ValueSets must be enums."),
    }
}

pub fn typechoice_serialization(input: DeriveInput) -> TokenStream {
    let name = input.ident;
    match input.data {
        Data::Enum(data) => {
            let typechoice_name =
                get_attribute_value(&input.attrs, "type_choice_field_name").unwrap();

            let serialize_field_variants = data.variants.iter().map(|v| {
                let variant_name = &v.ident;
                let variant_name_str = variant_name.to_string();
                let is_primitive = is_attribute_present(&v.attrs, "primitive");
                let variant_name_str = typechoice_name.clone() + &variant_name_str;

                if is_primitive {
                    quote! {
                        #name::#variant_name(value) => value.serialize_as_field(#variant_name_str, serializer)?,
                    }
                } else {
                    quote!{
                        #name::#variant_name(value) => serializer.serialize_entry(#variant_name_str, value)?,
                    }
                }
            });

            let variants_serialize = data.variants.iter().map(|v| {
                let variant_name = &v.ident;
                quote! {
                    Self::#variant_name(value) => value.serialize(serializer),
                }
            });

            let serialize = quote! {
                impl serde::Serialize for #name {
                    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                        where
                            S: serde::Serializer,
                        {
                        match self {
                            #(#variants_serialize)*
                        }
                    }
                }

                impl #name {
                    fn serialize_as_field<M: serde::ser::SerializeMap>(&self, field_name: &str, serializer: &mut M) -> Result<(), M::Error> {
                        match self {
                            #(#serialize_field_variants)*
                        }
                        Ok(())
                    }
                }
            };

            serialize.into()
        }
        _ => panic!("Typechoice must be enums."),
    }
}

pub fn enum_variant_serialization(input: DeriveInput) -> TokenStream {
    let name = input.ident;
    match input.data {
        Data::Enum(data) => {
            let serialize_field_variants = data.variants.iter().map(|v| {
                let variant_name = &v.ident;
                quote! {
                    #name::#variant_name(v) => v.serialize(serializer),
                }
            });

            let serialize = quote! {
                impl serde::Serialize for #name {
                    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                    where
                        S: serde::Serializer,
                    {
                        match self {
                            #(#serialize_field_variants)*
                        }
                    }
                }
            };

            serialize.into()
        }
        _ => panic!("Enum variants must be enums."),
    }
}
