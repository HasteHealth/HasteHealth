# haste-fhir-operation-error

The error type of the Haste Health Rust stack. An `OperationOutcomeError` wraps a FHIR `OperationOutcome` (one or more issues with a severity, an issue type code and diagnostics) plus an optional source error, so every failure already has the shape FHIR clients expect. With the `axum` feature it also knows which HTTP status to answer with.

Almost every Haste Health crate returns it, from [`haste-fhir-client`](https://crates.io/crates/haste-fhir-client) and [`haste-fhirpath`](https://crates.io/crates/haste-fhirpath) through to the server and worker binaries.

## Install

```bash
cargo add haste-fhir-operation-error --features derive
```

## Usage

Build one directly, with a code from the FHIR `IssueType` value set:

```rust
use haste_fhir_model::r4::generated::terminology::IssueType;
use haste_fhir_operation_error::OperationOutcomeError;

let missing = OperationOutcomeError::error(
    IssueType::not_found(),
    "Patient/123 does not exist.".to_string(),
);
```

Or derive the conversion for your own error enum. Each variant carries one or more issue attributes (`fatal`, `error`, `warning`, `information`) whose `diagnostic` is a format string over the variant's fields as `{arg0}`, `{arg1}`. A `#[from]` field keeps the wrapped error as the source and generates `From<ThatError>` for the enum:

```rust
use haste_fhir_operation_error::{OperationOutcomeError, derive::OperationOutcomeError};

#[derive(Debug, OperationOutcomeError)]
pub enum ImportError {
    #[error(code = "invalid", diagnostic = "Line {arg0} is not valid FHIR JSON.")]
    BadLine(usize),
    #[fatal(code = "exception", diagnostic = "Could not read the import file.")]
    Io(#[from] std::io::Error),
}

fn import(path: &str) -> Result<(), OperationOutcomeError> {
    let text = std::fs::read_to_string(path).map_err(ImportError::from)?;
    // ...
    Ok(())
}
```

Under the `axum` feature, `status()` maps the first issue's code to a status (`invalid`, `structure`, `required`, `value` and `invariant` to 400, `not-found` to 404, `login`, `unknown` and `expired` to 401, `forbidden`, `security` and `suppressed` to 403, `conflict` to 409, `throttled` to 429, `business-rule` to 405, `not-supported` to 501, anything else to 500) and `IntoResponse` writes the `OperationOutcome` as `application/fhir+json`. `OperationOutcomeError::unprocessable(outcome)` forces 422 for a well formed resource that fails profile validation.

## Features

| Feature  | What it enables                                                                                                                      |
| -------- | ------------------------------------------------------------------------------------------------------------------------------------ |
| `derive` | `derive::OperationOutcomeError` from [haste-fhir-operation-error-derive](https://crates.io/crates/haste-fhir-operation-error-derive) |
| `axum`   | `status()` and `impl IntoResponse`; the error is also attached to the response extensions for logging middleware                     |

## API

- Constructors: `new(source, outcome)`, `fatal`, `error`, `warning`, `information` (each takes an `IssueType` code and a diagnostic), `unprocessable(outcome)`.
- `outcome()` for the `OperationOutcome`, `push_issue` to add to it, `backtrace()` from the source error.
- Implements `std::error::Error` and `Display` (the issues' diagnostics joined with commas).

## Links

- [API reference](https://rust.haste.health/haste_fhir_operation_error/)
- [Source](https://github.com/HasteHealth/HasteHealth/tree/main/backend/crates/fhir-operation-error)
- [Rust SDK overview](https://haste.health/docs/api/sdks/rust)
- Related: [haste-fhir-model](https://crates.io/crates/haste-fhir-model) for `OperationOutcome` and `IssueType`
- TypeScript equivalent: [@haste-health/operation-outcomes](https://www.npmjs.com/package/@haste-health/operation-outcomes)

Part of [Haste Health](https://haste.health), licensed under [Apache-2.0](https://github.com/HasteHealth/HasteHealth/blob/main/LICENSE).
