# @haste-health/operation-execution

Turns a FHIR [OperationDefinition](https://hl7.org/fhir/R4/operationdefinition.html) into a typed, validating operation object. An `Operation<Input, Output>` converts between the `Parameters` resource FHIR sends over the wire and a plain object keyed by parameter name, and validates both directions against the definition with [`@haste-health/fhir-validation`](https://www.npmjs.com/package/@haste-health/fhir-validation).

[`@haste-health/generated-ops`](https://www.npmjs.com/package/@haste-health/generated-ops) generates one `Operation` per OperationDefinition that Haste Health ships, and [`@haste-health/client`](https://www.npmjs.com/package/@haste-health/client) accepts any `IOperation` in `invoke_system`, `invoke_type` and `invoke_instance`, so operation calls are type-checked end to end.

## Install

```bash
npm install @haste-health/operation-execution
```

## Usage

```ts
import { Operation, OpCTX } from "@haste-health/operation-execution";
import type { ValueSet, uri } from "@haste-health/fhir-types/r4/types";

type Input = { url: uri; filter?: string };
type Output = ValueSet;

// valueSetExpandDefinition is the OperationDefinition resource for ValueSet/$expand.
const expand = new Operation<Input, Output>(valueSetExpandDefinition);

// Plain object -> Parameters resource, following the definition's "in" parameters.
const parameters = expand.parseToParameters("in", {
  url: "http://hl7.org/fhir/ValueSet/administrative-gender" as uri,
});
// { resourceType: "Parameters", parameter: [{ name: "url", valueUri: "..." }] }

// Parameters resource -> plain object. "out" does the same for responses.
const input = expand.parseToObject("in", parameters);

// Validate input or output against the definition. Issues are OperationOutcome issues.
const issues = await expand.validate(ctx, "in", input);
```

`ctx` is an `OpCTX`: the `ValidationCTX` of `@haste-health/fhir-validation` (`fhirVersion` and `resolveCanonical`, which looks up StructureDefinitions by URL) plus `level`, one of `"system"`, `"type"` or `"instance"`.

## Behaviour

- Parameters with `max: "*"` become arrays, single-valued ones scalars; nested `part` parameters become nested objects.
- Resource-typed parameters (`Any`, `Resource` or a resource type) travel in `parameter.resource`; everything else in the matching `value[Type]` field.
- An operation whose only output is a required `return` resource is passed through as that resource rather than wrapped in `Parameters`, and an operation with exactly one resource input accepts that resource directly as the request body, as the FHIR specification allows.
- Unknown parameter names, missing required parameters and cardinality overflows throw an `OperationError` from [`@haste-health/operation-outcomes`](https://www.npmjs.com/package/@haste-health/operation-outcomes).

## API

| Export                                                                                         | Purpose                                                                                                                    |
| ---------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------- |
| `Operation<I, O>`, `IOperation<I, O>`                                                          | The class and the interface it implements: `code`, `operationDefinition`, `parseToObject`, `parseToParameters`, `validate` |
| `parseParameters(definition, use, parameters)`, `toParametersResource(definition, use, value)` | The converters behind `Operation`, usable on their own                                                                     |
| `OPMetadata<Op>`                                                                               | Extracts `{ Input, Output }` from an operation type; the client's `invoke_*` signatures are built on it                    |
| `Invocation`, `Executor<CTX, I, O>`, `OpCTX`, `isParameters`                                   | Types and helpers for writing an operation handler                                                                         |

## Links

- [API reference](https://typescript.haste.health/modules/_haste-health_operation-execution.html)
- [Source](https://github.com/HasteHealth/HasteHealth/tree/main/frontend/packages/operation-execution)
- [FHIR REST API: invoking operations](https://haste.health/docs/api/rest_api/fhir/intro) and the [TypeScript SDK overview](https://haste.health/docs/api/sdks/typescript)
- Rust equivalent: [haste-fhir-ops](https://crates.io/crates/haste-fhir-ops)

Part of [Haste Health](https://haste.health), licensed under [Apache-2.0](https://github.com/HasteHealth/HasteHealth/blob/main/LICENSE).
