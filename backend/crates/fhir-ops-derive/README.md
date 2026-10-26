# haste-fhir-ops-derive

The procedural macros behind [`haste-fhir-ops`](https://crates.io/crates/haste-fhir-ops): `#[derive(FromParameters)]` and `#[derive(ToParameters)]`, which map a struct to and from the `parameter` list of a FHIR `Parameters` resource, with the `#[parameter_rename = "..."]` and `#[parameter_nested]` field attributes.

You do not depend on this crate directly. Enable the `derive` feature of `haste-fhir-ops` and import the macros from `haste_fhir_ops::derive`.

## Install

```bash
cargo add haste-fhir-ops --features derive
```

## Usage

```rust
use haste_fhir_model::r4::generated::resources::{ParametersParameter, Resource};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_ops::derive::{FromParameters, ToParameters};

#[derive(Debug, FromParameters, ToParameters)]
pub struct Input {
    pub code: haste_fhir_model::r4::generated::types::FHIRCode,
    pub system: Option<haste_fhir_model::r4::generated::types::FHIRUri>,
}
```

See the `haste-fhir-ops` README for how field types map onto `value[x]`, `resource` and `part`, and which inputs the generated `TryFrom` rejects.

## Links

- [API reference](https://rust.haste.health/haste_fhir_ops_derive/)
- [Source](https://github.com/HasteHealth/HasteHealth/tree/main/backend/crates/fhir-ops-derive)
- [Rust SDK overview](https://haste.health/docs/api/sdks/rust)
- Parent crate: [haste-fhir-ops](https://crates.io/crates/haste-fhir-ops)

Part of [Haste Health](https://haste.health), licensed under [Apache-2.0](https://github.com/HasteHealth/HasteHealth/blob/main/LICENSE).
