#![allow(unused)]
use std::{
    collections::{HashMap, HashSet},
    sync::LazyLock,
};

/// Some of these keywords are present as properties in the FHIR spec.
/// We need to prefix them with an underscore to avoid conflicts.
/// And use an attribute to rename the field in the generated code.
pub static RUST_KEYWORDS: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    let mut m = HashSet::new();
    m.insert("self");
    m.insert("Self");
    m.insert("super");
    m.insert("type");
    m.insert("use");
    m.insert("identifier");
    m.insert("abstract");
    m.insert("for");
    m.insert("if");
    m.insert("else");
    m.insert("match");
    m.insert("while");
    m.insert("loop");
    m.insert("break");
    m.insert("continue");
    m.insert("ref");
    m.insert("return");
    m.insert("async");
    m.insert("where");
    m.insert("in");
    m.insert("final");
    m.insert("as");
    m.insert("do");
    m.insert("box");
    m.insert("pub");
    m.insert("false");
    m.insert("true");
    m.insert("mod");
    m.insert("gen");
    m.insert("crate");
    m.insert("fn");
    m.insert("let");
    m.insert("const");
    m.insert("static");
    m.insert("struct");
    m.insert("enum");
    m.insert("trait");
    m.insert("impl");
    m.insert("unsafe");
    m.insert("extern");
    m.insert("move");
    m.insert("mut");
    m.insert("dyn");
    m.insert("await");
    m.insert("try");
    m.insert("yield");
    m.insert("macro");
    m.insert("union");
    m
});

pub static RUST_PRIMITIVES: LazyLock<HashMap<String, String>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert(
        "http://hl7.org/fhirpath/System.String".to_string(),
        "String".to_string(),
    );
    m.insert(
        "http://hl7.org/fhirpath/System.Decimal".to_string(),
        "f64".to_string(),
    );
    m.insert(
        "http://hl7.org/fhirpath/System.Boolean".to_string(),
        "bool".to_string(),
    );
    m.insert(
        "http://hl7.org/fhirpath/System.Integer".to_string(),
        "i64".to_string(),
    );
    m.insert(
        "http://hl7.org/fhirpath/System.Time".to_string(),
        "crate::r4::datetime::Time".to_string(),
    );
    m.insert(
        "http://hl7.org/fhirpath/System.Date".to_string(),
        "crate::r4::datetime::Date".to_string(),
    );
    m.insert(
        "http://hl7.org/fhirpath/System.DateTime".to_string(),
        "crate::r4::datetime::DateTime".to_string(),
    );
    m.insert(
        "http://hl7.org/fhirpath/System.Instant".to_string(),
        "crate::r4::datetime::Instant".to_string(),
    );
    m
});

pub static FHIR_PRIMITIVES: LazyLock<HashMap<String, String>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    // bool type
    m.insert("boolean".to_string(), "FHIRBoolean".to_string());

    // f64 type
    m.insert("decimal".to_string(), "FHIRDecimal".to_string());

    // i64 type
    m.insert("integer".to_string(), "FHIRInteger".to_string());
    // u64 type
    m.insert("positiveInt".to_string(), "FHIRPositiveInt".to_string());
    m.insert("unsignedInt".to_string(), "FHIRUnsignedInt".to_string());

    // String type
    m.insert("base64Binary".to_string(), "FHIRBase64Binary".to_string());
    m.insert("canonical".to_string(), "FHIRCanonical".to_string());
    m.insert("code".to_string(), "FHIRCode".to_string());
    m.insert("id".to_string(), "FHIRId".to_string());
    m.insert("markdown".to_string(), "FHIRMarkdown".to_string());
    m.insert("oid".to_string(), "FHIROid".to_string());
    m.insert("string".to_string(), "FHIRString".to_string());
    m.insert("uri".to_string(), "FHIRUri".to_string());
    m.insert("url".to_string(), "FHIRUrl".to_string());
    m.insert("uuid".to_string(), "FHIRUuid".to_string());
    m.insert("xhtml".to_string(), "FHIRXhtml".to_string());

    // Date and Time types
    m.insert("instant".to_string(), "FHIRInstant".to_string());
    m.insert("date".to_string(), "FHIRDate".to_string());
    m.insert("dateTime".to_string(), "FHIRDateTime".to_string());
    m.insert("time".to_string(), "FHIRTime".to_string());

    m
});

pub static FHIR_PRIMITIVE_VALUE_TYPE: LazyLock<HashMap<String, String>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    // bool type
    m.insert("boolean".to_string(), "bool".to_string());

    // f64 type
    m.insert("decimal".to_string(), "f64".to_string());

    // i64 type
    m.insert("integer".to_string(), "i64".to_string());
    // u64 type
    m.insert("positiveInt".to_string(), "u64".to_string());
    m.insert("unsignedInt".to_string(), "u64".to_string());

    // String type
    m.insert("base64Binary".to_string(), "String".to_string());
    m.insert("canonical".to_string(), "String".to_string());
    m.insert("code".to_string(), "String".to_string());
    m.insert("date".to_string(), "String".to_string());
    m.insert("dateTime".to_string(), "String".to_string());
    m.insert("id".to_string(), "String".to_string());
    m.insert("instant".to_string(), "String".to_string());
    m.insert("markdown".to_string(), "String".to_string());
    m.insert("oid".to_string(), "String".to_string());
    m.insert("string".to_string(), "String".to_string());
    m.insert("time".to_string(), "String".to_string());
    m.insert("uri".to_string(), "String".to_string());
    m.insert("url".to_string(), "String".to_string());
    m.insert("uuid".to_string(), "String".to_string());
    m.insert("xhtml".to_string(), "String".to_string());

    m
});

pub mod conversion {
    use std::collections::HashMap;

    use super::{FHIR_PRIMITIVES, RUST_PRIMITIVES};
    use haste_fhir_model::r4::generated::{terminology::BindingStrength, types::ElementDefinition};
    use proc_macro2::TokenStream;
    use quote::{format_ident, quote};

    /// Converts a FHIR type to its corresponding Rust type.
    ///
    /// Returns a tuple where:
    /// - The first element is the Rust type as a `TokenStream`.
    /// - The second element indicates whether the returned type is a generated
    ///   FHIR type (`true`) or a built-in Rust type (`false`).
    ///
    /// This function performs special handling for:
    /// - `unsignedInt.value` and `positiveInt.value`, which map to `u64`.
    /// - `instant.value`, which maps to the Rust `Instant` type.
    /// - Primitive FHIR types with required bindings that have been inlined into
    ///   generated terminology types.
    ///
    /// # Panics
    ///
    /// Panics if the internal `RUST_PRIMITIVES` mapping does not contain the
    /// `http://hl7.org/fhirpath/System.Instant` entry, or if any primitive type
    /// mapping stored in `RUST_PRIMITIVES` is not a valid `TokenStream`.
    pub fn fhir_type_to_rust_type<S: std::hash::BuildHasher>(
        element: &ElementDefinition,
        fhir_type: &str,
        inlined_terminology: &HashMap<String, String, S>,
    ) -> (TokenStream, bool) {
        let path = element.path.value.as_deref();

        match path {
            Some("unsignedInt.value" | "positiveInt.value") => {
                let k = format_ident!("{}", "u64");
                (
                    quote! {
                        #k
                    },
                    false,
                )
            }

            _ => {
                if let Some(rust_primitive) = RUST_PRIMITIVES.get(fhir_type) {
                    if matches!(path, Some("instant.value")) {
                        let k = RUST_PRIMITIVES
                            .get("http://hl7.org/fhirpath/System.Instant")
                            .unwrap()
                            .parse::<TokenStream>()
                            .unwrap();

                        (
                            quote! {
                                #k
                            },
                            false,
                        )
                    } else {
                        let k = rust_primitive.parse::<TokenStream>().unwrap();
                        (
                            quote! {
                                #k
                            },
                            false,
                        )
                    }
                } else if let Some(primitive) = FHIR_PRIMITIVES.get(fhir_type) {
                    // Support for inlined types.
                    // inlined could be a url | version for canonical.
                    // Only do inlined if the binding is required and exists as inlined terminology.

                    if Some(&BindingStrength::required())
                        == element.binding.as_ref().map(|b| &b.strength)
                        && let Some(canonical_string) = element
                            .binding
                            .as_ref()
                            .and_then(|b| b.valueSet.as_ref())
                            .and_then(|b| b.value.as_ref())
                            .map(std::string::String::as_str)
                        && let Some(url) = canonical_string.split('|').next()
                        && let Some(inlined) = inlined_terminology.get(url)
                    {
                        let inline_type = format_ident!("{}", inlined);
                        (
                            quote! {
                                terminology::BoundCode<terminology::#inline_type>
                            },
                            false,
                        )
                    } else {
                        let k = format_ident!("{}", primitive.clone());
                        (
                            quote! {
                                #k
                            },
                            true,
                        )
                    }
                } else {
                    let k = format_ident!("{}", fhir_type.to_string());
                    (
                        quote! {
                            #k
                        },
                        true,
                    )
                }
            }
        }
    }
}

pub mod extract {
    use haste_fhir_model::r4::generated::resources::StructureDefinition;
    use haste_fhir_model::r4::generated::types::ElementDefinition;
    pub fn field_types(element: &ElementDefinition) -> Vec<&str> {
        element.type_.as_ref().map_or_else(Vec::new, |types| {
            types
                .iter()
                .filter_map(|t| t.code.value.as_deref())
                .collect()
        })
    }

    #[must_use]
    pub fn field_name(path: &str) -> String {
        let field_name: String = path
            .split('.')
            .next_back()
            .unwrap_or("")
            .chars()
            .enumerate()
            .map(|(i, c)| {
                if i == 0 {
                    c.to_lowercase().next().unwrap_or(c)
                } else {
                    c
                }
            })
            .collect();
        if field_name.ends_with("[x]") {
            field_name.replace("[x]", "")
        } else {
            field_name.clone()
        }
    }

    pub fn is_abstract(sd: &StructureDefinition) -> bool {
        sd.abstract_.value == Some(true)
    }

    pub fn path(element: &ElementDefinition) -> String {
        element.path.value.clone().unwrap_or_default()
    }
    pub fn element_description(element: &ElementDefinition) -> String {
        element
            .definition
            .as_ref()
            .and_then(|d| d.value.as_ref())
            .cloned()
            .unwrap_or_else(|| {
                element
                    .path
                    .value
                    .clone()
                    .unwrap_or_else(|| "no description".to_string())
            })
    }

    /// Returns the FHIR type associated with an element.
    ///
    /// For the root element of a [`StructureDefinition`], the type is taken from
    /// the structure definition itself (`StructureDefinition.type_`).
    ///
    /// For all other elements, this function expects exactly one declared FHIR type
    /// in `ElementDefinition.type_` and returns its code.
    ///
    /// # Panics
    ///
    /// This function will panic if:
    /// - The root element does not have a `StructureDefinition.type_`.
    /// - A non-root element has no type code.
    /// - A non-root element has multiple declared types, as the FHIR type would be
    ///   ambiguous.
    ///
    /// # Arguments
    ///
    /// * `sd` - The [`StructureDefinition`] containing the element.
    /// * `element` - The [`ElementDefinition`] whose FHIR type is to be determined.
    ///
    /// # Returns
    ///
    /// A `String` containing the FHIR type name (e.g. `"Patient"`, `"string"`,
    /// `"CodeableConcept"`).
    pub fn fhir_type(sd: &StructureDefinition, element: &ElementDefinition) -> String {
        if crate::utilities::conditionals::is_root(sd, element) {
            sd.type_
                .value
                .as_ref()
                .expect("Root element must have a type")
                .clone()
        } else {
            let default_types = vec![];
            let fhir_types = element.type_.as_ref().unwrap_or(&default_types);
            if fhir_types.len() == 1 {
                fhir_types[0]
                    .code
                    .value
                    .as_ref()
                    .expect("Type must have a code")
                    .clone()
            } else {
                panic!("Element has multiple types, cannot determine FHIR type");
            }
        }
    }

    #[derive(Clone, Copy)]
    pub enum Max {
        Unlimited,
        Fixed(u64),
    }

    pub fn cardinality(element: &ElementDefinition) -> (u64, Max) {
        let min = element.min.as_ref().and_then(|m| m.value).map_or(0, |m| m);

        let max = element
            .max
            .as_ref()
            .and_then(|m| m.value.as_ref())
            .map(std::string::String::as_str)
            .and_then(|s| {
                if s == "*" {
                    Some(Max::Unlimited)
                } else {
                    s.parse::<u64>().ok().map(Max::Fixed)
                }
            });

        (min, max.unwrap_or(Max::Fixed(1)))
    }
}

pub mod generate {
    use std::collections::HashMap;

    use haste_fhir_model::r4::generated::{
        resources::StructureDefinition, types::ElementDefinition,
    };
    use proc_macro2::TokenStream;
    use quote::{format_ident, quote};

    use crate::utilities::{FHIR_PRIMITIVES, conditionals, conversion, extract};

    /// Capitalize the first character in s.
    #[must_use]
    pub fn capitalize(s: &str) -> String {
        let mut c = s.chars();
        match c.next() {
            None => String::new(),
            Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        }
    }

    /// Returns the generated Rust struct name for a FHIR element.
    ///
    /// For the root element, the struct name is derived from the structure
    /// definition's `id`. Primitive FHIR types are prefixed with `"FHIR"` to
    /// distinguish them from Rust primitive types (e.g. `string` → `FHIRString`).
    ///
    /// For nested elements, the struct name is constructed by:
    /// - Splitting the element id on `'.'`.
    /// - Capitalizing each path segment.
    /// - Concatenating the segments.
    /// - Removing the FHIR choice-type marker (`[x]`), if present.
    ///
    /// # Panics
    ///
    /// This function will panic if:
    /// - The root [`StructureDefinition`] does not have an `id`.
    /// - A non-root [`ElementDefinition`] does not have an `id`.
    ///
    /// # Arguments
    ///
    /// * `sd` - The [`StructureDefinition`] containing the element.
    /// * `element` - The [`ElementDefinition`] for which to generate a struct name.
    ///
    /// # Returns
    ///
    /// A `String` containing the generated Rust struct name.
    pub fn struct_name(sd: &StructureDefinition, element: &ElementDefinition) -> String {
        if conditionals::is_root(sd, element) {
            let mut interface_name: String = capitalize(sd.id.as_ref().unwrap());
            if conditionals::is_primitive_sd(sd) {
                interface_name = "FHIR".to_owned() + &interface_name;
            }
            interface_name
        } else {
            element
                .id
                .as_ref()
                .map(|p| p.split('.'))
                .map(|p| p.map(capitalize).collect::<String>())
                .unwrap()
                .replace("[x]", "")
        }
    }

    pub fn type_choice_name(sd: &StructureDefinition, element: &ElementDefinition) -> String {
        let name = struct_name(sd, element);
        name + "TypeChoice"
    }

    pub fn type_choice_variant_name(element: &ElementDefinition, fhir_type: &str) -> String {
        let field_name = extract::field_name(&extract::path(element));
        format!("{:0}{:1}", field_name, capitalize(fhir_type))
    }

    pub fn create_type_choice_variants(element: &ElementDefinition) -> Vec<String> {
        extract::field_types(element)
            .into_iter()
            .map(|fhir_type| type_choice_variant_name(element, fhir_type))
            .collect()
    }
    pub fn create_type_choice_primitive_variants(element: &ElementDefinition) -> Vec<String> {
        extract::field_types(element)
            .into_iter()
            .filter(|fhir_type| FHIR_PRIMITIVES.contains_key(*fhir_type))
            .map(|fhir_type| type_choice_variant_name(element, fhir_type))
            .collect()
    }

    /// Returns the Rust type for a generated struct field.
    ///
    /// The returned type depends on the kind of FHIR element:
    ///
    /// - **Choice elements** (`[x]`) use the generated choice enum.
    /// - **Nested complex elements** use the generated nested struct.
    /// - **All other elements** are mapped from their FHIR type to the
    ///   corresponding Rust type.
    ///
    /// The returned tuple contains:
    /// - The Rust type as a [`TokenStream`].
    /// - A boolean indicating whether the type is a primitive/value type, as
    ///   determined by [`conversion::fhir_type_to_rust_type`].
    ///
    /// # Panics
    ///
    /// This function will panic if a non-choice, non-nested element does not have
    /// a declared FHIR type or if the type code is missing.
    ///
    /// # Arguments
    ///
    /// * `sd` - The [`StructureDefinition`] containing the element.
    /// * `element` - The [`ElementDefinition`] whose field type is being generated.
    /// * `inlined_terminology` - A mapping of FHIR terminology bindings to generated
    ///   Rust types.
    ///
    /// # Returns
    ///
    /// A tuple `(TokenStream, bool)` where:
    /// - `TokenStream` is the generated Rust type.
    /// - `bool` indicates whether the type is a primitive/value type.
    pub fn field_typename<S: ::std::hash::BuildHasher>(
        sd: &StructureDefinition,
        element: &ElementDefinition,
        inlined_terminology: &HashMap<String, String, S>,
    ) -> (TokenStream, bool) {
        if conditionals::is_typechoice(element) {
            let k = format_ident!("{}", type_choice_name(sd, element));
            (
                quote! {
                    #k
                },
                false,
            )
        } else if conditionals::is_nested_complex(element) {
            let k = format_ident!("{}", struct_name(sd, element));
            (
                quote! {
                    #k
                },
                false,
            )
        } else {
            let fhir_type = element.type_.as_ref().unwrap()[0]
                .code
                .as_ref()
                .value
                .as_ref()
                .unwrap();

            conversion::fhir_type_to_rust_type(element, fhir_type, inlined_terminology)
        }
    }
}

pub mod conditionals {
    use haste_fhir_model::r4::generated::{
        resources::StructureDefinition, terminology::StructureDefinitionKind,
        types::ElementDefinition,
    };

    use crate::utilities::{FHIR_PRIMITIVES, RUST_PRIMITIVES, extract};

    pub fn is_root(sd: &StructureDefinition, element: &ElementDefinition) -> bool {
        element.path.value == sd.id
    }

    pub fn is_resource_sd(sd: &StructureDefinition) -> bool {
        sd.kind == StructureDefinitionKind::resource()
    }

    pub fn is_primitive_type(fhir_type: &str) -> bool {
        FHIR_PRIMITIVES.contains_key(fhir_type)
    }

    pub fn is_primitive_element(element: &ElementDefinition) -> bool {
        let types = extract::field_types(element);
        types.len() == 1 && is_primitive_type(types[0])
    }

    pub fn is_nested_complex(element: &ElementDefinition) -> bool {
        let types = extract::field_types(element);
        // Backbone or Typechoice elements Have inlined types created.
        types.len() > 1 || types[0] == "BackboneElement" || types[0] == "Element"
    }

    // All structs should be boxed if they are not rust primitive types.
    pub fn should_be_boxed(fhir_type: &str) -> bool {
        !RUST_PRIMITIVES.contains_key(fhir_type)
    }

    pub fn is_primitive_sd(sd: &StructureDefinition) -> bool {
        sd.kind == StructureDefinitionKind::primitive_type()
    }

    pub fn is_typechoice(element: &ElementDefinition) -> bool {
        extract::field_types(element).len() > 1
    }
}

pub mod load {
    use std::path::Path;

    use haste_fhir_model::r4::generated::{
        resources::{Resource, StructureDefinition},
        terminology::StructureDefinitionKind,
    };

    use crate::utilities::extract;

    /// Loads a FHIR resource from a JSON file.
    ///
    /// The file is read as UTF-8 text and deserialized into a [`Resource`] using
    /// `serde_json`.
    ///
    /// # Arguments
    ///
    /// * `file_path` - The path to the JSON file containing the FHIR resource.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    ///
    /// * The file cannot be read from the provided path.
    /// * The file contents cannot be parsed as valid JSON.
    /// * The parsed JSON does not match the expected [`Resource`] structure.
    ///
    /// The returned error message includes the underlying cause of the failure.
    pub fn load_from_file(file_path: &Path) -> Result<Resource, String> {
        let data =
            std::fs::read_to_string(file_path).map_err(|e| format!("Failed to read file: {e}"))?;

        let resource = serde_json::from_str::<Resource>(&data)
            .map_err(|e| format!("Failed to parse JSON: {e}"))?;

        Ok(resource)
    }

    /// Retrieves [`StructureDefinition`] resources from a FHIR [`Resource`].
    ///
    /// This function supports extracting structure definitions from:
    ///
    /// - A [`Resource::Bundle`], by collecting all entries containing a
    ///   [`Resource::StructureDefinition`].
    /// - A standalone [`Resource::StructureDefinition`].
    ///
    /// The returned definitions can optionally be filtered by their kind using the
    /// `level` parameter:
    ///
    /// - `"resource"` - Includes resource definitions.
    /// - `"complex-type"` - Includes complex type definitions.
    /// - `"primitive-type"` - Includes primitive type definitions.
    ///
    /// If no level filter is provided, all matching [`StructureDefinition`] values
    /// are returned.
    ///
    /// # Arguments
    ///
    /// * `resource` - The FHIR resource containing one or more structure
    ///   definitions.
    /// * `level` - Optional filter specifying the structure definition category to
    ///   return.
    ///
    /// # Returns
    ///
    /// Returns a vector of references to matching [`StructureDefinition`] values.
    ///
    /// # Errors
    ///
    /// This function currently does not return any errors during execution and
    /// returns `Ok` in all cases. The `Result` return type is reserved for future
    /// error handling.
    ///
    /// # Lifetimes
    ///
    /// The returned references borrow from the provided `resource` and are valid
    /// for the same lifetime as the input resource.
    pub fn get_structure_definitions<'a>(
        resource: &'a Resource,
        level: Option<&'static str>,
    ) -> Result<Vec<&'a StructureDefinition>, String> {
        match resource {
            Resource::Bundle(bundle) => {
                if let Some(entries) = bundle.entry.as_ref() {
                    let sds = entries
                        .iter()
                        .filter_map(|e| e.resource.as_ref())
                        .filter_map(|sd| match sd.as_ref() {
                            Resource::StructureDefinition(sd) => Some(sd),
                            _ => None,
                        });

                    let filtered_sds = sds.filter(move |sd| {
                        if let Some(level) = level {
                            match &sd.kind {
                                kind if kind == &StructureDefinitionKind::resource()
                                    || kind == &StructureDefinitionKind::null() =>
                                {
                                    level == "resource"
                                }
                                kind if kind == &StructureDefinitionKind::complex_type() => {
                                    level == "complex-type"
                                }
                                kind if kind == &StructureDefinitionKind::primitive_type() => {
                                    level == "primitive-type"
                                }
                                _ => false,
                            }
                        } else {
                            true
                        }
                    });

                    Ok(filtered_sds.collect())
                } else {
                    Ok(vec![])
                }
            }
            Resource::StructureDefinition(sd) => {
                let resources = std::iter::once(sd);
                let filtered_resources = resources.filter(|sd| {
                    if let Some(level) = level {
                        match &sd.kind {
                            kind if kind == &StructureDefinitionKind::resource()
                                || kind == &StructureDefinitionKind::null() =>
                            {
                                level == "resource"
                            }
                            kind if kind == &StructureDefinitionKind::complex_type() => {
                                level == "complex-type"
                            }
                            kind if kind == &StructureDefinitionKind::primitive_type() => {
                                level == "primitive-type"
                            }
                            _ => false,
                        }
                    } else {
                        true
                    }
                });

                Ok(filtered_resources.collect())
            }
            _ => Ok(vec![]),
        }
    }
}
