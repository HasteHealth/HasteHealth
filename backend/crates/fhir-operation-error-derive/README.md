# haste-fhir-operation-error-derive

The procedural macro behind [`haste-fhir-operation-error`](https://crates.io/crates/haste-fhir-operation-error): `#[derive(OperationOutcomeError)]` for an enum, with the `fatal`, `error`, `warning`, `information` and `from` attributes. It generates `From<YourEnum> for OperationOutcomeError`, building one `OperationOutcome` issue per attribute, and `From<Inner> for YourEnum` for each `#[from]` field.

You do not depend on this crate directly. Enable the `derive` feature of `haste-fhir-operation-error` and import the macro from `haste_fhir_operation_error::derive`.

## Install

```bash
cargo add haste-fhir-operation-error --features derive
```

## Usage

```rust
use haste_fhir_operation_error::{OperationOutcomeError, derive::OperationOutcomeError};

#[derive(Debug, OperationOutcomeError)]
pub enum LookupError {
    #[error(code = "not-found", diagnostic = "No CodeSystem at '{arg0}'.")]
    Missing(String),
    #[error(code = "not-supported", diagnostic = "Lookup is not supported.")]
    NotSupported,
}
```

`code` must be a code from the FHIR `IssueType` value set; the macro checks it at compile time. See the `haste-fhir-operation-error` README for the HTTP status each code maps to. Variants are unit or tuple variants, and a `#[from]` variant has that one field.

The generated `From` is `#[track_caller]`, so an error converted with `?` reports that `?` as its origin.

## Links

- [API reference](https://rust.haste.health/haste_fhir_operation_error_derive/)
- [Source](https://github.com/HasteHealth/HasteHealth/tree/main/backend/crates/fhir-operation-error-derive)
- [Rust SDK overview](https://haste.health/docs/api/sdks/rust)
- Parent crate: [haste-fhir-operation-error](https://crates.io/crates/haste-fhir-operation-error)

Part of [Haste Health](https://haste.health), licensed under [Apache-2.0](https://github.com/HasteHealth/HasteHealth/blob/main/LICENSE).
