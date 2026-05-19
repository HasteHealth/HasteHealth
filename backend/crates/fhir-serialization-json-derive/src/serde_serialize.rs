use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput};

use crate::DeserializeComplexType;

pub fn fhir_primitive_serialization(input: DeriveInput) -> TokenStream {
    let name = input.ident;

    match input.data {
        Data::Struct(_data) => {
            let serialize = quote! {
                impl Serialize for #name {
                    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                        where
                            S: Serializer,
                        {
                        self.value.serialize(serializer)
                    }
                }

                impl #name {
                    fn serialize_as_field(&self, field_name: &str, serializer: &mut dyn SerializeStruct) -> Result<(), serde::ser::Error> {
                        serializer.serialize_field(field_name, &self.value);
                        if self.extension.is_some() || self.id.is_some() {
                            let element_key = format!("_{}", field_name);

                            // Inline companion serializer so we don't depend on Element type here.
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

                            map.serialize_entry(
                                &element_key,
                                &Companion { id: &self.id, extension: &self.extension },
                            )?;
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
    let serialize = quote! {
        impl Serialize for Person {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                let mut s = serializer.serialize_struct("Person", 3)?;
                s.serialize_field("name", &self.name)?;
                s.serialize_field("age", &self.age)?;
                s.serialize_field("phones", &self.phones)?;
                s.end()
            }
        }
    };

    serialize.into()
}

pub fn valueset_serialization(input: DeriveInput) -> TokenStream {
    todo!();
}

pub fn typechoice_serialization(input: DeriveInput) -> TokenStream {
    todo!();
}
