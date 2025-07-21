use crate::utilities::get_attribute_value;
use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput};

pub enum ComplexSerializeType {
    Resource,
    Complex,
}

pub fn primitve_serialization(input: DeriveInput) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let name = input.ident;

    match input.data {
        Data::Struct(_data) => {
            let expanded = quote! {
                impl fhir_serialization_json::FHIRJSONSerializer for #name {
                    fn serialize_value(&self) -> Option<String> {
                        self.value.serialize_value()
                    }

                    fn serialize_extension(&self) -> Option<String> {
                        let mut result = Vec::with_capacity(2);
                        if let Some(extension) = &self.extension {
                            let mut ext_field = "\"extension\":".to_string();
                            ext_field.push_str(&extension.serialize_value()?);
                            result.push(ext_field);
                        }

                        if let Some(id_string) = self.id.serialize_field("id") {
                            result.push(id_string);
                        }

                        if result.is_empty() {
                            None
                        } else {
                            let mut res  = "{".to_string();
                            res.push_str(&result.join(","));
                            res.push_str("}");
                            Some(res)
                        }
                    }

                    fn serialize_field(&self, field: &str) -> Option<String> {
                        let value_field_string = self.value.serialize_field(field);
                        let extension_string = self.serialize_extension();

                        let mut result = Vec::with_capacity(2);

                        if let Some(value_field_string) = value_field_string {
                            result.push(value_field_string);
                        }
                        if let Some(extension_string) = extension_string {
                            let mut extension_field = "\"_".to_string();
                            extension_field.push_str(field);
                            extension_field.push_str("\"");
                            extension_field.push_str(":");
                            extension_field.push_str(&extension_string);
                            result.push(extension_field);
                        }

                        Some(result.join(","))
                    }

                    fn is_fp_primitive(&self) -> bool {
                        true
                    }
                }
            };

            // println!("{}", expanded.to_string());
            expanded.into()
        }
        _ => panic!("Only structs can be serialized for primitive serializer."),
    }
}

pub fn typechoice_serialization(input: DeriveInput) -> TokenStream {
    let name = input.ident;

    match input.data {
        Data::Enum(data) => {
            let variants_serialize_value = data.variants.iter().map(|variant| {
                let name = variant.ident.to_owned();
                quote! {
                    Self::#name(k) => k.serialize_value()
                }
            });

            let variants_serialize_extension = data.variants.iter().map(|variant| {
                let name = variant.ident.to_owned();
                quote! {
                    Self::#name(k) => k.serialize_extension()
                }
            });

            let variants_serialize_field = data.variants.iter().map(|variant| {
                let name = variant.ident.to_owned();
                quote! {
                    Self::#name(k) => k.serialize_field(&field)
                }
            });

            let variants_field_name = data.variants.iter().map(|variant| {
                let name = variant.ident.to_owned();
                let name_str = name.to_string();
                quote! {
                    Self::#name(k) => field.to_string() + #name_str
                }
            });

            let variants_is_primitive = data.variants.iter().map(|variant| {
                let name = variant.ident.to_owned();
                quote! {
                    Self::#name(k) => k.is_fp_primitive()
                }
            });

            let expanded = quote! {
                impl fhir_serialization_json::FHIRJSONSerializer for #name {
                    fn serialize_value(&self) -> Option<String> {
                        match self {
                            #(#variants_serialize_value),*
                        }
                    }

                    fn serialize_extension(&self) -> Option<String> {
                        match self {
                            #(#variants_serialize_extension),*
                        }
                    }

                    fn serialize_field(&self, field: &str) -> Option<String> {
                        let field = match self {
                            #(#variants_field_name),*
                        };
                        match self {
                            #(#variants_serialize_field),*
                        }
                    }
                    fn is_fp_primitive(&self) -> bool {
                        match self {
                            #(#variants_is_primitive),*
                        }
                    }
                }
            };

            // println!("{}", expanded);

            expanded.into()
        }
        _ => panic!("Only structs can be serialized for primitive serializer."),
    }
}

pub fn complex_serialization(
    input: DeriveInput,
    complex_type: ComplexSerializeType,
) -> TokenStream {
    let name = input.ident;
    let resource_type = format!("\"{}\"", name.to_string());
    match input.data {
        Data::Struct(data) => {
            let serializers = data.fields.iter().map(|field| {
                // If rename_field is used that means the field has been renamed because using a keyword rust.
                let field_str = if let Some(renamed_field) =
                    get_attribute_value(&field.attrs, "rename_field")
                {
                    renamed_field
                } else {
                    field.ident.to_owned().unwrap().to_string()
                };

                let accessor = field.ident.to_owned().unwrap();
                quote! {
                   if let Some(field_value) = self.#accessor.serialize_field(#field_str) {
                    serialized_fields.push(field_value);
                   }
                }
            });

            let include_resource_type = match complex_type {
                ComplexSerializeType::Resource => quote! {
                    let mut resource_type_field = "\"resourceType\":".to_string();
                    resource_type_field.push_str(#resource_type);
                    serialized_fields.push(resource_type_field);
                },
                ComplexSerializeType::Complex => quote! {},
            };

            let vector_capacity = data.fields.len() + 1;

            let expanded = quote! {
                impl fhir_serialization_json::FHIRJSONSerializer for #name {
                    fn serialize_value(&self) -> Option<String> {
                        let mut total  = 0;
                        let mut serialized_fields = Vec::with_capacity(#vector_capacity);

                        #include_resource_type

                        #(#serializers)*

                        if serialized_fields.is_empty() {
                            return None
                        }

                        let mut string_value = "{".to_string();

                        string_value.push_str(&serialized_fields.join(","));
                        string_value.push_str("}");

                        Some(string_value)
                    }

                    fn serialize_extension(&self) -> Option<String> {
                        None
                    }

                    fn serialize_field(&self, field: &str) -> Option<String> {
                        let mut string_value = "\"".to_string();

                        string_value.push_str(field);

                        string_value.push_str("\":");

                        string_value.push_str(&self.serialize_value()?);

                        Some(string_value)
                    }

                    fn is_fp_primitive(&self) -> bool {
                        false
                    }
                }
            };

            // println!("{}", expanded.to_string());

            expanded.into()
        }
        _ => panic!("Complex serialization only happens on Structs"),
    }
}

pub fn enum_variant_serialization(input: DeriveInput) -> TokenStream {
    let name = input.ident;
    match input.data {
        Data::Enum(data) => {
            let variants_serialize_value = data.variants.iter().map(|variant| {
                let name = variant.ident.to_owned();
                quote! {
                    Self::#name(k) => k.serialize_value()
                }
            });

            let variants_serialize_extension = data.variants.iter().map(|variant| {
                let name = variant.ident.to_owned();
                quote! {
                    Self::#name(k) => k.serialize_extension()
                }
            });

            let variants_serialize_fields = data.variants.iter().map(|variant| {
                let name = variant.ident.to_owned();
                quote! {
                    Self::#name(k) => k.serialize_field(field)
                }
            });

            let variants_is_fp_primitive = data.variants.iter().map(|variant| {
                let name = variant.ident.to_owned();
                quote! {
                    Self::#name(k) => k.is_fp_primitive()
                }
            });

            let expanded = quote! {
                impl fhir_serialization_json::FHIRJSONSerializer for #name {
                    fn serialize_value(&self) -> Option<String> {
                        match self {
                            #(#variants_serialize_value),*
                        }
                    }

                    fn serialize_extension(&self) -> Option<String> {
                        match self {
                            #(#variants_serialize_extension),*
                        }
                    }

                    fn serialize_field(&self, field: &str) -> Option<String> {
                        match self {
                            #(#variants_serialize_fields),*
                        }
                    }

                    fn is_fp_primitive(&self) -> bool {
                        match self {
                            #(#variants_is_fp_primitive),*
                        }
                    }
                }
            };

            expanded.into()
        }
        _ => panic!("Enum variant serialization only works for enums"),
    }
}
