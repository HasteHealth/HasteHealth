use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{Attribute, Data, DataStruct, DeriveInput, Ident, Type, Variant};

use crate::{
    DeserializeComplexType,
    utilities::{
        CardinalityAttribute, FieldInformation, TypeChoiceAttribute, TypeInformation,
        get_attribute_value, get_field_type, get_inner_type_if_optional, is_attribute_present,
        is_optional_field, is_type_string, process_field,
    },
};

fn fhir_primitive_value_deserialization(
    value_field_found: &syn::Field,
) -> proc_macro2::TokenStream {
    let is_optional = is_optional_field(value_field_found);
    let value_type = get_field_type(value_field_found);

    // Empty String should be treated as None. Special handling for this case.
    // For cases where value is required (ie non optional) we should error if value is empty string.
    if is_type_string(&value_field_found.ty) {
        if is_optional {
            quote! {
               let value = Option::<String>::deserialize(deserializer)?;
               let value = if let Some(v) = value.as_ref() && v.is_empty() {
                  None
               } else {
                  value
               };
            }
        } else {
            quote! {
               let value = #value_type::deserialize(deserializer)?;
               if value.is_empty() {
                   return Err(serde::de::Error::custom("Value field cannot be empty for non optional string primitive."));
               }
            }
        }
    } else {
        quote! { let value = #value_type::deserialize(deserializer)?; }
    }
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
                .find(|f| f.ident == Some(format_ident!("value")))
                .expect("value field is required for primitive deserialization");

            let value_deserialization = fhir_primitive_value_deserialization(value_field_found);

            // For markdown requires field value so always not empty when deserialized from above.
            let empty_check = if is_optional_field(value_field_found) {
                quote! { self.value.is_none() && self.id.is_none() && self.extension.is_none() }
            } else {
                quote! { false }
            };

            let deserialize_impl = quote! {
               impl<'de> serde::Deserialize<'de> for #name {
                    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                    where
                        D: serde::Deserializer<'de>,
                    {
                        #value_deserialization
                        Ok(#name {
                            id: None,
                            extension: None,
                            value,
                        })
                    }
                }

                impl #name {
                    pub fn empty(&self) -> bool {
                        #empty_check
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
                let variant_name = variant.ident.clone();
                let code = get_attribute_value(&variant.attrs, "code");
                code.map(|code| {
                    quote! {
                      #code =>  Ok(#name::#variant_name(None))
                    }
                })
            });

            let variants_merge_element = data.variants.iter().map(|variant| {
                let variant_name = variant.ident.clone();
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

                    pub fn empty(&self) -> bool {
                        match self {
                            Self::Null(e) => {
                                if let Some(e) = e.as_ref() {
                                  e.id.is_none() && e.extension.is_none()
                                } else {
                                  false
                                }
                            },
                            _ => false,
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
    // 1. Extract attributes and enum metadata
    let type_choice_field_name = get_attribute_value(&input.attrs, "type_choice_field_name")
        .expect("type_choice_field_name attribute is required for typechoice deserialization");

    let name = input.ident;
    let Data::Enum(data_enum) = input.data else {
        panic!("Only enums can be deserialized for type choice deserializer.")
    };

    // 2. Split variants into primitive and complex variants
    let (primitive_variants, complex_variants): (Vec<Variant>, Vec<Variant>) = data_enum
        .variants
        .into_iter()
        .partition(|variant| is_attribute_present(&variant.attrs, "primitive"));

    // 3. Generate the required code fragments
    let key_matches = gen_key_matches(
        &type_choice_field_name,
        &complex_variants,
        &primitive_variants,
    );
    let merge_matches = gen_primitive_merge_matches(&type_choice_field_name, &primitive_variants);
    let element_matches =
        gen_primitive_from_element_matches(&type_choice_field_name, &primitive_variants);
    let empty_matches = gen_empty_matches(&complex_variants, &primitive_variants);

    // 4. Generate the final implementation
    quote! {
        impl #name {
            // Attempts to deserialize a variant from the given key.
            // Returns `Some(Self)` if the key matches a known variant,
            // or `None` if the key is unknown and should be ignored.
            pub fn try_deserialize_from_key<'de, A: serde::de::MapAccess<'de>>(
                key: &str,
                map: &mut A,
            ) -> Result<Option<Self>, A::Error> {
                match key {
                    #key_matches
                    _ => Ok(None),
                }
            }

            // Merges a deferred element payload from `_<choiceKey>` into
            // an existing primitive variant.
            pub fn merge_element(&mut self, key: &str, element: Element) -> bool {
                match (key, self) {
                    #merge_matches
                    _ => false,
                }
            }

            // Constructs a primitive variant from a standalone element
            // payload (`_<choiceKey>`).
            pub fn try_deserialize_primitive_element_from_key(
                key: &str,
                element: Element,
            ) -> Option<Self> {
                match key {
                    #element_matches
                    _ => None,
                }
            }

            // Returns whether the variant contains no meaningful data.
            pub fn empty(&self) -> bool {
                match self {
                    #empty_matches
                }
            }
        }
    }
    .into()
}

fn get_variant_single_field_type(variant: &Variant) -> syn::Type {
    variant
        .fields
        .iter()
        .next()
        .expect("typechoice variant must have a single field")
        .ty
        .clone()
}

fn gen_key_matches(
    field_name: &str,
    complex: &[Variant],
    primitive: &[Variant],
) -> proc_macro2::TokenStream {
    let matches = complex.iter().chain(primitive.iter()).map(|variant| {
        let variant_ident = &variant.ident;
        let key = format!("{field_name}{variant_ident}");
        quote! {
            #key => Ok(Some(Self::#variant_ident(map.next_value()?))),
        }
    });
    quote! { #(#matches)* }
}

fn gen_primitive_merge_matches(
    field_name: &str,
    primitive: &[Variant],
) -> proc_macro2::TokenStream {
    let matches = primitive.iter().map(|variant| {
        let variant_ident = &variant.ident;
        let key = format!("_{field_name}{variant_ident}");
        quote! {
            (#key, Self::#variant_ident(v)) => {
                v.extension = element.extension;
                v.id = element.id;
                true
            }
        }
    });
    quote! { #(#matches)* }
}

fn gen_primitive_from_element_matches(
    field_name: &str,
    primitive: &[Variant],
) -> proc_macro2::TokenStream {
    let matches = primitive.iter().map(|variant| {
        let variant_ident = &variant.ident;
        let variant_ty = get_variant_single_field_type(variant);
        let key = format!("_{field_name}{variant_ident}");
        quote! {
            #key => {
                let mut value: #variant_ty = Default::default();
                value.extension = element.extension;
                value.id = element.id;
                Some(Self::#variant_ident(value))
            }
        }
    });
    quote! { #(#matches)* }
}

fn gen_empty_matches(complex: &[Variant], primitive: &[Variant]) -> proc_macro2::TokenStream {
    let matches = complex.iter().chain(primitive.iter()).map(|variant| {
        let variant_ident = &variant.ident;
        quote! {
            Self::#variant_ident(v) => v.empty(),
        }
    });
    quote! { #(#matches)* }
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
    assert!(
        matches!(field.type_info, TypeInformation::Primitive),
        "merge_primitive_extension_tokens should only be called for primitive fields."
    );

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

    assert!(
        !field.is_vector,
        "typechoice vector primitive extension merge is not supported yet."
    );

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

// Must be post binding.
fn filter_empty_values(field: &FieldInformation) -> proc_macro2::TokenStream {
    let field_ident = &field.ident;
    let field_name = &field.field_name;

    if field.is_vector {
        let filtered_vec = if field.is_optional {
            quote! {
                #field_ident = #field_ident.and_then(|vec| {
                    let tmp = vec.into_iter().filter(|v| !v.empty()).collect::<Vec<_>>();
                    if tmp.len() == 0 {
                        None
                    } else {
                        Some(tmp)
                    }
                });

            }
        } else {
            quote! { #field_ident = #field_ident.into_iter().filter(|v| !v.empty()).collect(); }
        };

        quote! {
          #filtered_vec
        }
    } else if field.is_optional {
        quote! {
            if let Some(v) = &#field_ident && v.empty() {
                #field_ident = None;
            }
        }
    } else {
        quote! {
            if #field_ident.empty() {
                    return Err(serde::de::Error::custom(format!(
                        "Required field '{}' has no value, id, or extension.",
                        #field_name,
                    )));
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

// If all fields are empty we can treat the whole complex type as empty and remove it from memory struct.
pub fn can_complex_be_empty(field_meta: &[FieldInformation]) -> bool {
    for field in field_meta {
        if !field.is_optional {
            return false;
        }
    }
    true
}

pub fn complex_empty_impl(
    field_meta: &[FieldInformation],
    struct_name: &Ident,
) -> proc_macro2::TokenStream {
    if can_complex_be_empty(field_meta) {
        let empty_checks = field_meta.iter().map(|field| {
            let field_ident = &field.ident;
            quote! { self.#field_ident.is_none() }
        });

        quote! {
            impl #struct_name {
                pub fn empty(&self) -> bool {
                    #(#empty_checks)&&*
                }
            }
        }
    } else {
        quote! {
            impl #struct_name {
                pub fn empty(&self) -> bool {
                    false
                }
            }
        }
    }
}

pub fn complex_deserialization(
    input: DeriveInput,
    deserialize_complex_type: DeserializeComplexType,
) -> TokenStream {
    let struct_ident = input.ident;

    match input.data {
        Data::Struct(data) => {
            let field_meta = data.fields.iter().map(process_field).collect::<Vec<_>>();

            generate_complex_deserializer(
                &struct_ident,
                &data,
                &field_meta,
                deserialize_complex_type,
                &input.attrs,
            )
            .into()
        }
        _ => panic!("Only structs can be deserialized for complex deserializer."),
    }
}

fn generate_complex_deserializer(
    struct_ident: &syn::Ident,
    data: &DataStruct,
    field_meta: &[FieldInformation],
    deserialize_complex_type: DeserializeComplexType,
    attrs: &[Attribute],
) -> proc_macro2::TokenStream {
    let visitor_name = format_ident!("{}Visitor", struct_ident);
    let name_str = struct_ident.to_string();

    let field_declarations = field_meta.iter().flat_map(create_complex_field_declaration);

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

    let value_presence_filtering = field_meta
        .iter()
        .filter(|f| !is_type_string(&f.field_type))
        .map(filter_empty_values)
        .collect::<Vec<_>>();

    let cardinality_checks = field_meta
        .iter()
        .filter_map(cardinality_checks)
        .collect::<Vec<_>>();

    let seen_resource_decl = generate_seen_resource_declaration(deserialize_complex_type);

    let key_match_arms =
        generate_key_match_arms(field_meta, deserialize_complex_type, attrs, &name_str);

    let bind_fields = field_meta.iter().map(|field| {
        let field_name = field.field_name.as_str();
        let field_ident = &field.ident;
        let value_ident = value_ident(field_ident);

        if field.is_optional {
            quote! {
                let mut #field_ident = #value_ident.and_then(|v| v);
            }
        } else {
            quote! {
                let mut #field_ident = #value_ident
                    .ok_or_else(|| serde::de::Error::missing_field(#field_name))?;
            }
        }
    });

    let field_names = data.fields.iter().map(|f| f.ident.as_ref().unwrap());

    let empty_impl = complex_empty_impl(field_meta, struct_ident);

    quote! {
        impl<'de> serde::Deserialize<'de> for #struct_ident {
            fn deserialize<D: serde::Deserializer<'de>>(
                d: D
            ) -> Result<Self, D::Error> {
                struct #visitor_name;

                impl<'de> serde::de::Visitor<'de> for #visitor_name {
                    type Value = #struct_ident;

                    fn expecting(
                        &self,
                        f: &mut std::fmt::Formatter
                    ) -> std::fmt::Result {
                        write!(f, "a JSON object for {}", #name_str)
                    }

                    fn visit_map<A>(
                        self,
                        mut map: A
                    ) -> Result<#struct_ident, A::Error>
                    where
                        A: serde::de::MapAccess<'de>,
                    {
                        #(#field_declarations)*

                        #seen_resource_decl

                        while let Some(key) = map.next_key::<String>()? {
                            match key.as_str() {
                                #(#key_match_arms)*

                                _ => {
                                    return Err(
                                        serde::de::Error::unknown_field(
                                            key.as_str(),
                                            &[]
                                        )
                                    );
                                }
                            }
                        }

                        #(#primitive_merge_blocks)*
                        #(#typechoice_merge_blocks)*

                        #(#bind_fields)*

                        #(#value_presence_filtering)*

                        #(#cardinality_checks)*

                        Ok(#struct_ident {
                            #(#field_names),*
                        })
                    }
                }

                d.deserialize_map(#visitor_name)
            }
        }

        #empty_impl
    }
}

fn generate_seen_resource_declaration(
    deserialize_complex_type: DeserializeComplexType,
) -> proc_macro2::TokenStream {
    if deserialize_complex_type == DeserializeComplexType::Resource {
        let ident = format_ident!("__seen_resource_type");

        quote! {
            let mut #ident = false;
        }
    } else {
        quote! {}
    }
}

fn generate_key_match_arms(
    field_meta: &[FieldInformation],
    deserialize_complex_type: DeserializeComplexType,
    attrs: &[Attribute],
    struct_name: &str,
) -> Vec<proc_macro2::TokenStream> {
    let mut key_match_arms = Vec::new();

    if deserialize_complex_type == DeserializeComplexType::Resource {
        key_match_arms.push(generate_resource_type_match_arm(attrs, struct_name));
    }

    for field in field_meta {
        key_match_arms.extend(generate_field_match_arms(field));
    }

    key_match_arms
}

fn generate_resource_type_match_arm(
    attrs: &[Attribute],
    struct_name: &str,
) -> proc_macro2::TokenStream {
    let seen_resource_type_ident = format_ident!("__seen_resource_type");

    let resource_type =
        get_attribute_value(attrs, "fhir_resource_type").unwrap_or_else(|| struct_name.to_string());

    quote! {
        "resourceType" => {
            if #seen_resource_type_ident {
                return Err(
                    serde::de::Error::duplicate_field("resourceType")
                );
            }

            let resource_type: String = map.next_value()?;

            if resource_type != #resource_type {
                return Err(
                    serde::de::Error::custom(format!(
                        "Invalid resourceType for {}: {}",
                        #resource_type,
                        resource_type
                    ))
                );
            }

            #seen_resource_type_ident = true;
        }
    }
}

fn generate_field_match_arms(field: &FieldInformation) -> Vec<proc_macro2::TokenStream> {
    match &field.type_info {
        TypeInformation::Primitive => generate_primitive_match_arms(field),

        TypeInformation::TypeChoice(attributes) => {
            generate_typechoice_match_arms(field, attributes)
        }

        TypeInformation::Complex => {
            vec![generate_complex_match_arm(field)]
        }
    }
}

fn generate_primitive_match_arms(field: &FieldInformation) -> Vec<proc_macro2::TokenStream> {
    let mut arms = Vec::new();

    let value_ident = value_ident(&field.ident);
    let ext_ident = extension_ident(&field.ident);

    let value_field_name = &field.field_name;
    let ext_field_name = format!("_{}", field.field_name);

    arms.push(quote! {
        #value_field_name => {
            if #value_ident.is_some() {
                return Err(
                    serde::de::Error::duplicate_field(
                        #value_field_name
                    )
                );
            }

            #value_ident = Some(map.next_value()?);
        }
    });

    if field.is_vector {
        arms.push(quote! {
            #ext_field_name => {
                if #ext_ident.is_some() {
                    return Err(
                        serde::de::Error::duplicate_field(
                            #ext_field_name
                        )
                    );
                }

                #ext_ident =
                    Some(map.next_value::<Vec<Option<Element>>>()?);
            }
        });
    } else {
        arms.push(quote! {
            #ext_field_name => {
                if #ext_ident.is_some() {
                    return Err(
                        serde::de::Error::duplicate_field(
                            #ext_field_name
                        )
                    );
                }

                #ext_ident =
                    Some(map.next_value::<Element>()?);
            }
        });
    }

    arms
}

fn generate_typechoice_match_arms(
    field: &FieldInformation,
    type_choice_attributes: &TypeChoiceAttribute,
) -> Vec<proc_macro2::TokenStream> {
    let mut arms = Vec::new();

    let value_ident = value_ident(&field.ident);
    let ext_ident = extension_ident(&field.ident);

    let primitives = &type_choice_attributes.primitive_variants;
    let complex_variants = &type_choice_attributes.complex_variants;

    for primitive_variant_fieldname in primitives {
        let value_setter =
            typechoice_value_setter(field, &value_ident, primitive_variant_fieldname);

        let found_ident = typechoice_variant_found_ident(&field.ident);

        let primitive_ext_field_name = format!("_{primitive_variant_fieldname}");

        arms.push(quote! {
            #primitive_variant_fieldname => {
                if #value_ident.is_some() {
                    return Err(
                        serde::de::Error::duplicate_field(
                            #primitive_variant_fieldname
                        )
                    );
                }

                #value_setter
                #found_ident = Some(#primitive_variant_fieldname);
            }
        });

        arms.push(quote! {
            #primitive_ext_field_name => {
                #ext_ident =
                    Some(map.next_value::<Element>()?);

                #found_ident =
                    Some(#primitive_variant_fieldname);
            }
        });
    }

    for complex_variant_fieldname in complex_variants {
        let value_setter = typechoice_value_setter(field, &value_ident, complex_variant_fieldname);

        arms.push(quote! {
            #complex_variant_fieldname => {
                #value_setter
            }
        });
    }

    arms
}

fn generate_complex_match_arm(field: &FieldInformation) -> proc_macro2::TokenStream {
    let value_ident = value_ident(&field.ident);
    let value_field_name = &field.field_name;

    quote! {
        #value_field_name => {
            if #value_ident.is_some() {
                return Err(
                    serde::de::Error::duplicate_field(
                        #value_field_name
                    )
                );
            }

            #value_ident = Some(map.next_value()?);
        }
    }
}
