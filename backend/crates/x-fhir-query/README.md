# haste-x-fhir-query

Evaluates [x-fhir-query](https://build.fhir.org/fhir-xquery.html) templates: FHIR search URLs with embedded `{{ FHIRPath }}` expressions. Each expression is evaluated with [`haste-fhirpath`](https://crates.io/crates/haste-fhirpath) against the values you pass in, and the results replace the placeholder as a comma separated list.

The Haste Health server uses it in [`haste-access-control`](https://crates.io/crates/haste-access-control), where access policy rules carry x-fhir-query templates that are resolved against the request and the signed-in user before the search runs.

## Install

```bash
cargo add haste-x-fhir-query
```

## Usage

```rust
use std::sync::Arc;

use haste_fhir_model::r4::generated::resources::Patient;
use haste_fhirpath::Config;

let patient = Patient {
    id: Some("example".to_string()),
    ..Default::default()
};

let query = haste_x_fhir_query::evaluation(
    "Observation?subject=Patient/{{$this.id}}",
    vec![&patient],
    Arc::new(Config::default()),
)
.await?;

assert_eq!(query, "Observation?subject=Patient/example");
```

`evaluation` takes the template, the values `$this` resolves to (anything implementing `haste_reflect::MetaValue`, which every generated FHIR type does) and a FHIRPath `Config` for variables and external constants. An expression that yields several values becomes a comma separated list, which FHIR search reads as OR.

Errors are `OperationOutcomeError`s from [`haste-fhir-operation-error`](https://crates.io/crates/haste-fhir-operation-error): an empty expression, a FHIRPath evaluation failure, or a result that has no string form. They convert directly into a FHIR `OperationOutcome` response.

## Links

- [API reference](https://rust.haste.health/haste_x_fhir_query/)
- [Source](https://github.com/HasteHealth/HasteHealth/tree/main/backend/crates/x-fhir-query)
- [Rust SDK overview](https://haste.health/docs/api/sdks/rust)
- TypeScript equivalent: [@haste-health/x-fhir-query](https://www.npmjs.com/package/@haste-health/x-fhir-query)

Part of [Haste Health](https://haste.health), licensed under [Apache-2.0](https://github.com/HasteHealth/HasteHealth/blob/main/LICENSE).
