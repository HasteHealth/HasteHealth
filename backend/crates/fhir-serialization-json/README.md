# haste-fhir-serialization-json

FHIR JSON serialization for Rust structs. FHIR JSON differs from what a plain serde derive produces: a primitive's `id` and `extension` live in a sibling `_field` entry (with `null` placeholders inside arrays), choice elements are written as `valueString` or `valueQuantity` rather than a nested object, resources carry `resourceType`, bound codes are bare strings, and empty values are dropped. The `FHIRSerdeSerialize` and `FHIRSerdeDeserialize` derive macros (feature `derive`) implement serde's `Serialize` and `Deserialize` with those rules, and `errors::DeserializeError` names what went wrong.

[`haste-fhir-model`](https://crates.io/crates/haste-fhir-model) derives every generated type with these macros, so `serde_json::to_string(&patient)` already produces FHIR JSON and most users never depend on this crate directly. Reach for it when you declare your own FHIR shaped structs, or to match on the error variants.

## Install

```bash
cargo add haste-fhir-serialization-json --features derive
```

## Usage

This is how `haste-fhir-model` declares `Patient` (abridged). The attributes tell the macros how each field appears in JSON.

```rust
use haste_fhir_serialization_json::derive::{FHIRSerdeDeserialize, FHIRSerdeSerialize};

#[derive(Clone, Debug, Default, Reflect, FHIRSerdeSerialize, FHIRSerdeDeserialize)]
#[fhir_type = "Patient"]
#[fhir_serialize_type = "resource"]
pub struct Patient {
    pub id: Option<String>,
    #[primitive]
    pub active: Option<Box<FHIRBoolean>>,
    #[primitive]
    pub gender: Option<BoundCode<AdministrativeGender>>,
    #[type_choice_variants(complex = [], primitive = ["deceasedBoolean", "deceasedDateTime"])]
    pub deceased: Option<PatientDeceasedTypeChoice>,
    #[reference(targets = ["Organization"])]
    pub managingOrganization: Option<Box<Reference>>,
    // ...
}
```

With that in place serde round trips FHIR JSON, extensions included:

```rust
let patient: Patient = serde_json::from_str(
    r#"{ "resourceType": "Patient", "active": true, "_active": { "id": "a1" }, "deceasedBoolean": false }"#,
)?;
assert_eq!(patient.active.as_ref().unwrap().id.as_deref(), Some("a1"));
```

## Attributes

| Attribute                                        | Meaning                                                                                                                           |
| ------------------------------------------------ | --------------------------------------------------------------------------------------------------------------------------------- |
| `fhir_serialize_type`                            | Which implementation to generate: `primitive`, `valueset`, `typechoice`, `complex`, `resource` or `enum-variant` (serialize only) |
| `primitive`                                      | The field is a FHIR primitive: its `id` and `extension` are written to and read from `_<field>`                                   |
| `type_choice_field_name`, `type_choice_variants` | A `value[x]` element: the enum's base name and the concrete `valueX` names it accepts                                             |
| `rename_field`                                   | The JSON name when the Rust name differs (`use_` for `use`)                                                                       |
| `cardinality`                                    | Minimum and maximum length of a repeating field, checked on read                                                                  |
| `reference`                                      | The resource types a `Reference` may point at, checked on read                                                                    |
| `fhir_resource_type`                             | The `resourceType` value when it is not the struct name                                                                           |
| `determine_by`                                   | How an enum variant is chosen while deserializing                                                                                 |

Errors are reported as `DeserializeError` variants: `UnknownField`, `MissingRequiredField`, `InvalidType`, `InvalidResourceType`, `InvalidEnumVariant`, `InvalidTypeChoiceVariant`, `DuplicateTypeChoiceVariant`, `CardinalityViolation` and `ReferenceTargetValidationFailed`, alongside the underlying `serde_json::Error`.

## Features

| Feature  | What it enables                                                                              |
| -------- | -------------------------------------------------------------------------------------------- |
| `derive` | The `derive` module, re-exporting the two macros from `haste-fhir-serialization-json-derive` |

## Links

- [API reference](https://rust.haste.health/haste_fhir_serialization_json/)
- [Source](https://github.com/HasteHealth/HasteHealth/tree/main/backend/crates/fhir-serialization-json)
- [Rust SDK overview](https://haste.health/docs/api/sdks/rust)
- Related crates: [haste-fhir-model](https://crates.io/crates/haste-fhir-model), [haste-fhir-serialization-json-derive](https://crates.io/crates/haste-fhir-serialization-json-derive)

Part of [Haste Health](https://haste.health), licensed under [Apache-2.0](https://github.com/HasteHealth/HasteHealth/blob/main/LICENSE).
