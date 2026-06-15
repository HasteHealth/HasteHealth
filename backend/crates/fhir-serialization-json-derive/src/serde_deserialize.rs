use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{Data, DeriveInput, Ident, Type, Variant};

use crate::{
    DeserializeComplexType,
    utilities::{
        CardinalityAttribute, FieldInformation, TypeInformation, get_attribute_value,
        get_field_type, get_inner_type_if_optional, is_attribute_present, process_field,
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
                        true
                    }
                }
            });

            let primitive_from_element_matches = primitive_variants.iter().map(|variant| {
                let variant_ident = variant.ident.clone();
                let variant_ty = variant
                    .fields
                    .iter()
                    .next()
                    .expect("typechoice variant must have a single field")
                    .ty
                    .clone();
                let key = format!("_{}{}", type_choice_field_name, variant_ident);
                quote! {
                    #key => {
                        let mut value: #variant_ty = Default::default();
                        value.extension = element.extension;
                        value.id = element.id;
                        Some(Self::#variant_ident(value))
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
                    pub fn merge_element(&mut self, key: &str, element: Element) -> bool {
                        match (key, self) {
                            #(#primitive_merge_matches,)*
                            _ => false,
                        }
                    }

                    // Construct a primitive variant from only an element payload (_<choiceKey>).
                    pub fn try_deserialize_primitive_element_from_key(
                        key: &str,
                        element: Element,
                    ) -> Option<Self> {
                        match key {
                            #(#primitive_from_element_matches,)*
                            _ => None,
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

// Handle optional vs non optional for typechoice setter.
fn typechoice_value_setter(
    field: &FieldInformation,
    value_ident: &Ident,
    field_name: &str,
) -> proc_macro2::TokenStream {
    let typechoice_type: Type = if field.is_optional {
        get_inner_type_if_optional(&field.ty)
    } else {
        field.ty.clone()
    };

    if field.is_optional {
        quote! { #value_ident = Some(#typechoice_type::try_deserialize_from_key(#field_name, &mut map)?); }
    } else {
        quote! { #value_ident = #typechoice_type::try_deserialize_from_key(#field_name, &mut map)?; }
    }
}

// For optional fields have two nestings Some(Some(v)) for requried fields Some(v).
fn get_matching_constraint_for_value(
    field: &FieldInformation,
    value_ident: &Ident,
) -> proc_macro2::TokenStream {
    if field.is_optional {
        quote! { Some(Some(#value_ident))}
    } else {
        quote! {Some(#value_ident) }
    }
}

fn merge_primitive_extension_tokens(field: &FieldInformation) -> proc_macro2::TokenStream {
    if !matches!(field.type_info, TypeInformation::Primitive) {
        panic!("merge_primitive_extension_tokens should only be called for primitive fields.");
    }

    let value_ident = value_ident(&field.ident);
    let ext_ident = extension_ident(&field.ident);
    let value_name = &field.field_name;

    if field.is_vector {
        let vector_pattern = format_ident!("__{}_vector", field.ident);
        let matching_constraint = get_matching_constraint_for_value(field, &vector_pattern);
        let initialize_empty = if field.is_optional {
            quote! { #value_ident = Some(Some(Vec::new())); }
        } else {
            quote! { #value_ident = Some(Vec::new()); }
        };

        quote! {
            if let Some(elements) = #ext_ident.take() {
                if #value_ident.is_none() {
                    #initialize_empty
                }

                match &mut #value_ident {
                    #matching_constraint => {
                        for (index, maybe_element) in elements.into_iter().enumerate() {
                            let Some(element) = maybe_element else {
                                continue;
                            };

                            if #vector_pattern.len() <= index {
                                #vector_pattern.resize_with(index + 1, Default::default);
                            }

                            let value = &mut #vector_pattern[index];
                            *value.extension_mut() = element.extension;
                            *value.id_mut() = element.id;
                        }
                    }
                    _ => {
                        return Err(serde::de::Error::custom(format!(
                            "Primitive field '{}' has extension entries but no value container",
                            #value_name,
                        )));
                    }
                }
            }
        }
    } else {
        let getter_setter = if field.is_optional {
            quote! { Some(Some(value)) }
        } else {
            quote! { Some(value) }
        };

        let inner_type = get_inner_type_if_optional(&field.ty);

        quote! {
            if let Some(element) = #ext_ident.take() {
                if let #getter_setter = #value_ident.as_mut() {
                    *value.extension_mut() = element.extension;
                    *value.id_mut() = element.id;
                } else {
                    let mut value: #inner_type = Default::default();
                    *value.extension_mut() = element.extension;
                    *value.id_mut() = element.id;
                    #value_ident = #getter_setter;
                }
            }
        }
    }
}

fn merge_typechoice_primitive_extension_tokens(
    field: &FieldInformation,
) -> proc_macro2::TokenStream {
    let TypeInformation::TypeChoice(_type_choice_attr) = &field.type_info else {
        panic!(
            "merge_typechoice_primitive_extension_tokens should only be called for typechoice fields."
        );
    };

    if field.is_vector {
        panic!("typechoice vector primitive extension merge is not supported yet.");
    }

    let value_ident = value_ident(&field.ident);
    let ext_ident = extension_ident(&field.ident);
    let typechoice_variant_found_ident = typechoice_variant_found_ident(&field.ident);
    let field_name = &field.field_name;
    let choice_ident = format_ident!("__{}_choice", field.ident);
    let matching_constraint = get_matching_constraint_for_value(field, &choice_ident);

    let typechoice_type: Type = if field.is_optional {
        get_inner_type_if_optional(&field.ty)
    } else {
        field.ty.clone()
    };

    let assign_created = if field.is_optional {
        quote! { #value_ident = Some(Some(created)); }
    } else {
        quote! { #value_ident = Some(created); }
    };

    quote! {
        if let Some(element) = #ext_ident.take() {
            let primitive_variant = #typechoice_variant_found_ident.ok_or_else(|| {
                serde::de::Error::custom(format!(
                    "Missing primitive type choice variant for extension field '{}'.",
                    #field_name,
                ))
            })?;
            let ext_key = format!("_{}", primitive_variant);

            match &mut #value_ident {
                #matching_constraint => {
                    if !#choice_ident.merge_element(ext_key.as_str(), element) {
                        return Err(serde::de::Error::custom(format!(
                            "Extension key '{}' does not match parsed type choice variant for '{}'.",
                            ext_key,
                            #field_name,
                        )));
                    }
                }
                _ => {
                    let created = #typechoice_type::try_deserialize_primitive_element_from_key(
                        ext_key.as_str(),
                        element,
                    )
                    .ok_or_else(|| {
                        serde::de::Error::custom(format!(
                            "Extension key '{}' is not valid for type choice '{}'.",
                            ext_key,
                            #field_name,
                        ))
                    })?;
                    #assign_created
                }
            }
        }
    }
}

fn cardinality_checks(field: &FieldInformation) -> Option<proc_macro2::TokenStream> {
    if let Some(CardinalityAttribute { min, max }) = &field.cardinality {
        let field_name = &field.field_name;
        let field_ident = &field.ident;
        let cardinality_field_ident = format_ident!("__{}_cardinality", field_ident);
        let set_cardinality_tmp_field = if field.is_optional {
            quote! {
                let #cardinality_field_ident = #field_ident.as_ref().map_or(0, |v| v.len());
            }
        } else {
            quote! {
                let #cardinality_field_ident = #field_ident.len();
            }
        };

        let min = min.unwrap_or(0);
        let min_check = if min > 0 {
            quote! {
                if #cardinality_field_ident < #min {
                    return Err(serde::de::Error::custom(format!(
                        "Field '{}' must have at least {} items.",
                        #field_name, #min
                    )));
                }
            }
        } else {
            quote! {}
        };

        let max_check = if let Some(max) = max {
            quote! {
                if #cardinality_field_ident > #max {
                    return Err(serde::de::Error::custom(format!(
                        "Field '{}' must have at most {} items.",
                        #field_name, #max
                    )));
                }
            }
        } else {
            quote! {}
        };

        Some(quote! {
            #set_cardinality_tmp_field
            #min_check
            #max_check
        })
    } else {
        None
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

            let primitive_merge_blocks = field_meta
                .iter()
                .filter(|field| matches!(field.type_info, TypeInformation::Primitive))
                .map(merge_primitive_extension_tokens)
                .collect::<Vec<_>>();

            let typechoice_merge_blocks = field_meta
                .iter()
                .filter(|field| matches!(field.type_info, TypeInformation::TypeChoice(_)))
                .map(merge_typechoice_primitive_extension_tokens)
                .collect::<Vec<_>>();

            let cardinality_checks = field_meta
                .iter()
                .filter_map(|field| cardinality_checks(field))
                .collect::<Vec<_>>();

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
                let field_name = &field.field_name;
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
                            let typechoice_variant_found_ident =
                                typechoice_variant_found_ident(&field.ident);
                            let primitive_ext_field_name =
                                format!("_{}", primitive_variant_fieldname);

                            key_match_arms.push(quote! {
                                #primitive_variant_fieldname => {
                                    if #value_ident.is_some() {
                                        return Err(serde::de::Error::duplicate_field(#ext_field_name));
                                    }

                                    if let Some(existing_variant) = #typechoice_variant_found_ident
                                        && existing_variant != #primitive_variant_fieldname {
                                        return Err(serde::de::Error::custom(format!(
                                            "Multiple primitive type choice variants for '{}': '{}' and '{}'.",
                                            #field_name,
                                            existing_variant,
                                            #primitive_variant_fieldname,
                                        )));
                                    }

                                    #value_setter
                                    #typechoice_variant_found_ident = Some(#primitive_variant_fieldname);
                                }
                            });

                            key_match_arms.push(quote! {
                                #primitive_ext_field_name => {
                                    if #ext_ident.is_some() {
                                        return Err(serde::de::Error::duplicate_field(#primitive_ext_field_name));
                                    }

                                    if let Some(existing_variant) = #typechoice_variant_found_ident
                                        && existing_variant != #primitive_variant_fieldname {
                                        return Err(serde::de::Error::custom(format!(
                                            "Extension for primitive type choice variant '{}' conflicts with already parsed variant '{}' for '{}'.",
                                            #primitive_variant_fieldname,
                                            existing_variant,
                                            #field_name,
                                        )));
                                    }

                                    #ext_ident = Some(map.next_value::<Element>()?);
                                    #typechoice_variant_found_ident = Some(#primitive_variant_fieldname);
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

            let bind_fields = field_meta.iter().map(|field| {
                let field_name = field.field_name.as_str();
                let field_ident = &field.ident;
                let value_ident = value_ident(field_ident);

                if field.is_optional {
                    // Special handling because empty vector should be treated as None.
                    // Because generated type will always have Option<Vec<T>> for empty vecs we should be okay 
                    // just handling edgecase here.
                    if field.is_vector {
                        quote! {
                            let value_len = #value_ident.as_ref().and_then(|v| v.as_ref()).map_or(0, |v| v.len());
                            let #field_ident = if value_len == 0 {
                                None
                            } else {
                                #value_ident.and_then(|v| v)
                            };
                        }
                    } else {
                        quote! { let #field_ident = #value_ident.and_then(|v| v); }
                    }
                } else {
                    quote! {
                        let #field_ident = #value_ident
                            .ok_or_else(|| serde::de::Error::missing_field(#field_name))?;
                    }
                }
            });

            let field_names = data.fields.iter().map(|f| f.ident.as_ref().unwrap());

            // Temp disabling because tag removes the field. Can add this back when I alter Resource enum to not remove the field.
            // let required_resource_check =
            //     if deserialize_complex_type == DeserializeComplexType::Resource {
            //         quote! {
            //             if !#seen_resource_type_ident {
            //                 return Err(serde::de::Error::missing_field("resourceType"));
            //             }
            //         }
            //     } else {
            //         quote! {}
            //     };

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

                                #(#primitive_merge_blocks)*
                                #(#typechoice_merge_blocks)*





                                #(#bind_fields)*

                                #(#cardinality_checks)*

                                Ok(#name {
                                    #(#field_names),*
                                })
                            }
                        }

                        d.deserialize_map(#visitor_name)
                    }
                }
            };

            // if name == "ClientApplication" {
            //     println!("{}", deserialize_impl.to_string());
            // }

            deserialize_impl.into()
        }
        _ => panic!("Only structs can be deserialized for complex deserializer."),
    }
}
