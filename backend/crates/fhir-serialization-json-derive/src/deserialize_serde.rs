use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{Data, DeriveInput, Variant};

use crate::{
    DeserializeComplexType,
    utilities::{
        get_attribute_value, get_field_name, get_field_type, get_type_choice_attribute,
        is_attribute_present,
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
    deserialize_complex_type: DeserializeComplexType,
) -> TokenStream {
    let name = input.ident;
    match input.data {
        Data::Struct(data) => {
            let visitor_name = format_ident!("{}Visitor", name);
            let name_str = name.to_string();

            // Declare all fields for the given struct.
            // Make all fields optional at this stage to allow for partial construction during deserialization,
            // we'll validate required fields at the end.

            let field_declarations = data.fields.iter().flat_map(|field| {
                let field_ident = field.ident.as_ref().unwrap();
                let field_ty = field.ty.clone();
                let value_ident = format_ident!("__{}_value", field_ident);

                if is_attribute_present(&field.attrs, "primitive") {
                    let ext_ident = format_ident!("__{}_ext", field_ident);
                    let target_ty = get_optional_inner_type(&field_ty).unwrap_or(field_ty.clone());
                    let is_vec = get_vec_inner_type(&target_ty).is_some();
                    if is_vec {
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
                } else if is_attribute_present(&field.attrs, "type_choice_variants") {
                    let pending_ident = format_ident!("__{}_pending_ext", field_ident);
                    vec![
                        quote! { let mut #value_ident: Option<#field_ty> = None; },
                        quote! { let mut #pending_ident: Vec<(String, Element)> = Vec::new(); },
                    ]
                } else {
                    vec![quote! { let mut #value_ident: Option<#field_ty> = None; }]
                }
            });

            let mut key_match_arms = Vec::new();
            if deserialize_complex_type == DeserializeComplexType::Resource {
                key_match_arms.push(quote! {
                    "resourceType" => {
                        let resource_type: String = map.next_value()?;
                        if resource_type != #name_str {
                            return Err(serde::de::Error::custom(format!(
                                "Invalid resourceType for {}: {}",
                                #name_str,
                                resource_type
                            )));
                        }
                        __seen_resource_type = true;
                    }
                });
            }

            for field in &data.fields {
                let field_ident = field.ident.as_ref().unwrap();
                let field_ty = field.ty.clone();
                let field_name = get_field_name(field);
                let ext_name = format!("_{}", field_name);
                let value_ident = format_ident!("__{}_value", field_ident);

                if is_attribute_present(&field.attrs, "primitive") {
                    let ext_ident = format_ident!("__{}_ext", field_ident);
                    let target_ty = get_optional_inner_type(&field_ty).unwrap_or(field_ty.clone());
                    let is_vec = get_vec_inner_type(&target_ty).is_some();
                    key_match_arms.push(quote! {
                        #field_name => {
                            if #value_ident.is_some() {
                                return Err(serde::de::Error::duplicate_field(#field_name));
                            }
                            #value_ident = Some(map.next_value()?);
                        }
                    });
                    if is_vec {
                        key_match_arms.push(quote! {
                            #ext_name => {
                                if #ext_ident.is_some() {
                                    return Err(serde::de::Error::duplicate_field(#ext_name));
                                }
                                #ext_ident = Some(map.next_value::<Vec<Option<Element>>>()?);
                            }
                        });
                    } else {
                        key_match_arms.push(quote! {
                            #ext_name => {
                                if #ext_ident.is_some() {
                                    return Err(serde::de::Error::duplicate_field(#ext_name));
                                }
                                #ext_ident = Some(map.next_value::<Element>()?);
                            }
                        });
                    }
                    continue;
                }

                if is_attribute_present(&field.attrs, "type_choice_variants") {
                    let pending_ident = format_ident!("__{}_pending_ext", field_ident);
                    let type_choice_attr = get_type_choice_attribute(&field.attrs)
                        .expect("type_choice_variants is required on type choice fields");
                    for key in type_choice_attr.all() {
                        key_match_arms.push(quote! {
                            #key => {
                                if key.starts_with('_') {
                                    let element = map.next_value::<Element>()?;
                                    #pending_ident.push((key.clone(), element));
                                } else {
                                    if #value_ident.is_some() {
                                        return Err(serde::de::Error::custom(format!(
                                            "Duplicate typechoice assignment for field '{}'",
                                            #field_name
                                        )));
                                    }
                                    #value_ident = #field_ty::try_deserialize_from_key(key.as_str(), &mut map)?;
                                    if #value_ident.is_none() {
                                        return Err(serde::de::Error::custom(format!(
                                            "Invalid typechoice variant '{}' for field '{}'",
                                            key,
                                            #field_name
                                        )));
                                    }
                                }
                            }
                        });
                    }
                    continue;
                }

                key_match_arms.push(quote! {
                    #field_name => {
                        if #value_ident.is_some() {
                            return Err(serde::de::Error::duplicate_field(#field_name));
                        }
                        #value_ident = Some(map.next_value()?);
                    }
                });
            }

            let primitive_finalize = data.fields.iter().filter_map(|field| {
                if !is_attribute_present(&field.attrs, "primitive") {
                    return None;
                }
                let field_ident = field.ident.as_ref().unwrap();
                let field_ty = field.ty.clone();
                let value_ident = format_ident!("__{}_value", field_ident);
                let ext_ident = format_ident!("__{}_ext", field_ident);

                let optional_inner = get_optional_inner_type(&field_ty);
                let target_ty = optional_inner.clone().unwrap_or(field_ty.clone());
                let vec_inner = get_vec_inner_type(&target_ty);
                let is_optional = optional_inner.is_some();

                if let Some(vec_item_ty) = vec_inner {
                    let merge_code = if is_optional {
                        quote! {
                            if let Some(elements) = #ext_ident.take() {
                                match #value_ident.as_mut() {
                                    Some(existing_opt) => {
                                        if existing_opt.is_none() {
                                            *existing_opt = Some(Vec::new());
                                        }
                                        let existing_vec = existing_opt.as_mut().expect("initialized above");
                                        if existing_vec.len() < elements.len() {
                                            existing_vec.resize_with(elements.len(), Default::default);
                                        }
                                        for (i, element_opt) in elements.into_iter().enumerate() {
                                            if let Some(element) = element_opt {
                                                existing_vec[i].merge_element(element);
                                            }
                                        }
                                    }
                                    None => {
                                        let mut created: Vec<#vec_item_ty> = Vec::new();
                                        created.resize_with(elements.len(), Default::default);
                                        for (i, element_opt) in elements.into_iter().enumerate() {
                                            if let Some(element) = element_opt {
                                                created[i].merge_element(element);
                                            }
                                        }
                                        #value_ident = Some(Some(created));
                                    }
                                }
                            }
                        }
                    } else {
                        quote! {
                            if let Some(elements) = #ext_ident.take() {
                                match #value_ident.as_mut() {
                                    Some(existing_vec) => {
                                        if existing_vec.len() < elements.len() {
                                            existing_vec.resize_with(elements.len(), Default::default);
                                        }
                                        for (i, element_opt) in elements.into_iter().enumerate() {
                                            if let Some(element) = element_opt {
                                                existing_vec[i].merge_element(element);
                                            }
                                        }
                                    }
                                    None => {
                                        let mut created: Vec<#vec_item_ty> = Vec::new();
                                        created.resize_with(elements.len(), Default::default);
                                        for (i, element_opt) in elements.into_iter().enumerate() {
                                            if let Some(element) = element_opt {
                                                created[i].merge_element(element);
                                            }
                                        }
                                        #value_ident = Some(created);
                                    }
                                }
                            }
                        }
                    };

                    Some(merge_code)
                } else {
                    let merge_code = if is_optional {
                        let inner_ty = optional_inner.expect("checked above");
                        quote! {
                            if let Some(element) = #ext_ident.take() {
                                match #value_ident.as_mut() {
                                    Some(existing_opt) => {
                                        if let Some(existing) = existing_opt.as_mut() {
                                            existing.merge_element(element);
                                        } else {
                                            let mut created: #inner_ty = Default::default();
                                            created.merge_element(element);
                                            *existing_opt = Some(created);
                                        }
                                    }
                                    None => {
                                        let mut created: #inner_ty = Default::default();
                                        created.merge_element(element);
                                        #value_ident = Some(Some(created));
                                    }
                                }
                            }
                        }
                    } else {
                        quote! {
                            if let Some(element) = #ext_ident.take() {
                                match #value_ident.as_mut() {
                                    Some(existing) => {
                                        existing.merge_element(element);
                                    }
                                    None => {
                                        let mut created: #field_ty = Default::default();
                                        created.merge_element(element);
                                        #value_ident = Some(created);
                                    }
                                }
                            }
                        }
                    };

                    Some(merge_code)
                }
            });

            let typechoice_finalize = data.fields.iter().filter_map(|field| {
                if !is_attribute_present(&field.attrs, "type_choice_variants") {
                    return None;
                }
                let field_ident = field.ident.as_ref().unwrap();
                let field_name = get_field_name(field);
                let value_ident = format_ident!("__{}_value", field_ident);
                let pending_ident = format_ident!("__{}_pending_ext", field_ident);

                Some(quote! {
                    if #value_ident.is_none() && !#pending_ident.is_empty() {
                        return Err(serde::de::Error::custom(format!(
                            "Found typechoice primitive extension without value for field '{}'",
                            #field_name
                        )));
                    }
                    if let Some(choice) = #value_ident.as_mut() {
                        for (k, element) in #pending_ident.drain(..) {
                            choice.merge_element(k.as_str(), element);
                        }
                    }
                })
            });

            let bind_fields = data.fields.iter().map(|field| {
                let field_ident = field.ident.as_ref().unwrap();
                let field_name = get_field_name(field);
                let value_ident = format_ident!("__{}_value", field_ident);
                if is_optional_field(field) {
                    quote! { let #field_ident = #value_ident.and_then(|v| v); }
                } else {
                    quote! {
                        let #field_ident = #value_ident
                            .ok_or_else(|| serde::de::Error::missing_field(#field_name))?;
                    }
                }
            });

            let field_names = data.fields.iter().map(|f| f.ident.as_ref().unwrap());

            let seen_resource_decl = if deserialize_complex_type == DeserializeComplexType::Resource
            {
                quote! { let mut __seen_resource_type = false; }
            } else {
                quote! {}
            };

            let required_resource_check =
                if deserialize_complex_type == DeserializeComplexType::Resource {
                    quote! {
                        if !__seen_resource_type {
                            return Err(serde::de::Error::missing_field("resourceType"));
                        }
                    }
                } else {
                    quote! {}
                };

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

                                #(#primitive_finalize)*
                                #(#typechoice_finalize)*
                                #required_resource_check

                                #(#bind_fields)*

                                Ok(#name {
                                    #(#field_names),*
                                })
                            }
                        }

                        d.deserialize_map(#visitor_name)
                    }
                }
            };

            deserialize_impl.into()
        }
        _ => panic!("Only structs can be deserialized for complex deserializer."),
    }
}
