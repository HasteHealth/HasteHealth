use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput};

use crate::{DeserializeComplexType, utilities::is_optional_field};

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
                quote! {
                    is_none()
                }
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
                    fn serialize_as_field<M: serde::ser::SerializeMap>(&self, field_name: &str, serializer: &mut M) -> Result<(), M::Error> {
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

                    fn serialize_as_vector<M: serde::ser::SerializeMap>(field_name: &str, values: &[Box<Self>], serializer: &mut M) -> Result<(), M::Error> {
                        let value_array: Vec<_> = values.iter().map(|v| &v.value).collect();

                        if value_array.iter().any(|v| !v.#function_to_check_empty) {
                            serializer.serialize_entry(field_name, &value_array)?;
                        }

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

                        Ok(())
                    }
                }
            };

            // if name.to_string() == "FHIRBase64Binary" {
            //     println!("{}", serialize.to_string());
            // }

            serialize.into()
        }
        _ => panic!("FHIR primitives must be structs with a single value field."),
    }
}

pub fn complex_serialization(
    input: DeriveInput,
    _deserialize_complex_type: DeserializeComplexType,
) -> TokenStream {
    let name = input.ident;
    match input.data {
        Data::Struct(_data) => {
            let serialize = quote! {
                impl serde::Serialize for #name {
                    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                    where
                        S: serde::Serializer,
                    {
                        todo!();
                    }
                }
            };

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
                    fn serialize_as_field<M: serde::ser::SerializeMap>(&self, field_name: &str, serializer: &mut M) -> Result<(), M::Error> {
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

                    fn serialize_as_vector<M: serde::ser::SerializeMap>(field_name: &str, values: &[Self], serializer: &mut M) -> Result<(), M::Error> {
                        let value_array: Vec<Option<String>> = values.iter().map(|v| v.into()).collect();
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
        Data::Enum(_data) => {
            let serialize = quote! {
                impl serde::Serialize for #name {
                    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                    where
                        S: serde::Serializer,
                    {
                        todo!();
                    }
                }
            };

            serialize.into()
        }
        _ => panic!("Typechoice must be enums."),
    }
}
