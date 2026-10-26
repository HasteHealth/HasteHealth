# haste-fhir-ops

Building blocks for implementing FHIR operations (`$everything`, `$expand`, your own `$custom-op`) in Rust. An operation is a struct for its input parameters, a struct for its output and an async function between them; this crate turns a FHIR `Parameters` resource into the input, runs the function and turns the output back into a resource.

[`haste-fhir-generated-ops`](https://crates.io/crates/haste-fhir-generated-ops) uses the derives here for every `OperationDefinition` shipped with Haste Health, and the server registers each operation it implements as an `OperationExecutor`.

## Install

```bash
cargo add haste-fhir-ops --features derive
```

## Usage

Implement a generated operation. The closure gets the caller's context, the tenant and project ids, the `InvocationRequest` (to read the instance id or resource type) and the parsed input, and returns a boxed future:

```rust
use haste_fhir_generated_ops::generated::PatientEverything;
use haste_fhir_ops::{OperationExecutor, OperationInvocation};

let everything = OperationExecutor::new(
    &PatientEverything::OPERATION_DEFINITION,
    Box::new(|ctx: MyCtx, _tenant, _project, request, input: PatientEverything::Input| {
        let request = request.clone();
        Box::pin(async move {
            let bundle = collect_everything(&ctx, &request, input.since).await?;
            Ok(PatientEverything::Output { return_: bundle })
        })
    }),
);

// Dispatch: `defines` checks the code, the level (system, type or instance) and the resource type.
if haste_fhir_ops::defines(everything.operation_definition(), &request) {
    let resource = everything.execute(ctx, tenant, project, &request).await?;
}
```

Describe your own operation's parameters with the derives. `Option` fields are optional, `Vec` fields repeat, FHIR primitive types become `value[x]`, resource types become `resource`, and `#[parameter_nested]` structs become `part`:

```rust
use haste_fhir_model::r4::generated::resources::{Bundle, ParametersParameter, Resource};
use haste_fhir_model::r4::generated::types::{FHIRCode, FHIRUri};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_ops::derive::{FromParameters, ToParameters};

#[derive(Debug, FromParameters, ToParameters)]
pub struct Input {
    pub profile: Option<FHIRUri>,
    #[parameter_rename = "_type"]
    pub type_: Option<Vec<FHIRCode>>,
}

#[derive(Debug, FromParameters)]
pub struct Output {
    #[parameter_rename = "return"]
    pub return_: Bundle,
}
```

`FromParameters` implements `TryFrom<Vec<ParametersParameter>>` and rejects unknown names, repeated single-valued parameters, wrong value types and missing required fields with an `OperationOutcomeError`. `ToParameters` implements `Into<Vec<ParametersParameter>>`. The generated code refers to `ParametersParameter`, `Resource` and `OperationOutcomeError` unqualified, so import them as above.

## Features

| Feature  | What it enables                                                                                                       |
| -------- | --------------------------------------------------------------------------------------------------------------------- |
| `derive` | `derive::{FromParameters, ToParameters}` from [haste-fhir-ops-derive](https://crates.io/crates/haste-fhir-ops-derive) |

## API

- `OperationInvocation<CTX>`: `execute(ctx, tenant, project, &InvocationRequest) -> Resource` and `operation_definition()`.
- `OperationExecutor<CTX, I, O>`: the standard implementation over a parsed input `I` and an output `O`.
- `defines(definition, request)` and `overlaps(a, b)`: match an invocation to a definition, and detect two definitions that claim the same invocation.
- `Param<T>`: an input given either as the typed struct or as a raw `Parameters` resource, with `as_parameters()`.

## Links

- [API reference](https://rust.haste.health/haste_fhir_ops/)
- [Source](https://github.com/HasteHealth/HasteHealth/tree/main/backend/crates/fhir-ops)
- [Rust SDK overview](https://haste.health/docs/api/sdks/rust) and [invoking operations](https://haste.health/docs/api/rest_api/fhir/endpoints/invoke-type) over the REST API
- Related: [haste-fhir-generated-ops](https://crates.io/crates/haste-fhir-generated-ops), [haste-fhir-client](https://crates.io/crates/haste-fhir-client)
- TypeScript equivalent: [@haste-health/operation-execution](https://www.npmjs.com/package/@haste-health/operation-execution)

Part of [Haste Health](https://haste.health), licensed under [Apache-2.0](https://github.com/HasteHealth/HasteHealth/blob/main/LICENSE).
