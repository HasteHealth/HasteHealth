use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    path::Path,
};

use crate::documentation::{format_documentation, generate_doc_attributes};
use crate::{
    traversal,
    utilities::{
        RUST_KEYWORDS, conditionals,
        conversion::fhir_type_to_rust_type,
        extract,
        generate::{self, field_typename},
        load,
    },
};
use haste_fhir_model::r4::generated::{
    resources::StructureDefinition,
    terminology::{StructureDefinitionKind, TypeDerivationRule},
    types::ElementDefinition,
};
use indexmap::IndexMap;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use walkdir::WalkDir;

type NestedTypes = IndexMap<String, TokenStream>;

fn type_ident(name: &str) -> Ident {
    format_ident!("{name}")
}

fn rust_resource_ident(name: &str) -> Ident {
    format_ident!("{}", generate::capitalize(name))
}

fn field_ident(field_name: &str) -> Ident {
    if RUST_KEYWORDS.contains(&field_name) {
        format_ident!("{field_name}_")
    } else {
        format_ident!("{field_name}")
    }
}

fn min_max_attribute(element: &ElementDefinition) -> TokenStream {
    let (min, max) = extract::cardinality(element);

    match max {
        extract::Max::Unlimited => {
            if min > 0 {
                quote! { #[cardinality(min = #min)] }
            } else {
                quote! {}
            }
        }

        extract::Max::Fixed(1) => {
            quote! {}
        }

        extract::Max::Fixed(max) => {
            if min > 0 {
                quote! {
                    #[cardinality(min = #min, max = #max)]
                }
            } else {
                quote! {
                    #[cardinality(max = #max)]
                }
            }
        }
    }
}

/// Elements directly under the resource root (e.g. `Patient.status`) count as
/// top-level; deeper elements (e.g. `Patient.name.given`) are part of a complex
/// element and are kept or dropped as a whole along with it.
fn get_top_level_elements(sd: &StructureDefinition) -> Vec<&ElementDefinition> {
    let Some(resource_type) = sd.type_.value.as_ref() else {
        return vec![];
    };

    let prefix = format!("{resource_type}.");

    sd.snapshot
        .as_ref()
        .map(|snapshot| {
            snapshot
                .element
                .iter()
                .filter(|element| {
                    element
                        .path
                        .value
                        .as_ref()
                        .and_then(|path| path.strip_prefix(&prefix))
                        .is_some_and(|rest| !rest.contains('.'))
                })
                .collect()
        })
        .unwrap_or_default()
}

fn get_top_level_required_fields(sd: &StructureDefinition) -> Vec<String> {
    get_top_level_elements(sd)
        .into_iter()
        .filter(|element| {
            element
                .min
                .as_ref()
                .is_some_and(|min| min.value.unwrap_or(0) > 0)
        })
        .map(|element| extract::field_name(&extract::path(element)))
        .collect()
}

fn get_top_level_fields(sd: &StructureDefinition) -> Vec<String> {
    get_top_level_elements(sd)
        .into_iter()
        .map(|element| extract::field_name(&extract::path(element)))
        .collect()
}

/// `id`, `meta`, and any element with `min >= 1` are always returned regardless
/// of what a client asked to subset down to, per the `_elements`/`_summary` rules.
///
/// `id`/`meta` are forced in explicitly since they're min=0 in the base
/// resource but still mandatory under those rules.
fn get_mandatory_fields(sd: &StructureDefinition, all_fields: &[String]) -> Vec<String> {
    let mut mandatory = Vec::with_capacity(all_fields.len());

    mandatory.push("id".to_string());
    mandatory.push("meta".to_string());

    mandatory.extend(get_top_level_required_fields(sd));

    mandatory.sort_unstable();
    mandatory.dedup();

    mandatory.retain(|field| all_fields.contains(field));

    mandatory
}

fn generate_filter_field_helper() -> TokenStream {
    quote! {
        #[inline]
        fn filter_field<T: Default>(
            fields: &[&str],
            name: &str,
            value: T,
        ) -> T {
            if fields.contains(&name) {
                value
            } else {
                T::default()
            }
        }
    }
}

/// Generates the `impl <Resource>::filter` used to implement `_elements`.
///
/// A fresh `Self::default()` is not needed: optional fields are replaced with
/// their default values directly while mandatory fields are left untouched.
///
/// `_elements` only addresses top-level elements, so this never recurses into
/// complex-type substructure.
fn generate_filter_impl(sd: &StructureDefinition, struct_ident: &Ident) -> TokenStream {
    const FILTER_FIELDS_PER_HELPER: usize = 25;

    let resource_type = sd.type_.value.clone().unwrap_or_default();

    let all_fields = get_top_level_fields(sd);
    let mandatory_fields = get_mandatory_fields(sd, &all_fields);

    // Own the filtered list so the generated iterators don't borrow
    // from a temporary Vec.
    let optional_fields: Vec<String> = all_fields
        .iter()
        .filter(|field| !mandatory_fields.contains(field))
        .cloned()
        .collect();

    let filter_fields = quote! {
        const FILTER_FIELDS: &[&str] = &[
            #(#all_fields),*
        ];
    };

    let filter_helpers = optional_fields
        .chunks(FILTER_FIELDS_PER_HELPER)
        .enumerate()
        .map(|(index, fields)| {
            let helper_name = format_ident!("filter_fields_{index}");

            let field_initializers = fields.iter().map(|field| {
                let ident = field_ident(field);

                quote! {
                    value.#ident = filter_field(
                        fields,
                        #field,
                        value.#ident,
                    );
                }
            });

            quote! {
                #[inline]
                fn #helper_name(
                    mut value: Self,
                    fields: &[&str],
                ) -> Self {
                    #(#field_initializers)*
                    value
                }
            }
        });

    let helper_calls = optional_fields
        .chunks(FILTER_FIELDS_PER_HELPER)
        .enumerate()
        .map(|(index, _)| {
            let helper_name = format_ident!("filter_fields_{index}");

            quote! {
                let value = Self::#helper_name(value, fields);
            }
        });

    quote! {
        impl #struct_ident {
            #filter_fields

            #[inline]
            fn is_filter_field(field: &str) -> bool {
                Self::FILTER_FIELDS.contains(&field)
            }

            #(#filter_helpers)*

            /// Filters this resource to the requested top-level fields.
            ///
            /// # Errors
            ///
            /// Returns `FilterFieldsError::UnknownField` if `fields` contains
            /// a field that is not supported for this resource.
            pub fn filter(
                self,
                fields: &[&str],
            ) -> Result<Self, FilterFieldsError> {
                for field in fields {
                    if !Self::is_filter_field(field) {
                        return Err(FilterFieldsError::UnknownField(
                            field.to_string(),
                            #resource_type.to_string(),
                        ));
                    }
                }

                let value = self;

                #(#helper_calls)*

                Ok(value)
            }
        }
    }
}

fn wrap_if_vec(
    element: &ElementDefinition,
    field_value: &TokenStream,
    should_box: bool,
) -> TokenStream {
    match extract::cardinality(element).1 {
        extract::Max::Fixed(1) => {
            if should_box {
                quote! {
                    Box<#field_value>
                }
            } else {
                field_value.clone()
            }
        }
        extract::Max::Unlimited | extract::Max::Fixed(_) => {
            quote! {
                Vec<#field_value>
            }
        }
    }
}

fn wrap_cardinality_and_optionality(
    element: &ElementDefinition,
    field_value: &TokenStream,
    should_box: bool,
) -> TokenStream {
    let field_value = wrap_if_vec(element, field_value, should_box);

    if extract::cardinality(element).0 == 0 {
        quote! {
            Option<#field_value>
        }
    } else {
        field_value
    }
}

fn get_reference_target_attribute(element: &ElementDefinition) -> TokenStream {
    let Some(type_vec) = element.type_.as_ref() else {
        return quote! {};
    };

    let Some(reference_type) = type_vec
        .iter()
        .find(|ty| ty.code.value.as_deref() == Some("Reference"))
    else {
        return quote! {};
    };

    let Some(targets) = reference_type.targetProfile.as_ref() else {
        return quote! {};
    };

    let profiles = targets
        .iter()
        .filter_map(
            |target: &haste_fhir_model::r4::generated::types::FHIRCanonical| {
                target.value.as_deref()
            },
        )
        .filter_map(|target| target.rsplit('/').next());

    quote! {
        #[reference(targets = [#(#profiles),*])]
    }
}

fn get_struct_key_value(
    element: &ElementDefinition,
    field_value_type_name: &(TokenStream, bool),
) -> TokenStream {
    let description = extract::element_description(element);
    let formatted_description = format_documentation(&description);
    let doc_attributes = generate_doc_attributes(&formatted_description);

    let field_name = extract::field_name(&extract::path(element));
    let field_name_ident = field_ident(&field_name);

    let reflect_attribute = if RUST_KEYWORDS.contains(&field_name.as_str()) {
        quote! {
            #[rename_field = #field_name]
        }
    } else {
        quote! {}
    };

    let type_choice_variants = if conditionals::is_typechoice(element) {
        let variants = generate::create_type_choice_variants(element);
        let primitive_variants = generate::create_type_choice_primitive_variants(element);

        let complex_variants = variants
            .iter()
            .filter(|variant| !primitive_variants.contains(variant));

        quote! {
            #[type_choice_variants(
                complex = [#(#complex_variants),*],
                primitive = [#(#primitive_variants),*]
            )]
        }
    } else {
        quote! {}
    };

    let primitive_attribute = if conditionals::is_primitive_element(element) {
        quote! {
            #[primitive]
        }
    } else {
        quote! {}
    };

    let target_types = if conditionals::is_typechoice(element) {
        quote! {}
    } else {
        get_reference_target_attribute(element)
    };

    let cardinality_attribute = min_max_attribute(element);

    let field_type = wrap_cardinality_and_optionality(
        element,
        &field_value_type_name.0,
        field_value_type_name.1,
    );

    quote! {
        #type_choice_variants
        #reflect_attribute
        #primitive_attribute
        #cardinality_attribute
        #target_types
        #doc_attributes
        pub #field_name_ident: #field_type
    }
}

fn resolve_content_reference<'a>(
    sd: &'a StructureDefinition,
    element: &ElementDefinition,
) -> &'a ElementDefinition {
    let content_reference_id = element
        .contentReference
        .as_ref()
        .unwrap()
        .value
        .as_ref()
        .unwrap()[1..]
        .to_string();

    let content_reference_element = sd
        .snapshot
        .as_ref()
        .expect("StructureDefinition has no snapshot")
        .element
        .iter()
        .filter(|element| element.id == Some(content_reference_id.clone()))
        .collect::<Vec<_>>();

    assert_eq!(
        content_reference_element.len(),
        1,
        "Content reference element not found {content_reference_id}",
    );

    content_reference_element[0]
}

fn create_type_choice(
    sd: &StructureDefinition,
    element: &ElementDefinition,
    inlined_terminology: &HashMap<String, String>,
) -> TokenStream {
    let field_name = extract::field_name(&extract::path(element));
    let type_name = type_ident(&generate::type_choice_name(sd, element));
    let types = extract::field_types(element);

    let enum_variants = types.iter().map(|fhir_type| {
        let enum_name = rust_resource_ident(fhir_type);

        let (rust_type, should_box) =
            fhir_type_to_rust_type(element, fhir_type, inlined_terminology);

        let rust_type = wrap_if_vec(element, &rust_type, should_box);

        let target_types = if *fhir_type == "Reference" {
            get_reference_target_attribute(element)
        } else {
            quote! {}
        };

        let primitive_attribute = if conditionals::is_primitive_type(fhir_type) {
            quote! {
                #[primitive]
            }
        } else {
            quote! {}
        };

        quote! {
            #primitive_attribute
            #target_types
            #enum_name(#rust_type)
        }
    });

    let default_enum = rust_resource_ident(types[0]);

    let default_impl = if conditionals::should_be_boxed(types[0]) {
        quote! {
            impl Default for #type_name {
                fn default() -> Self {
                    #type_name::#default_enum(Box::default())
                }
            }
        }
    } else {
        quote! {
            impl Default for #type_name {
                fn default() -> Self {
                    #type_name::#default_enum(Default::default())
                }
            }
        }
    };

    quote! {
        #[derive(
            Clone,
            Reflect,
            Debug,
            haste_fhir_serialization_json::derive::FHIRSerdeSerialize,
            haste_fhir_serialization_json::derive::FHIRSerdeDeserialize
        )]
        #[fhir_serialize_type = "typechoice"]
        #[type_choice_field_name = #field_name]
        pub enum #type_name {
            #(#enum_variants),*
        }

        #default_impl
    }
}

fn process_leaf(
    sd: &StructureDefinition,
    element: &ElementDefinition,
    types: &mut NestedTypes,
    inlined_terminology: &HashMap<String, String>,
) -> TokenStream {
    if element.contentReference.is_some() {
        let content_reference_element = resolve_content_reference(sd, element);

        let field_type_name = field_typename(sd, content_reference_element, inlined_terminology);

        return get_struct_key_value(element, &field_type_name);
    }

    if conditionals::is_typechoice(element) {
        let (type_choice_name_ident, should_box) = field_typename(sd, element, inlined_terminology);

        types
            .entry(type_choice_name_ident.to_string())
            .or_insert_with(|| create_type_choice(sd, element, inlined_terminology));

        return get_struct_key_value(element, &(quote! { #type_choice_name_ident }, should_box));
    }

    let fhir_type = extract::field_types(element)[0];

    let rust_type = fhir_type_to_rust_type(element, fhir_type, inlined_terminology);

    get_struct_key_value(element, &rust_type)
}

fn from_rust_type_to_fhir_primitive(
    sd_ident: &Ident,
    sd: &StructureDefinition,
    inlined_terminology: &HashMap<String, String>,
) -> TokenStream {
    let value_element = sd
        .snapshot
        .as_ref()
        .map(|snapshot| &snapshot.element)
        .and_then(|elements| {
            elements.iter().find(|element| {
                #[allow(clippy::case_sensitive_file_extension_comparisons)]
                element
                    .path
                    .value
                    .as_ref()
                    .is_some_and(|path| path.ends_with(".value"))
            })
        });

    let Some(value_element) = value_element else {
        return quote! {};
    };

    let field_types = extract::field_types(value_element);

    let Some(fhir_type) = field_types.first() else {
        return quote! {};
    };

    let (value_type, _should_box) =
        fhir_type_to_rust_type(value_element, fhir_type, inlined_terminology);

    let required = value_element
        .min
        .as_ref()
        .and_then(|min| min.value)
        .unwrap_or(0)
        > 0;

    if required {
        quote! {
            impl From<#value_type> for #sd_ident {
                fn from(value: #value_type) -> Self {
                    Self {
                        value,
                        ..Default::default()
                    }
                }
            }
        }
    } else {
        quote! {
            impl From<#value_type> for #sd_ident {
                fn from(value: #value_type) -> Self {
                    Self {
                        value: Some(value),
                        ..Default::default()
                    }
                }
            }
        }
    }
}

fn create_complex_struct(
    sd: &StructureDefinition,
    element: &ElementDefinition,
    children: &[TokenStream],
    types: &mut NestedTypes,
    rust_type_name_to_fhir_type: &mut HashMap<String, String>,
    inlined_terminology: &HashMap<String, String>,
) -> TokenStream {
    let struct_name = generate::struct_name(sd, element);
    let fhir_type = extract::fhir_type(sd, element);
    let struct_ident = type_ident(&struct_name);

    rust_type_name_to_fhir_type.insert(struct_name.clone(), fhir_type.clone());

    let description = extract::element_description(element);
    let formatted_description = format_documentation(&description);
    let doc_attributes = generate_doc_attributes(&formatted_description);

    let (derive, additional_impls) = if conditionals::is_root(sd, element)
        && conditionals::is_primitive_sd(sd)
    {
        let from_impl = from_rust_type_to_fhir_primitive(&struct_ident, sd, inlined_terminology);

        let impls = quote! {
            impl #struct_ident {
                #[inline]
                pub fn extension_mut(
                    &mut self,
                ) -> &mut Option<Vec<Extension>> {
                    &mut self.extension
                }

                #[inline]
                pub fn id_mut(
                    &mut self,
                ) -> &mut Option<String> {
                    &mut self.id
                }
            }

            #from_impl
        };

        let derive = quote! {
            #[derive(
                Clone,
                Reflect,
                Debug,
                Default,
                haste_fhir_serialization_json::derive::FHIRSerdeSerialize,
                haste_fhir_serialization_json::derive::FHIRSerdeDeserialize
            )]
            #[fhir_type = #fhir_type]
            #[fhir_serialize_type = "primitive"]
        };

        (derive, impls)
    } else if conditionals::is_root(sd, element) && conditionals::is_resource_sd(sd) {
        let resource_type = sd
            .type_
            .value
            .as_ref()
            .expect("resource StructureDefinition has no type");

        let resource_type_attribute = if *resource_type == struct_name {
            quote! {}
        } else {
            quote! {
                #[fhir_resource_type = #resource_type]
            }
        };

        let filter_impl = generate_filter_impl(sd, &struct_ident);

        let derive = quote! {
            #[derive(
                Clone,
                Reflect,
                Debug,
                Default,
                haste_fhir_serialization_json::derive::FHIRSerdeSerialize,
                haste_fhir_serialization_json::derive::FHIRSerdeDeserialize
            )]
            #[fhir_type = #fhir_type]
            #resource_type_attribute
            #[fhir_serialize_type = "resource"]
        };

        (derive, filter_impl)
    } else {
        let derive = quote! {
            #[derive(
                Clone,
                Reflect,
                Debug,
                Default,
                haste_fhir_serialization_json::derive::FHIRSerdeSerialize,
                haste_fhir_serialization_json::derive::FHIRSerdeDeserialize
            )]
            #[fhir_type = #fhir_type]
            #[fhir_serialize_type = "complex"]
        };

        (derive, quote! {})
    };

    let generated = quote! {
        #derive
        #doc_attributes
        pub struct #struct_ident {
            #(#children),*
        }

        #additional_impls
    };

    types.insert(struct_name, generated);

    get_struct_key_value(element, &(quote! { #struct_ident }, false))
}

fn generate_from_structure_definition(
    sd: &StructureDefinition,
    inlined_terminology: &HashMap<String, String>,
    rust_type_name_to_fhir_type: &mut HashMap<String, String>,
) -> Result<TokenStream, String> {
    let mut nested_types = NestedTypes::new();

    let mut visitor =
        |element: &ElementDefinition, children: Vec<TokenStream>, _index: usize| -> TokenStream {
            if children.is_empty() {
                process_leaf(sd, element, &mut nested_types, inlined_terminology)
            } else {
                create_complex_struct(
                    sd,
                    element,
                    &children,
                    &mut nested_types,
                    rust_type_name_to_fhir_type,
                    inlined_terminology,
                )
            }
        };

    traversal::traversal(sd, &mut visitor)?;

    let types_generated = nested_types.values();

    Ok(quote! {
        #(#types_generated)*
    })
}

struct GeneratedTypes {
    resources: Vec<TokenStream>,
    types: Vec<TokenStream>,
    resource_types: Vec<ResourceTypeInfo>,
    rust_type_name_to_fhir_type: HashMap<String, String>,
}

struct ResourceTypeInfo {
    resource_type: String,
    rust_type_name: String,
}

fn should_generate_structure_definition(sd: &StructureDefinition) -> bool {
    let is_specialization = sd.derivation.as_ref() == Some(&TypeDerivationRule::specialization());

    if !is_specialization && sd.derivation.is_some() {
        return false;
    }

    if sd.kind == StructureDefinitionKind::resource() {
        !extract::is_abstract(sd)
    } else {
        true
    }
}

fn generate_fhir_types_from_file(
    file_path: &Path,
    level: Option<&'static str>,
    inlined_terminology: &HashMap<String, String>,
) -> Result<GeneratedTypes, String> {
    let resource = load::load_from_file(file_path)?;

    let structure_definitions = load::get_structure_definitions(&resource, level)
        .map_err(|e| format!("Failed to get structure definitions: {e}"))?;

    let mut resources = Vec::new();
    let mut types = Vec::new();
    let mut resource_types = Vec::new();

    let mut rust_type_name_to_fhir_type = HashMap::new();

    for sd in structure_definitions
        .iter()
        .filter(|sd| should_generate_structure_definition(sd))
    {
        if conditionals::is_resource_sd(sd) {
            resource_types.push(ResourceTypeInfo {
                resource_type: sd
                    .type_
                    .value
                    .as_ref()
                    .expect("resource has no type")
                    .clone(),

                rust_type_name: sd.id.as_ref().expect("resource has no id").clone(),
            });

            resources.push(generate_from_structure_definition(
                sd,
                inlined_terminology,
                &mut HashMap::new(),
            )?);
        } else {
            types.push(generate_from_structure_definition(
                sd,
                inlined_terminology,
                &mut rust_type_name_to_fhir_type,
            )?);
        }
    }

    Ok(GeneratedTypes {
        resources,
        types,
        resource_types,
        rust_type_name_to_fhir_type,
    })
}

fn generate_resource_type_letter_helpers(resource_types: &[ResourceTypeInfo]) -> TokenStream {
    let mut groups: BTreeMap<u8, Vec<&ResourceTypeInfo>> = BTreeMap::new();

    for resource_type_info in resource_types {
        let Some(&first_byte) = resource_type_info.resource_type.as_bytes().first() else {
            continue;
        };

        groups
            .entry(first_byte)
            .or_default()
            .push(resource_type_info);
    }

    groups
        .into_iter()
        .map(|(first_byte, resources)| {
            let letter = first_byte as char;

            let helper_name = format_ident!("resource_type_from_{}", letter.to_ascii_lowercase());

            let match_arms = resources.iter().map(|resource_type_info| {
                let rust_type_name = &resource_type_info.rust_type_name;

                let resource_type_name = &resource_type_info.resource_type;

                let variant = rust_resource_ident(rust_type_name);

                if rust_type_name == resource_type_name {
                    quote! {
                        #resource_type_name =>
                            Ok(ResourceType::#variant)
                    }
                } else {
                    quote! {
                        #rust_type_name |
                        #resource_type_name =>
                            Ok(ResourceType::#variant)
                    }
                }
            });

            quote! {
                #[inline]
                fn #helper_name(
                    s: &str,
                ) -> Result<ResourceType, ResourceTypeError> {
                    match s {
                        #(#match_arms),*,

                        _ => Err(
                            ResourceTypeError::Invalid(
                                s.to_string(),
                            )
                        ),
                    }
                }
            }
        })
        .collect()
}

fn generate_resource_type(resource_types: &[ResourceTypeInfo]) -> TokenStream {
    let definition = generate_resource_type_definition(resource_types);
    let deserializers_and_filters =
        generate_resource_type_deserializers_and_filters(resource_types);
    let resource_type_impl = generate_resource_type_impl(resource_types);

    quote! {
        #definition
        #deserializers_and_filters
        #resource_type_impl
    }
}

fn generate_resource_type_definition(resource_types: &[ResourceTypeInfo]) -> TokenStream {
    let enum_variants = generate_resource_type_enum_variants(resource_types);
    let resource_type_names = generate_resource_type_names(resource_types);
    let deserializer_table = generate_resource_type_deserializer_table(resource_types);
    let filter_table = generate_resource_type_filter_table(resource_types);

    quote! {
        #[derive(Error, Debug)]
        pub enum ResourceTypeError {
            #[error("Invalid resource type: {0}")]
            Invalid(String),
        }

        #[derive(
            Debug,
            Clone,
            Copy,
            PartialEq,
            Eq,
            Hash,
            serde::Deserialize,
            serde::Serialize,
            PartialOrd,
            Ord
        )]
        #[repr(u16)]
        pub enum ResourceType {
            #(#enum_variants),*
        }

        const RESOURCE_TYPE_NAMES: &[&str] = &[
            #(#resource_type_names),*
        ];

        type ResourceDeserializer = fn(
            &[u8],
        ) -> Result<
            Resource,
            haste_fhir_serialization_json::errors::DeserializeError,
        >;

        const RESOURCE_DESERIALIZERS: &[ResourceDeserializer] = &[
            #(#deserializer_table),*
        ];

        type ResourceFilter =
            fn(
                Resource,
                &[&str],
            ) -> Result<Resource, FilterFieldsError>;

        const RESOURCE_FILTERS: &[ResourceFilter] = &[
            #(#filter_table),*
        ];
    }
}

fn generate_resource_type_deserializers_and_filters(
    resource_types: &[ResourceTypeInfo],
) -> TokenStream {
    let deserializer_functions = generate_resource_type_deserializers(resource_types);
    let filter_functions = generate_resource_type_filters(resource_types);

    quote! {
        #(#deserializer_functions)*
        #(#filter_functions)*
    }
}

fn generate_resource_type_impl(resource_types: &[ResourceTypeInfo]) -> TokenStream {
    let letter_helpers = generate_resource_type_letter_helpers(resource_types);
    let first_byte_dispatch = generate_resource_type_first_byte_dispatch(resource_types);

    quote! {
        impl ResourceType {
            #letter_helpers

            /// Deserializes a resource using the deserializer associated with
            /// this resource type.
            ///
            /// # Errors
            ///
            /// Returns
            /// [`haste_fhir_serialization_json::errors::DeserializeError`] if
            /// the input data cannot be deserialized into the
            /// expected resource type.
            #[inline]
            pub fn deserialize<D: AsRef<[u8]>>(
                &self,
                data: D,
            ) -> Result<
                Resource,
                haste_fhir_serialization_json::errors::DeserializeError,
            > {
                RESOURCE_DESERIALIZERS[*self as usize](
                    data.as_ref()
                )
            }

            /// Filters a resource to the requested top-level fields.
            ///
            /// Mandatory fields are retained even when they are not included
            /// in fields.
            ///
            /// # Errors
            ///
            /// Returns [`FilterFieldsError::UnknownField`] if fields contains a
            /// field that is not supported by the resource type.
            #[inline]
            pub fn filter(
                &self,
                resource: Resource,
                fields: &[&str],
            ) -> Result<Resource, FilterFieldsError> {
                RESOURCE_FILTERS[*self as usize](
                    resource,
                    fields,
                )
            }
        }

        impl AsRef<str> for ResourceType {
            #[inline]
            fn as_ref(&self) -> &str {
                RESOURCE_TYPE_NAMES[*self as usize]
            }
        }

        impl TryFrom<String> for ResourceType {
            type Error = ResourceTypeError;

            #[inline]
            fn try_from(
                s: String,
            ) -> Result<Self, Self::Error> {
                Self::try_from(s.as_str())
            }
        }

        impl TryFrom<&str> for ResourceType {
            type Error = ResourceTypeError;

            #[inline]
            fn try_from(
                s: &str,
            ) -> Result<Self, Self::Error> {
                match s.as_bytes().first().copied() {
                    #(#first_byte_dispatch)*

                    _ => Err(
                        ResourceTypeError::Invalid(
                            s.to_string(),
                        )
                    ),
                }
            }
        }
    }
}

fn generate_resource_type_enum_variants(
    resource_types: &[ResourceTypeInfo],
) -> impl Iterator<Item = TokenStream> + '_ {
    resource_types.iter().map(|resource_type_info| {
        let struct_name = format_ident!(
            "{}",
            generate::capitalize(&resource_type_info.rust_type_name)
        );

        let type_name = &resource_type_info.resource_type;

        if resource_type_info.rust_type_name == resource_type_info.resource_type {
            quote! {
                #struct_name
            }
        } else {
            quote! {
                #[serde(rename = #type_name)]
                #struct_name
            }
        }
    })
}

fn generate_resource_type_names(
    resource_types: &[ResourceTypeInfo],
) -> impl Iterator<Item = TokenStream> + '_ {
    resource_types.iter().map(|resource_type_info| {
        let resource_name = &resource_type_info.resource_type;

        quote! {
            #resource_name
        }
    })
}

fn generate_resource_type_deserializers(
    resource_types: &[ResourceTypeInfo],
) -> impl Iterator<Item = TokenStream> + '_ {
    resource_types.iter().map(|resource_type_info| {
        let struct_name = format_ident!(
            "{}",
            generate::capitalize(&resource_type_info.rust_type_name)
        );

        let function_name = format_ident!(
            "deserialize_{}",
            resource_type_info.rust_type_name.to_ascii_lowercase()
        );

        quote! {
            #[inline]
            fn #function_name(
                data: &[u8],
            ) -> Result<
                Resource,
                haste_fhir_serialization_json::errors::DeserializeError,
            > {
                Ok(Resource::#struct_name(
                    serde_json::from_slice::<#struct_name>(data)?
                ))
            }
        }
    })
}

fn generate_resource_type_deserializer_table(
    resource_types: &[ResourceTypeInfo],
) -> impl Iterator<Item = TokenStream> + '_ {
    resource_types.iter().map(|resource_type_info| {
        let function_name = format_ident!(
            "deserialize_{}",
            resource_type_info.rust_type_name.to_ascii_lowercase()
        );

        quote! {
            #function_name
        }
    })
}

fn generate_resource_type_filters(
    resource_types: &[ResourceTypeInfo],
) -> impl Iterator<Item = TokenStream> + '_ {
    resource_types.iter().map(|resource_type_info| {
        let struct_name = format_ident!(
            "{}",
            generate::capitalize(&resource_type_info.rust_type_name)
        );

        let function_name = format_ident!(
            "filter_{}",
            resource_type_info.rust_type_name.to_ascii_lowercase()
        );

        quote! {
            #[inline]
            fn #function_name(
                resource: Resource,
                fields: &[&str],
            ) -> Result<Resource, FilterFieldsError> {
                match resource {
                    Resource::#struct_name(resource) => {
                        resource
                            .filter(fields)
                            .map(Resource::#struct_name)
                    }

                    _ => Err(
                        FilterFieldsError::ResourceTypeMismatch
                    ),
                }
            }
        }
    })
}

fn generate_resource_type_filter_table(
    resource_types: &[ResourceTypeInfo],
) -> impl Iterator<Item = TokenStream> + '_ {
    resource_types.iter().map(|resource_type_info| {
        let function_name = format_ident!(
            "filter_{}",
            resource_type_info.rust_type_name.to_ascii_lowercase()
        );

        quote! {
            #function_name
        }
    })
}

fn generate_resource_type_first_byte_dispatch(
    resource_types: &[ResourceTypeInfo],
) -> Vec<TokenStream> {
    let mut first_bytes = BTreeSet::new();

    for resource_type_info in resource_types {
        if let Some(&first_byte) = resource_type_info.resource_type.as_bytes().first() {
            first_bytes.insert(first_byte);
        }
    }

    first_bytes
        .into_iter()
        .map(|first_byte| {
            let letter = first_byte as char;

            let helper_name = format_ident!("resource_type_from_{}", letter.to_ascii_lowercase());

            let byte_literal = proc_macro2::Literal::byte_character(first_byte);

            quote! {
                Some(#byte_literal) =>
                    Self::#helper_name(s),
            }
        })
        .collect()
}

fn generate_filter_fields_error() -> TokenStream {
    quote! {
        #[derive(Error, Debug)]
        pub enum FilterFieldsError {
            #[error(
                "Unknown or unsupported _elements field '{0}' \
                 for resource type '{1}'"
            )]
            UnknownField(String, String),

            #[error(
                "Resource type does not match the provided resource"
            )]
            ResourceTypeMismatch,
        }
    }
}

pub struct GeneratedCode {
    pub resources: TokenStream,
    pub types: TokenStream,
}

fn collect_identifiers(tokens: &TokenStream, identifiers: &mut BTreeSet<String>) {
    for token in tokens.clone() {
        match token {
            proc_macro2::TokenTree::Ident(ident) => {
                identifiers.insert(ident.to_string());
            }

            proc_macro2::TokenTree::Group(group) => {
                collect_identifiers(&group.stream(), identifiers);
            }

            _ => {}
        }
    }
}

fn generate_headers(type_imports: &[String]) -> (TokenStream, TokenStream) {
    let type_import_idents = type_imports.iter().map(|name| type_ident(name));

    let resource_type_imports = if type_imports.is_empty() {
        quote! {}
    } else {
        quote! {
            use self::super::types::{
                #(#type_import_idents),*
            };
        }
    };

    let resource_code = quote! {
        #![allow(non_snake_case)]
        #![doc = " @generated by `bash scripts/types_build.sh` - do not edit."]

        #resource_type_imports

        use self::super::terminology;
        use haste_reflect::{MetaValue, derive::Reflect};
        use haste_fhir_serialization_json;
        use thiserror::Error;
    };

    let type_code = quote! {
        #![allow(non_snake_case)]
        #![doc = " @generated by `bash scripts/types_build.sh` - do not edit."]

        use self::super::resources::Resource;
        use self::super::terminology;
        use haste_reflect::{MetaValue, derive::Reflect};
        use haste_fhir_serialization_json;
    };

    (resource_code, type_code)
}

fn generate_from_files(
    file_paths: &[String],
    level: Option<&'static str>,
    inlined_terminology: &HashMap<String, String>,
    resource_code: &mut TokenStream,
    type_code: &mut TokenStream,
    rust_type_name_to_fhir_type: &mut BTreeMap<String, String>,
    resource_types: &mut Vec<ResourceTypeInfo>,
) -> Result<(), String> {
    for dir_path in file_paths {
        for entry in WalkDir::new(dir_path)
            .sort_by_file_name()
            .into_iter()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().is_file())
            .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "json"))
        {
            let generated_types =
                generate_fhir_types_from_file(entry.path(), level, inlined_terminology)?;

            rust_type_name_to_fhir_type.extend(generated_types.rust_type_name_to_fhir_type);

            resource_types.extend(generated_types.resource_types);

            // Avoid repeatedly rebuilding the complete accumulated
            // TokenStream with quote!.
            resource_code.extend(generated_types.resources);
            type_code.extend(generated_types.types);
        }
    }

    Ok(())
}

fn generate_resource_enum(resource_types: &[ResourceTypeInfo]) -> TokenStream {
    let resource_type_enum_variant_idents = resource_types.iter().map(|resource_type_info| {
        let rust_struct_name = &resource_type_info.rust_type_name;
        let resource_type_name = &resource_type_info.resource_type;

        let variant = format_ident!("{}", generate::capitalize(rust_struct_name));

        if rust_struct_name == resource_type_name {
            quote! {
                #variant(#variant)
            }
        } else {
            quote! {
                #[serde(rename = #resource_type_name)]
                #variant(#variant)
            }
        }
    });

    let resource_to_resource_type_match_arms = resource_types.iter().map(|resource_type_info| {
        let resource_type_ident = format_ident!("{}", &resource_type_info.rust_type_name);

        quote! {
            Resource::#resource_type_ident(_) =>
                ResourceType::#resource_type_ident
        }
    });

    let resource_to_id_match_arms = resource_types.iter().map(|resource_type_info| {
        let resource_type_ident = format_ident!("{}", &resource_type_info.rust_type_name);

        quote! {
            Resource::#resource_type_ident(r) => &r.id
        }
    });

    quote! {
        #[derive(
            Clone,
            Reflect,
            Debug,
            haste_fhir_serialization_json::derive::FHIRSerdeSerialize,
            serde::Deserialize,
        )]
        #[fhir_serialize_type = "enum-variant"]
        #[serde(tag = "resourceType")]
        pub enum Resource {
            #(#resource_type_enum_variant_idents),*
        }

        impl Resource {
            #[inline]
            pub fn empty(&self) -> bool {
                false
            }

            /// Filters this resource to the requested top-level fields.
            ///
            /// Fields that are mandatory for the resource are retained
            /// regardless of whether they are included in fields.
            ///
            /// # Errors
            ///
            /// Returns [`FilterFieldsError::UnknownField`] if fields contains a
            /// field that is not supported by this resource type.
            #[inline]
            pub fn filter(
                self,
                fields: &[&str],
            ) -> Result<Resource, FilterFieldsError> {
                let resource_type = self.resource_type();

                resource_type.filter(
                    self,
                    fields,
                )
            }

            #[inline]
            #[allow(clippy::too_many_lines)]
            pub fn resource_type(&self) -> ResourceType {
                match self {
                    #(#resource_to_resource_type_match_arms),*
                }
            }

            #[inline]
            #[allow(clippy::too_many_lines)]
            pub fn id(&self) -> &Option<String> {
                match self {
                    #(#resource_to_id_match_arms),*
                }
            }
        }
    }
}

/*
 * 067  public static final String FP_String = "http://hl7.org/fhirpath/System.String";
 * 068  public static final String FP_Boolean = "http://hl7.org/fhirpath/System.Boolean";
 * 069  public static final String FP_Integer = "http://hl7.org/fhirpath/System.Integer";
 * 070  public static final String FP_Decimal = "http://hl7.org/fhirpath/System.Decimal";
 * 071  public static final String FP_Quantity = "http://hl7.org/fhirpath/System.Quantity";
 * 072  public static final String FP_DateTime = "http://hl7.org/fhirpath/System.DateTime";
 * "http://hl7.org/fhirpath/System.Date"
 * 073  public static final String FP_Time = "http://hl7.org/fhirpath/System.Time";
 */
#[allow(dead_code)]
static PRIMITIVE_TYPES: &[&str] = &[
    "http://hl7.org/fhirpath/System.String",
    "http://hl7.org/fhirpath/System.Boolean",
    "http://hl7.org/fhirpath/System.Integer",
    "http://hl7.org/fhirpath/System.Decimal",
    "http://hl7.org/fhirpath/System.Quantity",
    "http://hl7.org/fhirpath/System.DateTime",
    "http://hl7.org/fhirpath/System.Date",
    "http://hl7.org/fhirpath/System.Time",
];

pub fn generate(
    file_paths: &[String],
    level: Option<&'static str>,
    inlined_terminology: &HashMap<String, String>,
) -> Result<GeneratedCode, String> {
    let mut resource_code = TokenStream::new();
    let mut type_code = TokenStream::new();

    let mut rust_type_name_to_fhir_type = BTreeMap::<String, String>::new();

    let mut resource_types = Vec::new();

    generate_from_files(
        file_paths,
        level,
        inlined_terminology,
        &mut resource_code,
        &mut type_code,
        &mut rust_type_name_to_fhir_type,
        &mut resource_types,
    )?;

    let mut resource_identifiers = BTreeSet::new();

    collect_identifiers(&resource_code, &mut resource_identifiers);

    let mut type_imports: Vec<String> = rust_type_name_to_fhir_type
        .keys()
        .filter(|type_name| resource_identifiers.contains(*type_name))
        .cloned()
        .collect();

    type_imports.push("Element".to_string());

    type_imports.sort_unstable();
    type_imports.dedup();

    let (resource_header, type_header) = generate_headers(&type_imports);

    let resource_enum = generate_resource_enum(&resource_types);

    let resource_type_type = generate_resource_type(&resource_types);

    let filter_fields_error = generate_filter_fields_error();

    let filter_field_helper = generate_filter_field_helper();

    resource_code = {
        let mut output = TokenStream::new();

        output.extend(resource_header);
        output.extend(resource_code);
        output.extend(resource_enum);
        output.extend(resource_type_type);
        output.extend(filter_fields_error);
        output.extend(filter_field_helper);

        output
    };

    type_code = {
        let mut output = TokenStream::new();

        output.extend(type_header);
        output.extend(type_code);

        output
    };

    Ok(GeneratedCode {
        resources: resource_code,
        types: type_code,
    })
}
