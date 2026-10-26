# haste-fhir-generated-ops

Typed Rust bindings for every FHIR operation Haste Health ships: one module per `OperationDefinition` with an `Input` struct, an `Output` struct and the definition itself. The 65 modules cover the FHIR R4 core operations (`ValueSetExpand`, `ValueSetValidateCode`, `CodeSystemLookup`, `PatientEverything`, `ResourceValidate`, `ConceptMapTranslate`, `ObservationLastn`, ...), the R5 Subscriptions backport (`BackportSubscriptionStatus`, `BackportSubscriptionEvents`, ...), SQL on FHIR (`ViewDefinitionRun`) and Haste Health's own operations (`HasteHealthEvaluatePolicy`, `HasteHealthSmartLaunch`, `Hl7v2Parse`, `HasteHealthListScopes`, ...).

The server implements its operations against these types, [`haste-fhir-terminology`](https://crates.io/crates/haste-fhir-terminology) speaks `ValueSetExpand::Input` and `Output`, and the TypeScript package [`@haste-health/generated-ops`](https://www.npmjs.com/package/@haste-health/generated-ops) is generated from the same definitions.

## Install

```bash
cargo add haste-fhir-generated-ops
```

## Usage

Each module exposes `CODE` (the operation's `$code`), `OPERATION_DEFINITION` (a `LazyLock<OperationDefinition>`) and the two structs. `Input` converts both ways between a `Parameters` resource; `Output` parses from one and converts into the resource the operation returns, which for a single `return` parameter is that resource itself:

```rust
use haste_fhir_generated_ops::generated::ResourceValidate;
use haste_fhir_model::r4::generated::resources::{Parameters, Resource};
use haste_fhir_model::r4::generated::types::FHIRUri;

// Build the Parameters body for POST [base]/Patient/$validate.
let input = ResourceValidate::Input {
    resource: Some(patient),
    mode: None,
    profile: Some(FHIRUri {
        value: Some("http://hl7.org/fhir/us/core/StructureDefinition/us-core-patient".to_string()),
        ..Default::default()
    }),
};
let body: Resource = input.into();

// Parse the Parameters a client sent into the typed input.
let input = ResourceValidate::Input::try_from(parameters.parameter.unwrap_or_default())?;
```

To implement an operation on a server, pair a module with `OperationExecutor` from [`haste-fhir-ops`](https://crates.io/crates/haste-fhir-ops), which parses the input, runs your function and converts the output.

Field names follow the `OperationDefinition`, so they are camelCase (`valueSet`, `includeDesignations`); names that are not valid Rust identifiers carry `#[parameter_rename]`, as `return_` for `return` and `type_` for `_type`. Nested parts become `Output...` structs (`OutputMatch` on `PatientMatch`, `OutputErrors` on `HasteHealthIndexingErrors`).

## Generated code

`src/generated.rs` is written by `bash scripts/operation_build.sh` from `backend/`, which reads the operation definitions in `artifacts/r4` (the R5 subscriptions backport, Haste Health's definitions, the HL7 core `profiles-resources` bundle and the SQL on FHIR operations). Do not edit it by hand: change the artifact and regenerate.

## Links

- [API reference](https://rust.haste.health/haste_fhir_generated_ops/)
- [Source](https://github.com/HasteHealth/HasteHealth/tree/main/backend/crates/fhir-generated-ops)
- [Rust SDK overview](https://haste.health/docs/api/sdks/rust) and [invoking operations](https://haste.health/docs/api/rest_api/fhir/endpoints/invoke-type) over the REST API
- Related: [haste-fhir-ops](https://crates.io/crates/haste-fhir-ops), [haste-fhir-model](https://crates.io/crates/haste-fhir-model)
- TypeScript equivalent: [@haste-health/generated-ops](https://www.npmjs.com/package/@haste-health/generated-ops)

Part of [Haste Health](https://haste.health), licensed under [Apache-2.0](https://github.com/HasteHealth/HasteHealth/blob/main/LICENSE).
