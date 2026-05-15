use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{Data, DeriveInput, Field, Type};

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
               impl<'de> Deserialize<'de> for #name {
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
        _ => panic!("Only structs can be serialized for primitive serializer."),
    }
}
