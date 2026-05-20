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
        reference
    )
)]
pub fn serialize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let serialize_type = get_attribute_serialization_type(&input.attrs);

    let result = match serialize_type.unwrap().as_str() {
        "primitive" => serialize::primitve_serialization(input),
        "typechoice" => serialize::typechoice_serialization(input),
        "complex" => {
            serialize::complex_serialization(input, serialize::ComplexSerializeType::Complex)
        }
        "resource" => {
            serialize::complex_serialization(input, serialize::ComplexSerializeType::Resource)
        }
        "valueset" => serialize::value_set_serialization(input),
        "enum-variant" => serialize::enum_variant_serialization(input),
        // Some("typechoice") => typechoice_serialization(input),
        _ => panic!("Must be one of primitive, typechoice, complex or resource."),
    };

    result
}

#[derive(PartialEq)]
enum DeserializeComplexType {
    Complex,
    Resource,
}

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
        reference
    )
)]

pub fn deserialize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let serialize_type = get_attribute_serialization_type(&input.attrs);

    let result = match serialize_type.unwrap().as_str() {
        "primitive" => deserialize::fhir_primitive_deserialization(input),
        "typechoice" => deserialize::deserialize_typechoice(input),
        "resource" => deserialize::deserialize_complex(input, DeserializeComplexType::Resource),
        "complex" => deserialize::deserialize_complex(input, DeserializeComplexType::Complex),
        "enum-variant" => deserialize::enum_variant_deserialization(input),
        "valueset" => deserialize::deserialize_valueset(input),
        _ => panic!("Must be one of primitive, typechoice, complex or resource."),
    };

    result.into()
}

#[proc_macro_derive(
    FHIRSerdeDeserialize,
    attributes(fhir_serialize_type, rename_field, type_choice_variants, determine_by)
)]
pub fn serde_deserialize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let serialize_type = get_attribute_serialization_type(&input.attrs);

    let result = match serialize_type.unwrap().as_str() {
        "primitive" => serde_deserialize::fhir_primitive_deserialization(input),
        "valueset" => serde_deserialize::valueset_deserialization(input),
        "typechoice" => serde_deserialize::typechoice_deserialization(input),
        "complex" => {
            serde_deserialize::complex_deserialization(input, DeserializeComplexType::Complex)
        }
        "resource" => {
            serde_deserialize::complex_deserialization(input, DeserializeComplexType::Resource)
        }
        _ => panic!("Only primitive and valueset supported for serde deserialization."),
    };

    result.into()
}

#[proc_macro_derive(
    FHIRSerdeSerialize,
    attributes(fhir_serialize_type, rename_field, type_choice_variants, determine_by)
)]
pub fn serde_serialize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let serialize_type = get_attribute_serialization_type(&input.attrs);

    let result = match serialize_type.unwrap().as_str() {
        "primitive" => serde_serialize::fhir_primitive_serialization(input),
        "valueset" => serde_serialize::valueset_serialization(input),
        "typechoice" => serde_serialize::typechoice_serialization(input),
        "complex" => serde_serialize::complex_serialization(input, DeserializeComplexType::Complex),
        "resource" => {
            serde_serialize::complex_serialization(input, DeserializeComplexType::Resource)
        }
        "enum-variant" => serde_serialize::enum_variant_serialization(input),
        _ => panic!("Only primitive and valueset supported for serde deserialization."),
    };

    result.into()
}
