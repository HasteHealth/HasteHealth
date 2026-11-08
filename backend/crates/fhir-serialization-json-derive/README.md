# haste-fhir-serialization-json-derive

The procedural macros behind [`haste-fhir-serialization-json`](https://crates.io/crates/haste-fhir-serialization-json): `FHIRSerdeSerialize` and `FHIRSerdeDeserialize`. They implement serde's `Serialize` and `Deserialize` for a struct or enum following the FHIR JSON rules, which plain serde derives do not know: `_field` entries for primitive `id` and `extension`, `valueX` names for choice elements, `resourceType` on resources, bare strings for bound codes, and cardinality and reference target checks on read.

You do not normally depend on this crate directly. Enable the `derive` feature of `haste-fhir-serialization-json` and import the macros from its `derive` module, which keeps the macro and the runtime crate at matching versions.

## Install

```bash
cargo add haste-fhir-serialization-json --features derive
```

## Usage

```rust
use haste_fhir_serialization_json::derive::{FHIRSerdeDeserialize, FHIRSerdeSerialize};

#[derive(FHIRSerdeSerialize, FHIRSerdeDeserialize)]
#[fhir_serialize_type = "complex"]
pub struct Period {
    #[primitive]
    pub start: Option<Box<FHIRDateTime>>,
    #[primitive]
    pub end: Option<Box<FHIRDateTime>>,
}
```

`fhir_serialize_type` selects the implementation: `primitive`, `valueset`, `typechoice`, `complex`, `resource` or (serialize only) `enum-variant`. The field attributes `primitive`, `rename_field`, `type_choice_field_name`, `type_choice_variants`, `cardinality`, `reference`, `determine_by` and `fhir_resource_type` are documented on the parent crate.

## Links

- [API reference](https://rust.haste.health/haste_fhir_serialization_json_derive/)
- [Source](https://github.com/HasteHealth/HasteHealth/tree/main/backend/crates/fhir-serialization-json-derive)
- [Rust SDK overview](https://haste.health/docs/api/sdks/rust)
- Parent crate: [haste-fhir-serialization-json](https://crates.io/crates/haste-fhir-serialization-json)

Part of [Haste Health](https://haste.health), licensed under [Apache-2.0](https://github.com/HasteHealth/HasteHealth/blob/main/LICENSE).
