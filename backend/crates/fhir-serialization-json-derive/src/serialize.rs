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
                impl oxidized_fhir_serialization_json::FHIRJSONSerializer for #name {
                    fn serialize_value(&self, writer: &mut dyn std::io::Write) -> Result<bool, oxidized_fhir_serialization_json::SerializeError> {
                        self.value.serialize_value(writer)
                    }

                    fn serialize_extension(&self, writer: &mut dyn std::io::Write) -> Result<bool, oxidized_fhir_serialization_json::SerializeError> {
                        let mut tmp_buffer = std::io::BufWriter::new(Vec::new());
                        let has_extension = self.extension.is_some();
                        let has_id = self.id.is_some();

                        if has_extension {
                            tmp_buffer.write_all("\"extension\":".as_bytes())?;
                            self.extension.serialize_value(&mut tmp_buffer)?;
                            if has_id {
                                tmp_buffer.write_all(&[b','])?;
                            }
                        }

                        if has_id {
                            self.id.serialize_field("id", &mut tmp_buffer)?;
                        }

                        if has_extension || has_id {
                            tmp_buffer.flush()?;
                            let value = tmp_buffer.into_inner()?;
                            writer.write_all(&[b'{'])?;
                            writer.write_all(&value)?;
                            writer.write_all(&[b'}'])?;
                            Ok(true)
                        } else{
                            Ok(false)
                        }
                    }

                    fn serialize_field(&self, field: &str, writer: &mut dyn std::io::Write) -> Result<bool, oxidized_fhir_serialization_json::SerializeError> {
                        let mut value_buffer = std::io::BufWriter::new(Vec::new());
                        let mut extension_buffer = std::io::BufWriter::new(Vec::new());

                        let has_value = self.value.serialize_field(field, &mut value_buffer)?;
                        let has_extension = self.serialize_extension(&mut extension_buffer)?;

                        if has_value {
                             value_buffer.flush()?;
                            let value = value_buffer.into_inner()?;
                            writer.write_all(&value)?;
                            if has_extension {
                                writer.write_all(&[b','])?;
                            }
                        }
                        if has_extension {
                            extension_buffer.flush()?;
                            let extension = extension_buffer.into_inner()?;
                            writer.write_all(&[b'"', b'_'])?;
                            writer.write_all(field.as_bytes())?;
                            writer.write_all(&[b'"', b':'])?;
                            writer.write_all(&extension)?;
                        }
                        Ok(has_value || has_extension)
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
                    Self::#name(k) => k.serialize_value(writer)
                }
            });

            let variants_serialize_extension = data.variants.iter().map(|variant| {
                let name = variant.ident.to_owned();
                quote! {
                    Self::#name(k) => k.serialize_extension(writer)
                }
            });

            let variants_serialize_field = data.variants.iter().map(|variant| {
                let name = variant.ident.to_owned();
                quote! {
                    Self::#name(k) => k.serialize_field(&field, writer)
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
                impl oxidized_fhir_serialization_json::FHIRJSONSerializer for #name {
                    fn serialize_value(&self, writer: &mut dyn std::io::Write) -> Result<bool, oxidized_fhir_serialization_json::SerializeError> {
                        match self {
                            #(#variants_serialize_value),*
                        }
                    }

                    fn serialize_extension(&self, writer: &mut dyn std::io::Write) -> Result<bool, oxidized_fhir_serialization_json::SerializeError> {
                        match self {
                            #(#variants_serialize_extension),*
                        }
                    }

                    fn serialize_field(&self, field: &str, writer: &mut dyn std::io::Write) -> Result<bool, oxidized_fhir_serialization_json::SerializeError> {
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
                    // Means successful serialization so increment total.
                   if self.#accessor.serialize_field(#field_str, &mut tmp_buffer)? {
                       tmp_buffer.write_all(&[b','])?;
                       total += 1;
                   }
                }
            });

            let include_resource_type = match complex_type {
                ComplexSerializeType::Resource => quote! {
                    tmp_buffer.write_all("\"resourceType\":".as_bytes())?;
                    tmp_buffer.write_all(#resource_type.as_bytes())?;
                    tmp_buffer.write_all(&[b','])?;
                    total += 1;
                },
                ComplexSerializeType::Complex => quote! {},
            };

            let expanded = quote! {
                impl oxidized_fhir_serialization_json::FHIRJSONSerializer for #name {
                    fn serialize_value(&self, writer: &mut dyn std::io::Write) -> Result<bool, oxidized_fhir_serialization_json::SerializeError> {
                        let mut tmp_buffer = std::io::BufWriter::new(Vec::new());
                        let mut total = 0;

                        #include_resource_type

                        #(#serializers)*

                        if total == 0  {
                            return Ok(false);
                        }


                        writer.write_all(&[b'{'])?;
                        tmp_buffer.flush()?;
                        let tmp_buffer = tmp_buffer.into_inner()?;
                        // Slice off the last comma.
                        writer.write_all(&tmp_buffer[0..(tmp_buffer.len()-1)])?;
                        writer.write_all(&[b'}'])?;

                        Ok(true)
                    }

                    fn serialize_extension(&self, writer: &mut dyn std::io::Write) -> Result<bool, oxidized_fhir_serialization_json::SerializeError> {
                        Ok(false)
                    }

                    fn serialize_field(&self, field: &str, writer: &mut dyn std::io::Write) -> Result<bool, oxidized_fhir_serialization_json::SerializeError> {
                        let mut tmp_buffer = std::io::BufWriter::new(Vec::new());
                        let should_serialize = self.serialize_value(&mut tmp_buffer)?;
                        if !should_serialize {
                            return Ok(false);
                        }

                        writer.write_all(&[b'"'])?;
                        writer.write_all(field.as_bytes())?;
                        writer.write_all(&[b'"', b':'])?;
                        tmp_buffer.flush()?;
                        let tmp_buffer = tmp_buffer.into_inner()?;
                        writer.write_all(&tmp_buffer)?;

                        Ok(true)
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

pub fn value_set_serialization(input: DeriveInput) -> TokenStream {
    let enum_name = input.ident;
    match input.data {
        Data::Enum(data) => {
            let variants_serialize_value = data.variants.iter().map(|variant| {
                let name = variant.ident.to_owned();
                let code = get_attribute_value(&variant.attrs, "code")
                    .expect("Value set enum variants must have a code attribute");
                quote! {
                    Self::#name(k) => #code.to_string().serialize_value(writer)
                }
            });

            let variants_serialize_extension = data.variants.iter().map(|variant| {
                let name = variant.ident.to_owned();
                quote! {
                    Self::#name(k) => k.serialize_value(writer)
                }
            });

            let expanded = quote! {
                impl oxidized_fhir_serialization_json::FHIRJSONSerializer for #enum_name {
                    fn serialize_value(&self, writer: &mut dyn std::io::Write) -> Result<bool, oxidized_fhir_serialization_json::SerializeError> {
                        match self {
                            #(#variants_serialize_value),*
                        }
                    }

                    fn serialize_extension(&self, writer: &mut dyn std::io::Write) -> Result<bool, oxidized_fhir_serialization_json::SerializeError> {
                        match self {
                            #(#variants_serialize_extension),*
                        }
                    }

                    fn serialize_field(&self, field: &str, writer: &mut dyn std::io::Write) -> Result<bool, oxidized_fhir_serialization_json::SerializeError> {
                        let mut value_buffer = std::io::BufWriter::new(Vec::new());
                        let mut extension_buffer = std::io::BufWriter::new(Vec::new());

                        let has_value = self.value.serialize_field(field, &mut value_buffer)?;
                        let has_extension = self.serialize_extension(&mut extension_buffer)?;

                        if has_value {
                             value_buffer.flush()?;
                            let value = value_buffer.into_inner()?;
                            writer.write_all(&value)?;
                            if has_extension {
                                writer.write_all(&[b','])?;
                            }
                        }
                        if has_extension {
                            extension_buffer.flush()?;
                            let extension = extension_buffer.into_inner()?;
                            writer.write_all(&[b'"', b'_'])?;
                            writer.write_all(field.as_bytes())?;
                            writer.write_all(&[b'"', b':'])?;
                            writer.write_all(&extension)?;
                        }
                        Ok(has_value || has_extension)
                    }

                    fn is_fp_primitive(&self) -> bool {
                        true
                    }
                }
            };

            // println!("{}", expanded.to_string());
            expanded.into()
        }
        _ => panic!("Value set serialization only works for enums"),
    }
}

pub fn enum_variant_serialization(input: DeriveInput) -> TokenStream {
    let enum_name = input.ident;

    match input.data {
        Data::Enum(data) => {
            let variants_serialize_value = data.variants.iter().map(|variant| {
                let name = variant.ident.to_owned();
                quote! {
                    Self::#name(k) => k.serialize_value(writer)
                }
            });

            let variants_serialize_extension = data.variants.iter().map(|variant| {
                let name = variant.ident.to_owned();
                quote! {
                    Self::#name(k) => k.serialize_extension(writer)
                }
            });

            let variants_serialize_fields = data.variants.iter().map(|variant| {
                let name = variant.ident.to_owned();
                quote! {
                    Self::#name(k) => k.serialize_field(field, writer)
                }
            });

            let variants_is_fp_primitive = data.variants.iter().map(|variant| {
                let name = variant.ident.to_owned();
                quote! {
                    Self::#name(k) => k.is_fp_primitive()
                }
            });

            let expanded = quote! {
                impl oxidized_fhir_serialization_json::FHIRJSONSerializer for #enum_name {
                    fn serialize_value(&self, writer: &mut dyn std::io::Write) -> Result<bool, oxidized_fhir_serialization_json::SerializeError> {
                        match self {
                            #(#variants_serialize_value),*
                        }
                    }

                    fn serialize_extension(&self, writer: &mut dyn std::io::Write) -> Result<bool, oxidized_fhir_serialization_json::SerializeError> {
                        match self {
                            #(#variants_serialize_extension),*
                        }
                    }

                    fn serialize_field(&self, field: &str, writer: &mut dyn std::io::Write) -> Result<bool, oxidized_fhir_serialization_json::SerializeError> {
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
