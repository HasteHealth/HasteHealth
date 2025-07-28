use proc_macro::TokenStream;
use quote::quote;
use syn::{Attribute, Data, DeriveInput, Expr, Lit, Meta, parse_macro_input};

fn get_attribute_rename(attrs: &[Attribute]) -> Option<String> {
    attrs.iter().find_map(|attr| match &attr.meta {
        Meta::NameValue(name_value) => {
            if name_value.path.is_ident("rename_field") {
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

#[proc_macro_derive(oxidized_reflect, attributes(rename_field))]
pub fn oxidized_reflect(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    match input.data {
        Data::Struct(data) => {
            let all_fields = data
                .fields
                .iter()
                .map(|field| field.ident.to_owned().unwrap().to_string());

            let name = input.ident;
            let name_str = name.to_string();

            let accessors = data.fields.iter().map(|field| {
                let renamed = get_attribute_rename(&field.attrs);
                let name = if let Some(renamed_field) = renamed {
                    renamed_field
                } else {
                    field.ident.to_owned().unwrap().to_string()
                };

                let accessor = field.ident.to_owned().unwrap();
                quote! {
                    #name => Some(&self.#accessor)
                }
            });

            let mutable_accessor = data.fields.iter().map(|field| {
                let renamed = get_attribute_rename(&field.attrs);
                let name = if let Some(renamed_field) = renamed {
                    renamed_field
                } else {
                    field.ident.to_owned().unwrap().to_string()
                };

                let accessor = field.ident.to_owned().unwrap();
                quote! {
                    #name => Some(&mut self.#accessor)
                }
            });

            let expanded = quote! {
                impl oxidized_reflect::MetaValue for #name {
                    fn fields(&self) -> Vec<&'static str> {
                        vec![
                            #(#all_fields),*
                        ]
                    }

                    fn get_field<'a>(&'a self, field: &str) -> Option<&'a dyn MetaValue> {
                        match field {
                            #(#accessors),*
                            ,_ => None,
                        }
                    }

                    fn get_field_mut<'a>(&'a mut self, field: &str) -> Option<&'a mut dyn MetaValue> {
                         match field {
                            #(#mutable_accessor),*
                            ,_ => None,
                        }
                    }

                    fn get_index_mut<'a>(&'a mut self, index: usize) -> Option<&'a mut dyn MetaValue> {
                        None
                    }

                    fn get_index<'a>(&'a self, _index: usize) -> Option<&'a dyn MetaValue> {
                        None
                    }

                    fn typename(&self) -> &'static str {
                        #name_str
                    }

                    fn as_any(&self) -> &dyn std::any::Any {
                        self
                    }

                    fn flatten(&self) -> Vec<&dyn MetaValue> {
                        vec![self]
                    }

                }
            };

            expanded.into()
        }

        Data::Enum(data) => {
            let enum_name = input.ident;

            let variants_fields = data.variants.iter().map(|variant| {
                let name = variant.ident.to_owned();
                quote! {
                    Self::#name(k) => k.fields()
                }
            });

            let variants_get_field = data.variants.iter().map(|variant| {
                let name = variant.ident.to_owned();
                quote! {
                    Self::#name(k) => k.get_field(field)
                }
            });

            let variants_get_index = data.variants.iter().map(|variant| {
                let name = variant.ident.to_owned();
                quote! {
                    Self::#name(k) => k.get_index(field)
                }
            });

            let variants_get_field_mut = data.variants.iter().map(|variant| {
                let name = variant.ident.to_owned();
                quote! {
                    Self::#name(k) => k.get_field_mut(field)
                }
            });

            let variants_get_index_mut = data.variants.iter().map(|variant| {
                let name = variant.ident.to_owned();
                quote! {
                    Self::#name(k) => k.get_index_mut(index)
                }
            });

            let variants_typename = data.variants.iter().map(|variant| {
                let name = variant.ident.to_owned();
                quote! {
                    Self::#name(k) => k.typename()
                }
            });

            let variants_as_any = data.variants.iter().map(|variant| {
                let name = variant.ident.to_owned();
                quote! {
                    Self::#name(k) => k.as_any()
                }
            });

            let variants_flatten = data.variants.iter().map(|variant| {
                let name = variant.ident.to_owned();
                quote! {
                    Self::#name(k) => k.flatten()
                }
            });

            let expanded = quote! {
                impl oxidized_reflect::MetaValue for #enum_name {
                    fn fields(&self) -> Vec<&'static str> {
                        match self {
                            #(#variants_fields),*
                        }
                    }

                    fn get_field<'a>(&'a self, field: &str) -> Option<&'a dyn MetaValue> {
                        match self {
                            #(#variants_get_field),*
                        }
                    }

                    fn get_index<'a>(&'a self, field: usize) -> Option<&'a dyn MetaValue> {
                        match self {
                            #(#variants_get_index),*
                        }
                    }

                    fn get_field_mut<'a>(&'a mut self, field: &str) -> Option<&'a mut dyn MetaValue> {
                         match self {
                            #(#variants_get_field_mut),*
                        }
                    }

                    fn get_index_mut<'a>(&'a mut self, index: usize) -> Option<&'a mut dyn MetaValue> {
                        match self {
                            #(#variants_get_index_mut),*
                        }
                    }

                    fn typename(&self) ->  &'static str {
                        match self {
                            #(#variants_typename),*
                        }
                    }

                    fn as_any(&self) -> &dyn std::any::Any {
                        match self {
                            #(#variants_as_any),*
                        }
                    }

                    fn flatten(&self) -> Vec<&dyn MetaValue> {
                        match self {
                            #(#variants_flatten),*
                        }
                    }
                }
            };

            // println!("{}", expanded);

            expanded.into()
        }

        Data::Union(_data) => {
            todo!("Union not supported");
        }
    }
}
