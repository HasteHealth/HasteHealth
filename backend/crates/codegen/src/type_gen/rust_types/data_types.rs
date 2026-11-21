use std::{collections::HashMap, path::Path};

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
        extract::Max::Unlimited if min > 0 => quote! { #[cardinality(min = #min)] },
        // `Fixed(1)` means it's a singular value.
        extract::Max::Unlimited | extract::Max::Fixed(1) => quote! {},
        extract::Max::Fixed(max) if min > 0 => quote! { #[cardinality(min = #min, max = #max)] },
        extract::Max::Fixed(max) => quote! { #[cardinality(max = #max)] },
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
/// of what a client asked to subset down to, per the `_elements`/`_summary`
/// rules. `id`/`meta` are forced in explicitly since they're min=0 in the base
/// resource but still mandatory under those rules.
fn get_mandatory_fields(sd: &StructureDefinition, all_fields: &[String]) -> Vec<String> {
    let mut mandatory = vec!["id".to_string(), "meta".to_string()];
    mandatory.extend(get_top_level_required_fields(sd));
    mandatory.retain(|field| all_fields.contains(field));
    mandatory
}

/// Generates the `impl <Resource>::filter` used to implement `_elements`
/// <https://hl7.org/fhir/R4/search.html#elements>: `id`/`meta`, any element
/// with `min >= 1`, and any element the caller explicitly requested are kept,
/// every other element is dropped. `_elements` only addresses top-level
/// elements, so this never recurses into complex-type substructure. Unknown
/// field names are rejected.
fn generate_filter_impl(sd: &StructureDefinition, struct_ident: &Ident) -> TokenStream {
    let resource_type = sd.type_.value.clone().unwrap_or_default();

    let all_fields = get_top_level_fields(sd);
    let mandatory_fields = get_mandatory_fields(sd, &all_fields);

    let field_values = all_fields.iter().map(|field| {
        let ident = field_ident(field);
        if mandatory_fields.contains(field) {
            quote! { #ident: self.#ident }
        } else {
            // An element with `min = 0` is always an `Option`.
            quote! { #ident: self.#ident.filter(|_| fields.contains(&#field)) }
        }
    });

    quote! {
        impl #struct_ident {
            /// Keeps `id`, `meta`, the mandatory elements and the elements named in `fields`.
            ///
            /// # Errors
            ///
            /// Returns [`FilterFieldsError::UnknownField`] if `fields` names an element
            /// this resource does not have.
            pub fn filter(self, fields: &[&str]) -> Result<Self, FilterFieldsError> {
                const FIELDS: &[&str] = &[#(#all_fields),*];

                if let Some(field) = fields.iter().find(|field| !FIELDS.contains(field)) {
                    return Err(FilterFieldsError::UnknownField(
                        (*field).to_string(),
                        #resource_type.to_string(),
                    ));
                }

                Ok(Self {
                    #(#field_values),*
                })
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
        extract::Max::Fixed(1) if should_box => quote! { Box<#field_value> },
        extract::Max::Fixed(1) => field_value.clone(),
        extract::Max::Unlimited | extract::Max::Fixed(_) => quote! { Vec<#field_value> },
    }
}

fn wrap_cardinality_and_optionality(
    element: &ElementDefinition,
    field_value: &TokenStream,
    should_box: bool,
) -> TokenStream {
    let field_value = wrap_if_vec(element, field_value, should_box);

    if extract::cardinality(element).0 == 0 {
        quote! { Option<#field_value> }
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
    let content_reference_id = &element
        .contentReference
        .as_ref()
        .and_then(|content_reference| content_reference.value.as_deref())
        .expect("element has no contentReference")[1..];

    let content_reference_element = sd
        .snapshot
        .as_ref()
        .expect("StructureDefinition has no snapshot")
        .element
        .iter()
        .filter(|element| element.id.as_deref() == Some(content_reference_id))
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
    inlined_terminology: &HashMap<String, String>,
) -> TokenStream {
    let struct_name = generate::struct_name(sd, element);
    let fhir_type = extract::fhir_type(sd, element);
    let struct_ident = type_ident(&struct_name);

    let description = extract::element_description(element);
    let formatted_description = format_documentation(&description);
    let doc_attributes = generate_doc_attributes(&formatted_description);

    let (derive, additional_impls) = if conditionals::is_root(sd, element)
        && conditionals::is_primitive_sd(sd)
    {
        let from_impl = from_rust_type_to_fhir_primitive(&struct_ident, sd, inlined_terminology);

        let impls = quote! {
            impl #struct_ident {
                pub fn extension_mut(&mut self) -> &mut Option<Vec<Extension>> {
                    &mut self.extension
                }
                pub fn id_mut(&mut self) -> &mut Option<String> {
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
    // Extract StructureDefinitions
    let structure_definitions = load::get_structure_definitions(&resource, level)
        .map_err(|e| format!("Failed to get structure definitions: {e}"))?;

    let mut resources = vec![];
    let mut types = vec![];
    let mut resource_types = vec![];

    for sd in structure_definitions
        .iter()
        .filter(|sd| should_generate_structure_definition(sd))
    {
        let generated = generate_from_structure_definition(sd, inlined_terminology)?;

        if conditionals::is_resource_sd(sd) {
            resource_types.push(ResourceTypeInfo {
                resource_type: sd.type_.value.clone().expect("resource has no type"),
                rust_type_name: sd.id.clone().expect("resource has no id"),
            });
            resources.push(generated);
        } else {
            types.push(generated);
        }
    }

    Ok(GeneratedTypes {
        resources,
        types,
        resource_types,
    })
}

fn generate_resource_type(resource_types: &[ResourceTypeInfo]) -> TokenStream {
    let enum_variants = resource_types.iter().map(|resource_type_info| {
        let variant = rust_resource_ident(&resource_type_info.rust_type_name);
        let type_name = &resource_type_info.resource_type;

        if resource_type_info.rust_type_name == resource_type_info.resource_type {
            quote! { #variant }
        } else {
            quote! {
                #[serde(rename = #type_name)]
                #variant
            }
        }
    });

    let resource_names = resource_types
        .iter()
        .map(|resource_type_info| &resource_type_info.resource_type);

    let deserialize_arms = resource_types.iter().map(|resource_type_info| {
        let variant = rust_resource_ident(&resource_type_info.rust_type_name);

        quote! {
            ResourceType::#variant => Ok(Resource::#variant(serde_json::from_slice::<#variant>(data.as_ref())?))
        }
    });

    let to_str_arms = resource_types.iter().map(|resource_type_info| {
        let variant = rust_resource_ident(&resource_type_info.rust_type_name);
        let resource_name = &resource_type_info.resource_type;

        quote! { ResourceType::#variant => #resource_name }
    });

    let from_str_arms = resource_types.iter().map(|resource_type_info| {
        let rust_type_name = &resource_type_info.rust_type_name;
        let resource_name = &resource_type_info.resource_type;
        let variant = rust_resource_ident(rust_type_name);

        if rust_type_name == resource_name {
            quote! { #resource_name => Ok(ResourceType::#variant) }
        } else {
            quote! { #rust_type_name | #resource_name => Ok(ResourceType::#variant) }
        }
    });

    quote! {
        #[derive(Error, Debug)]
        pub enum ResourceTypeError {
            #[error("Invalid resource type: {0}")]
            Invalid(String),
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Deserialize, serde::Serialize, PartialOrd, Ord)]
        pub enum ResourceType {
            #(#enum_variants),*
        }

        impl ResourceType {
            /// The FHIR name of every resource type, as `as_ref` returns it.
            pub const NAMES: &'static [&'static str] = &[#(#resource_names),*];

            /// Deserializes `data` as a resource of this type.
            ///
            /// # Errors
            ///
            /// Returns a `DeserializeError` if `data` is not a valid resource of this type.
            pub fn deserialize<D: AsRef<[u8]>>(self, data: D) -> Result<Resource, haste_fhir_serialization_json::errors::DeserializeError> {
                match self {
                    #(#deserialize_arms),*
                }
            }
        }

        impl AsRef<str> for ResourceType {
            fn as_ref(&self) -> &str {
                match self {
                    #(#to_str_arms),*
                }
            }
        }

        impl TryFrom<String> for ResourceType {
            type Error = ResourceTypeError;

            fn try_from(s: String) -> Result<Self, Self::Error> {
                Self::try_from(s.as_str())
            }
        }

        impl TryFrom<&str> for ResourceType {
            type Error = ResourceTypeError;

            fn try_from(s: &str) -> Result<Self, Self::Error> {
                match s {
                    #(#from_str_arms),*,
                    _ => Err(ResourceTypeError::Invalid(s.to_string())),
                }
            }
        }
    }
}

fn generate_filter_fields_error() -> TokenStream {
    quote! {
        #[derive(Error, Debug)]
        pub enum FilterFieldsError {
            #[error("Unknown or unsupported _elements field '{0}' for resource type '{1}'")]
            UnknownField(String, String),
        }
    }
}

pub struct GeneratedCode {
    pub resources: TokenStream,
    pub types: TokenStream,
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
    let (mut resource_code, mut type_code) = generate_headers();
    let mut resource_types = vec![];

    generate_from_files(
        file_paths,
        level,
        inlined_terminology,
        &mut resource_code,
        &mut type_code,
        &mut resource_types,
    )?;

    resource_code.extend(generate_resource_enum(&resource_types));
    resource_code.extend(generate_resource_type(&resource_types));
    resource_code.extend(generate_filter_fields_error());

    Ok(GeneratedCode {
        resources: resource_code,
        types: type_code,
    })
}

fn generate_headers() -> (TokenStream, TokenStream) {
    // Generated code: every resource match covers all ~150 resource types, so
    // long function bodies are expected.
    let resource_code = quote! {
        #![allow(non_snake_case, clippy::too_many_lines, clippy::wildcard_imports)]
        #![doc = " @generated by `bash scripts/types_build.sh` - do not edit."]
        use self::super::types::*;
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

            resource_types.extend(generated_types.resource_types);
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

        let variant = rust_resource_ident(rust_struct_name);

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
            Resource::#resource_type_ident(_) => ResourceType::#resource_type_ident
        }
    });

    let resource_to_id_match_arms = resource_types.iter().map(|resource_type_info| {
        let resource_type_ident = format_ident!("{}", &resource_type_info.rust_type_name);

        quote! {
            Resource::#resource_type_ident(r) => &r.id
        }
    });

    let filter_fields = resource_types.iter().map(|resource_type_info| {
        let resource_type_ident = format_ident!("{}", &resource_type_info.rust_type_name);

        quote! {
            Resource::#resource_type_ident(r) => r.filter(fields).map(Resource::#resource_type_ident)
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
            /// Returns true if the resource is empty, false otherwise.
            pub fn empty(&self) -> bool {
                false
            }

            /// Keeps `id`, `meta`, the mandatory elements and the elements named in `fields`.
            ///
            /// # Errors
            ///
            /// Returns [`FilterFieldsError::UnknownField`] if `fields` names an element
            /// this resource does not have.
            pub fn filter(self, fields: &[&str]) -> Result<Resource, FilterFieldsError> {
                match self {
                    #(#filter_fields),*
                }
            }

            pub fn resource_type(&self) -> ResourceType {
                match self {
                    #(#resource_to_resource_type_match_arms),*
                }
            }

            pub fn id(&self) -> &Option<String> {
                match self {
                    #(#resource_to_id_match_arms),*
                }
            }
        }
    }
}
