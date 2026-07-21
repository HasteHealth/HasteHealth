mod deserialize;
mod serde_deserialize;
mod serde_serialize;
mod serialize;
mod utilities;

use proc_macro::TokenStream;
use syn::{Attribute, DeriveInput, Expr, Lit, Meta, parse_macro_input};

/// Determines the de/serialization type of the derive macro.
fn get_attribute_serialization_type(attrs: &[Attribute]) -> Option<String> {
    attrs.iter().find_map(|attr| match &attr.meta {
        Meta::NameValue(name_value) => {
            if name_value.path.is_ident("fhir_serialize_type") {
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

/// Derives FHIR JSON serialization implementations.
///
/// The macro behavior is selected using the `fhir_serialize_type` attribute.
///
/// Supported serialization types:
/// - `primitive`
/// - `typechoice`
/// - `complex`
/// - `resource`
/// - `valueset`
/// - `enum-variant`
///
/// # Panics
///
/// Panics if:
/// - The `fhir_serialize_type` attribute is missing.
/// - The `fhir_serialize_type` attribute contains an unsupported value.
///
/// # Attributes
///
/// This macro supports the following attributes:
/// - `fhir_serialize_type` - Selects the serialization implementation.
/// - `rename_field` - Renames a serialized field.
/// - `type_choice_field_name` - Defines the field name used for type choices.
/// - `type_choice_variants` - Defines variants for type choice fields.
/// - `primitive` - Marks primitive FHIR fields.
/// - `code` - Marks code fields.
/// - `cardinality` - Defines validation cardinality constraints.
/// - `reference` - Marks reference fields.
/// - `fhir_resource_type` - Overrides the FHIR resource type name.
#[proc_macro_derive(
    FHIRJSONSerialize,
    attributes(
        fhir_serialize_type,
        rename_field,
        // Used on the enum itself for typechoice.
        type_choice_field_name,
         // Used on field itself for variants.
        type_choice_variants,
        primitive,
        code,
        // For validation on vector min maxes.
        cardinality,
        reference,
        // if resourcetype doesn't correlate to struct name
        fhir_resource_type
    )
)]
pub fn serialize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let serialize_type = get_attribute_serialization_type(&input.attrs);

    match serialize_type.as_deref() {
        Some("primitive") => serialize::primitve_serialization(input),
        Some("typechoice") => serialize::typechoice_serialization(input),
        Some("complex") => {
            serialize::complex_serialization(input, serialize::ComplexSerializeType::Complex)
        }
        Some("resource") => {
            serialize::complex_serialization(input, serialize::ComplexSerializeType::Resource)
        }
        Some("valueset") => serialize::value_set_serialization(input),
        Some("enum-variant") => serialize::enum_variant_serialization(input),
        // Some("typechoice") => typechoice_serialization(input),
        None => panic!("Missing serialization type attribute"),
        _ => panic!(
            "Must be one of primitive, typechoice, complex, resource, valueset or enum-variant."
        ),
    }
}

#[derive(Clone, Copy, PartialEq)]
enum DeserializeComplexType {
    Complex,
    Resource,
}

/// Derives FHIR JSON deserialization implementations.
///
/// The macro behavior is selected using the `fhir_serialize_type` attribute.
///
/// Supported deserialization types:
/// - `primitive`
/// - `typechoice`
/// - `complex`
/// - `resource`
/// - `valueset`
/// - `enum-variant`
///
/// # Panics
///
/// Panics if:
/// - The `fhir_serialize_type` attribute is missing.
/// - The `fhir_serialize_type` attribute contains an unsupported value.
///
/// # Attributes
///
/// This macro supports the following attributes:
/// - `fhir_serialize_type` - Selects the deserialization implementation.
/// - `rename_field` - Renames a deserialized field.
/// - `type_choice_field_name` - Defines the field name used for type choices.
/// - `type_choice_variants` - Defines variants for type choice fields.
/// - `primitive` - Marks primitive FHIR fields.
/// - `determine_by` - Specifies how enum variants are determined during deserialization.
/// - `cardinality` - Defines validation cardinality constraints.
/// - `reference` - Marks reference fields.
/// - `fhir_resource_type` - Overrides the FHIR resource type name.
#[proc_macro_derive(
    FHIRJSONDeserialize,
    attributes(
        fhir_serialize_type,
        rename_field,

        // Used on the enum itself for typechoice.
        type_choice_field_name,

        // Used on field itself for variants.
        type_choice_variants,

        primitive,

        // Used for enum serialization.
        determine_by,

        // For validation on vector min maxes.
        cardinality,
        reference,

        // if resourcetype doesn't correlate to struct name
        fhir_resource_type
    )
)]
pub fn deserialize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let serialize_type = get_attribute_serialization_type(&input.attrs);

    match serialize_type.as_deref() {
        Some("primitive") => deserialize::fhir_primitive_deserialization(input),
        Some("typechoice") => deserialize::deserialize_typechoice(input),
        Some("resource") => {
            deserialize::deserialize_complex(input, DeserializeComplexType::Resource)
        }
        Some("complex") => deserialize::deserialize_complex(input, DeserializeComplexType::Complex),
        Some("enum-variant") => deserialize::enum_variant_deserialization(input),
        Some("valueset") => deserialize::deserialize_valueset(input),
        None => panic!("Missing deserialization type attribute"),
        _ => panic!(
            "Must be one of primitive, typechoice, complex, resource, valueset or enum-variant."
        ),
    }
    .into()
}

/// Derives Serde-based FHIR JSON deserialization implementations.
///
/// The macro behavior is selected using the `fhir_serialize_type` attribute.
///
/// Supported deserialization types:
/// - `primitive`
/// - `valueset`
/// - `typechoice`
/// - `complex`
/// - `resource`
///
/// # Panics
///
/// Panics if:
/// - The `fhir_serialize_type` attribute is missing.
/// - The `fhir_serialize_type` attribute contains an unsupported value.
/// - A deserialization type other than the supported Serde deserialization
///   variants is specified.
///
/// # Attributes
///
/// This macro supports the following attributes:
/// - `fhir_serialize_type` - Selects the deserialization implementation.
/// - `rename_field` - Renames a deserialized field.
/// - `type_choice_field_name` - Defines the field name used for type choice
///   deserialization.
/// - `type_choice_variants` - Defines variants for type choice fields.
/// - `primitive` - Marks primitive FHIR fields.
/// - `determine_by` - Specifies how enum variants are determined during
///   deserialization.
/// - `cardinality` - Defines validation cardinality constraints.
/// - `reference` - Marks reference fields.
/// - `fhir_resource_type` - Overrides the FHIR resource type name.
#[proc_macro_derive(
    FHIRSerdeDeserialize,
    attributes(
        fhir_serialize_type,
        rename_field,

        // Used on the enum itself for typechoice.
        type_choice_field_name,

        // Used on field itself for variants.
        type_choice_variants,

        primitive,

        // Used for enum serialization.
        determine_by,

        // For validation on vector min maxes.
        cardinality,
        reference,

        // if resourcetype doesn't correlate to struct name
        fhir_resource_type
    )
)]
pub fn serde_deserialize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let serialize_type = get_attribute_serialization_type(&input.attrs);

    match serialize_type.as_deref() {
        Some("primitive") => serde_deserialize::fhir_primitive_deserialization(input),
        Some("valueset") => serde_deserialize::valueset_deserialization(input),
        Some("typechoice") => serde_deserialize::typechoice_deserialization(input),
        Some("complex") => {
            serde_deserialize::complex_deserialization(input, DeserializeComplexType::Complex)
        }
        Some("resource") => {
            serde_deserialize::complex_deserialization(input, DeserializeComplexType::Resource)
        }
        None => panic!("Missing deserialization type attribute"),
        _ => panic!("Only primitive and valueset supported for serde deserialization."),
    }
}

/// Derives Serde-based FHIR JSON serialization implementations.
///
/// The macro behavior is selected using the `fhir_serialize_type` attribute.
///
/// Supported serialization types:
/// - `primitive`
/// - `valueset`
/// - `typechoice`
/// - `complex`
/// - `resource`
/// - `enum-variant`
///
/// # Panics
///
/// Panics if:
/// - The `fhir_serialize_type` attribute is missing.
/// - The `fhir_serialize_type` attribute contains an unsupported value.
///
/// # Attributes
///
/// This macro supports the following attributes:
/// - `fhir_serialize_type` - Selects the serialization implementation.
/// - `rename_field` - Renames a serialized field.
/// - `type_choice_field_name` - Defines the field name used for type choice
///   serialization.
/// - `type_choice_variants` - Defines variants for type choice fields.
/// - `primitive` - Marks primitive FHIR fields.
/// - `code` - Marks code fields.
/// - `cardinality` - Defines validation cardinality constraints.
/// - `reference` - Marks reference fields.
/// - `fhir_resource_type` - Overrides the FHIR resource type name.
#[proc_macro_derive(
    FHIRSerdeSerialize,
    attributes(
        fhir_serialize_type,
        rename_field,
        // Used on the enum itself for typechoice.
        type_choice_field_name,
         // Used on field itself for variants.
        type_choice_variants,
        primitive,
        code,
        // For validation on vector min maxes.
        cardinality,
        reference,

        // if resourcetype doesn't correlate to struct name
        fhir_resource_type
    )
)]
pub fn serde_serialize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let serialize_type = get_attribute_serialization_type(&input.attrs);

    match serialize_type.as_deref() {
        Some("primitive") => serde_serialize::fhir_primitive_serialization(input),
        Some("valueset") => serde_serialize::valueset_serialization(input),
        Some("typechoice") => serde_serialize::typechoice_serialization(input),
        Some("complex") => {
            serde_serialize::complex_serialization(input, DeserializeComplexType::Complex)
        }
        Some("resource") => {
            serde_serialize::complex_serialization(input, DeserializeComplexType::Resource)
        }
        Some("enum-variant") => serde_serialize::enum_variant_serialization(input),
        None => panic!("Missing serialization type attribute"),
        _ => panic!("Only primitive and valueset supported for serde deserialization."),
    }
}
