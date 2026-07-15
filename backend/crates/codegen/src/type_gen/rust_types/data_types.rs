use std::{
    collections::{BTreeMap, HashMap},
    path::Path,
};

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

fn min_max_attribute(element: &ElementDefinition) -> TokenStream {
    let cardinality = extract::cardinality(element);
    let max = cardinality.1;
    let min = cardinality.0;

    match max {
        extract::Max::Unlimited => {
            if min > 0 {
                quote! { #[cardinality(min = #min)] }
            } else {
                quote! {}
            }
        }
        // Means it's a singular value.
        extract::Max::Fixed(1) => quote! {},
        extract::Max::Fixed(n) => {
            if min > 0 {
                quote! { #[cardinality(min = #min, max = #n)] }
            } else {
                quote! { #[cardinality(max = #n)] }
            }
        }
    }
}

fn wrap_if_vec(element: &ElementDefinition, field_value: TokenStream) -> TokenStream {
    let cardinality = extract::cardinality(element);

    // Check the cardinality.
    let wrapped_field = match cardinality.1 {
        extract::Max::Unlimited => quote! {
            Vec<#field_value>
        },
        extract::Max::Fixed(1) => quote! {
            #field_value
        },
        extract::Max::Fixed(_n) => quote! {
            Vec<#field_value>
        },
    };

    wrapped_field
}

fn wrap_cardinality_and_optionality(
    element: &ElementDefinition,
    field_value: TokenStream,
) -> TokenStream {
    let cardinality = extract::cardinality(element);

    let field_value = wrap_if_vec(element, field_value);

    // Check the Optionality
    if cardinality.0 == 0 {
        quote! {
            Option<#field_value>
        }
    } else {
        field_value
    }
}

fn get_reference_target_attribute(element: &ElementDefinition) -> TokenStream {
    if let Some(type_vec) = element.type_.as_ref()
        && let Some(reference_type) = type_vec
            .iter()
            .find(|t| t.code.value.as_ref().map(|s| s.as_str()) == Some("Reference"))
        && let Some(targets) = reference_type.targetProfile.as_ref()
    {
        let profiles = targets
            .iter()
            .filter_map(
                |tp: &Box<haste_fhir_model::r4::generated::types::FHIRCanonical>| tp.value.as_ref(),
            )
            .filter_map(|tp| tp.split("/").last())
            .collect::<Vec<_>>();
        quote! {
            #[reference(targets = [#(#profiles),*])]
        }
    } else {
        quote! {}
    }
}

fn get_struct_key_value(
    element: &ElementDefinition,
    field_value_type_name: TokenStream,
) -> TokenStream {
    let description = extract::element_description(element);
    let field_name = extract::field_name(&extract::path(element));
    let field_name_ident = if RUST_KEYWORDS.contains(&field_name.as_str()) {
        format_ident!("{}_", field_name)
    } else {
        format_ident!("{}", field_name)
    };

    let reflect_attribute = if RUST_KEYWORDS.contains(&field_name.as_str()) {
        quote! {
            #[rename_field = #field_name]
        }
    } else {
        quote! {}
    };

    let type_choice_variants = if conditionals::is_typechoice(element) {
        let type_choice_variants = generate::create_type_choice_variants(element);
        let type_choice_primitives = generate::create_type_choice_primitive_variants(element);
        let type_choice_complex_variants = type_choice_variants
            .iter()
            .filter(|variant| !type_choice_primitives.contains(variant));

        quote! {
           #[type_choice_variants(complex = [#(#type_choice_complex_variants),*], primitive = [#(#type_choice_primitives),*])]
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

    // For typechoices set the header on the variant.
    let target_types = if !conditionals::is_typechoice(element) {
        get_reference_target_attribute(element)
    } else {
        quote! {}
    };

    let cardinality_attribute = min_max_attribute(element);
    let field_value = wrap_cardinality_and_optionality(element, field_value_type_name);

    quote! {
        #type_choice_variants
        #reflect_attribute
        #primitive_attribute
        #cardinality_attribute
        #target_types
        #[doc = #description]
        pub #field_name_ident: #field_value
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

    let content_reference_element: Vec<&Box<ElementDefinition>> = sd
        .snapshot
        .as_ref()
        .ok_or("StructureDefinition has no snapshot")
        .unwrap()
        .element
        .iter()
        .filter(|e| e.id == Some(content_reference_id.to_string()))
        .collect();

    if content_reference_element.len() != 1 {
        panic!(
            "Content reference element not found {}",
            content_reference_id
        );
    }

    let content_reference_element = content_reference_element[0];
    content_reference_element
}

fn create_type_choice(
    sd: &StructureDefinition,
    element: &ElementDefinition,
    inlined_terminology: &HashMap<String, String>,
) -> TokenStream {
    let field_name = extract::field_name(&extract::path(element));
    let type_name = format_ident!("{}", generate::type_choice_name(sd, element));
    let types = extract::field_types(element);

    let enum_variants = types
        .iter()
        .map(|fhir_type| {
            let enum_name = format_ident!("{}", generate::capitalize(fhir_type));
            let rust_type = wrap_if_vec(
                element,
                fhir_type_to_rust_type(element, fhir_type, inlined_terminology),
            );
            // For Reference types, extract target profiles and use as an attribute.
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
        })
        .collect::<Vec<TokenStream>>();

    let default_enum = format_ident!("{}", generate::capitalize(&types[0].to_string()));
    let default_impl = if conditionals::should_be_boxed(&types[0].to_string()) {
        quote! {
            impl Default for #type_name {
                fn default() -> Self {
                    #type_name::#default_enum(Box::new(Default::default()))
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
            haste_fhir_serialization_json::derive::FHIRSerdeDeserialize)]
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
        get_struct_key_value(element, field_type_name)
    } else if conditionals::is_typechoice(element) {
        let type_choice_name_ident = field_typename(sd, element, inlined_terminology);
        let type_choice = create_type_choice(sd, element, inlined_terminology);

        types.insert(type_choice_name_ident.to_string(), type_choice);

        get_struct_key_value(element, quote! {#type_choice_name_ident})
    } else {
        let fhir_type = extract::field_types(element)[0];
        let rust_type = fhir_type_to_rust_type(element, fhir_type, inlined_terminology);

        get_struct_key_value(element, rust_type)
    }
}

fn from_rust_type_to_fhir_primitive<'a>(
    sd_ident: &Ident,
    sd: &'a StructureDefinition,
    inlined_terminology: &HashMap<String, String>,
) -> TokenStream {
    let value_element = sd
        .snapshot
        .as_ref()
        .map(|s| &s.element)
        .and_then(|element_definitions| {
            element_definitions
                .iter()
                .filter(|e| {
                    e.path
                        .value
                        .as_ref()
                        .map(|p| p.ends_with(".value"))
                        .unwrap_or(false)
                })
                .next()
        });

    if let Some(value_element) = value_element
        && let Some(fhir_type) = extract::field_types(&*value_element).get(0)
    {
        let value_type = fhir_type_to_rust_type(value_element, *fhir_type, inlined_terminology);

        if value_element
            .min
            .as_ref()
            .and_then(|min| min.value)
            .unwrap_or(0)
            > 0
        {
            quote! {
                impl From<#value_type> for #sd_ident {
                    fn from(value: #value_type) -> Self {
                        Self { value: value, ..Default::default() }
                    }
                }
            }
        } else {
            quote! {
                impl From<#value_type> for #sd_ident {
                    fn from(value: #value_type) -> Self {
                        let mut instance = Self::default();
                        instance.value = Some(value);
                        instance
                    }
                }
            }
        }
    } else {
        quote! {}
    }
}

fn create_complex_struct(
    sd: &StructureDefinition,
    element: &ElementDefinition,
    children: Vec<TokenStream>,
    types: &mut NestedTypes,
    rust_type_name_to_fhir_type: &mut HashMap<String, String>,
    inlined_terminology: &HashMap<String, String>,
) -> TokenStream {
    let struct_name = generate::struct_name(sd, element);
    let fhir_type = extract::fhir_type(sd, element);

    rust_type_name_to_fhir_type.insert(struct_name.clone(), fhir_type.clone());

    let struct_ident = format_ident!("{}", struct_name.clone());
    let description = extract::element_description(element);
    let mut additional_impls = quote! {};

    let derive = if conditionals::is_root(sd, element) && conditionals::is_primitive_sd(sd) {
        let from_impl = from_rust_type_to_fhir_primitive(&struct_ident, sd, inlined_terminology);
        additional_impls = quote! {
            impl #struct_ident {
                pub fn extension_mut(&mut self) -> &mut Option<Vec<Box<Extension>>> {
                    &mut self.extension
                }
                pub fn id_mut(&mut self) -> &mut Option<String> {
                    &mut self.id
                }
            }
            #from_impl
        };

        quote! {
           #[derive(Clone, Reflect, Debug, Default,
                haste_fhir_serialization_json::derive::FHIRSerdeSerialize,
                haste_fhir_serialization_json::derive::FHIRSerdeDeserialize)]
            #[fhir_type = #fhir_type]
           #[fhir_serialize_type = "primitive"]
        }
    } else if conditionals::is_root(sd, element) && conditionals::is_resource_sd(sd) {
        let type_ = sd.type_.value.as_ref().unwrap();
        let resource_type_attribute = if *type_ != struct_name {
            quote! {
                #[fhir_resource_type = #type_]
            }
        } else {
            quote! {}
        };

        quote! {
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
        }
    } else {
        quote! {
            #[derive(
                Clone,
                Reflect,
                Debug,
                Default,
                haste_fhir_serialization_json::derive::FHIRSerdeSerialize,
                haste_fhir_serialization_json::derive::FHIRSerdeDeserialize)]
            #[fhir_type = #fhir_type]
            #[fhir_serialize_type = "complex"]
        }
    };

    let type_value = quote! {
        #derive
        #[doc = #description]
        pub struct #struct_ident {
            #(#children),*
        }
        #additional_impls
    };

    let i = struct_name.clone();
    types.insert(i, type_value);
    let i = format_ident!("{}", struct_name.clone());
    get_struct_key_value(element, quote! {#i})
}

fn generate_from_structure_definition(
    sd: &StructureDefinition,
    inlined_terminology: &HashMap<String, String>,
    rust_type_name_to_fhir_type: &mut HashMap<String, String>,
) -> Result<TokenStream, String> {
    let mut nested_types = IndexMap::<String, TokenStream>::new();

    let mut visitor =
        |element: &ElementDefinition, children: Vec<TokenStream>, _index: usize| -> TokenStream {
            if children.len() == 0 {
                process_leaf(&sd, element, &mut nested_types, inlined_terminology)
            } else {
                create_complex_struct(
                    &sd,
                    element,
                    children,
                    &mut nested_types,
                    rust_type_name_to_fhir_type,
                    inlined_terminology,
                )
            }
        };

    traversal::traversal(sd, &mut visitor)?;
    let types_generated = nested_types.values();

    let generated_code = quote! {
        #(#types_generated)*
    };

    Ok(generated_code)
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

fn generate_fhir_types_from_file(
    file_path: &Path,
    level: Option<&'static str>,
    inlined_terminology: &HashMap<String, String>,
) -> Result<GeneratedTypes, String> {
    let resource = load::load_from_file(file_path)?;
    // Extract StructureDefinitions
    let structure_definitions = load::get_structure_definitions(&resource, level)
        .map_err(|e| format!("Failed to get structure definitions: {}", e))?;

    let mut resources = vec![];
    let mut types = vec![];
    // let mut generated_code = vec![];
    let mut resource_types: Vec<ResourceTypeInfo> = vec![];
    let mut rust_type_name_to_fhir_type: HashMap<String, String> = HashMap::new();

    for sd in
        structure_definitions
            .iter()
            .filter(|sd| match sd.derivation.as_ref().map(|d| d.as_ref()) {
                Some(TypeDerivationRule::Specialization(_)) | None => match sd.kind.as_ref() {
                    StructureDefinitionKind::Resource(_) => !extract::is_abstract(sd),
                    _ => true,
                },
                _ => false,
            })
    {
        if conditionals::is_resource_sd(&sd) {
            resource_types.push(ResourceTypeInfo {
                resource_type: sd.type_.value.as_ref().unwrap().to_string(),
                rust_type_name: sd.id.as_ref().unwrap().to_string(),
            });

            resources.push(generate_from_structure_definition(
                sd,
                inlined_terminology,
                &mut rust_type_name_to_fhir_type,
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
        resource_types: resource_types,
        rust_type_name_to_fhir_type,
    })
}

fn generate_resource_type(resource_types: &Vec<ResourceTypeInfo>) -> TokenStream {
    let data_ident = format_ident!("data");
    let get_resource_deserialize_variant = resource_types.iter().map(|resource_type_info| {
        let struct_name = format_ident!("{}", generate::capitalize(&resource_type_info.rust_type_name));

        quote! {
            ResourceType::#struct_name => Ok(Resource::#struct_name(serde_json::from_str::<#struct_name>(#data_ident)?)),
        }
    });

    let enum_variants = resource_types.iter().map(|resource_type_info| {
        let struct_name = format_ident!(
            "{}",
            generate::capitalize(&resource_type_info.rust_type_name)
        );
        let type_name = &resource_type_info.resource_type;

        if resource_type_info.rust_type_name != resource_type_info.resource_type {
            quote! {
                #[serde(rename = #type_name)]
                #struct_name
            }
        } else {
            quote! {
                #struct_name
            }
        }
    });

    let from_str_variants = resource_types.iter().map(|resource_type_info| {
        let struct_name = &resource_type_info.rust_type_name;
        let resource_type = format_ident!("{}", generate::capitalize(struct_name));
        let resource_name = &resource_type_info.resource_type;

        if resource_type != resource_name {
            quote! {
                #struct_name | #resource_name => Ok(ResourceType::#resource_type)
            }
        } else {
            quote! {
                #resource_name => Ok(ResourceType::#resource_type)
            }
        }
    });

    let from_string_variants = from_str_variants.clone();

    let to_str_variants = resource_types.iter().map(|resource_name| {
        let resource_type =
            format_ident!("{}", generate::capitalize(&resource_name.rust_type_name));
        let resource_name = &resource_name.resource_type;
        quote! {
            ResourceType::#resource_type => #resource_name,
        }
    });

    quote! {
        #[derive(Error, Debug)]
        pub enum ResourceTypeError {
            #[error("Invalid resource type: {0}")]
            Invalid(String),
        }

        #[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Deserialize, serde::Serialize, PartialOrd, Ord)]
        pub enum ResourceType {
            #(#enum_variants),*
        }

        impl ResourceType {
            pub fn deserialize(&self, #data_ident: &str) -> Result<Resource, haste_fhir_serialization_json::errors::DeserializeError> {
                match self {
                    #(#get_resource_deserialize_variant)*
                }
            }
        }

        impl AsRef<str> for ResourceType {
            fn as_ref(&self) -> &str {
                match self {
                    #(#to_str_variants)*
                }
            }
        }

        impl TryFrom<String> for ResourceType {
            type Error = ResourceTypeError;

            fn try_from(s: String) -> Result<Self, Self::Error> {
                match s.as_str() {
                    #(#from_string_variants),*,
                     _ => Err(ResourceTypeError::Invalid(s.to_string())),
                }
            }
        }

        impl TryFrom<&str> for ResourceType {
            type Error = ResourceTypeError;

            fn try_from(s: &str) -> Result<Self, Self::Error> {
                match s {
                    #(#from_str_variants),*,
                    _ => Err(ResourceTypeError::Invalid(s.to_string())),
                }
            }
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
    file_paths: &Vec<String>,
    level: Option<&'static str>,
    inlined_terminology: &HashMap<String, String>,
) -> Result<GeneratedCode, String> {
    let mut resource_code = quote! {
        #![allow(non_snake_case)]
        /// DO NOT EDIT THIS FILE. It is auto-generated by the FHIR Rust code generator.
        use self::super::types::*;
        use self::super::terminology;
        use haste_reflect::{derive::Reflect, MetaValue};
        use haste_fhir_serialization_json;
        use thiserror::Error;
    };

    let mut type_code = quote! {
        #![allow(non_snake_case)]
        /// DO NOT EDIT THIS FILE. It is auto-generated by the FHIR Rust code generator.
        use self::super::resources::Resource;
        use self::super::terminology;
        use haste_reflect::{derive::Reflect, MetaValue};
        use haste_fhir_serialization_json;
    };

    let mut rust_type_name_to_fhir_type: BTreeMap<String, String> = BTreeMap::new();
    let mut resource_types: Vec<ResourceTypeInfo> = vec![];

    for dir_path in file_paths {
        let walker = WalkDir::new(dir_path).into_iter();
        for entry in walker
            .filter_map(|e| e.ok())
            .filter(|e| e.metadata().unwrap().is_file())
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "json"))
        {
            let generated_types =
                generate_fhir_types_from_file(entry.path(), level, inlined_terminology)?;
            let code = generated_types.resources;
            rust_type_name_to_fhir_type.extend(generated_types.rust_type_name_to_fhir_type);
            resource_types.extend(generated_types.resource_types);
            resource_code = quote! {
                #resource_code
                #(#code)*
            };

            let code = generated_types.types;
            type_code = quote! {
                #type_code
                #(#code)*
            };
        }
    }

    let resource_type_enum_variant_idents = resource_types.iter().map(|resource_type_info| {
        let rust_struct_name = &resource_type_info.rust_type_name;
        let resource_type_name = &resource_type_info.resource_type;

        let variant = format_ident!("{}", generate::capitalize(rust_struct_name));

        if rust_struct_name != resource_type_name {
            quote! {
                #[serde(rename = #resource_type_name)]
                #variant(#variant)
            }
        } else {
            quote! {
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

    let resource_enum = quote! {
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
            #[doc = "Returns true if the resource is empty, false otherwise."]
            pub fn empty(&self) -> bool {
                false
            }
            pub fn resource_type(&self) -> ResourceType {
                match self {
              #(#resource_to_resource_type_match_arms),*
                }
            }
            pub fn id<'a>(&'a self) -> &'a Option<String> {
                match self {
                #(#resource_to_id_match_arms),*
                }
            }
        }
    };

    let resource_type_type = generate_resource_type(&resource_types);
    // Add resourcetype plus the base Resource enum.
    resource_code = quote! {
        #resource_code
        #resource_enum
        #resource_type_type
    };

    Ok(GeneratedCode {
        resources: resource_code,
        types: type_code,
    })
}
