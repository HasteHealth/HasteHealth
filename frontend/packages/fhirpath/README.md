# @haste-health/fhirpath

A [FHIRPath](https://hl7.org/fhirpath/) evaluator for TypeScript and JavaScript. Expressions are parsed once and cached, then evaluated against any JavaScript value. When the value is a FHIR resource, evaluation is type-aware through [`@haste-health/meta-value`](https://www.npmjs.com/package/@haste-health/meta-value): choice types resolve (`Patient.deceased` reads `deceasedBoolean`), primitive extensions are reachable (`name.given.extension`), and `is`, `as` and `ofType` know the element types of FHIR R4 and R4B.

It powers the search, validation and TestScript tooling in Haste Health's TypeScript packages: [`@haste-health/fhir-validation`](https://www.npmjs.com/package/@haste-health/fhir-validation), [`@haste-health/testscript-runner`](https://www.npmjs.com/package/@haste-health/testscript-runner), [`@haste-health/x-fhir-query`](https://www.npmjs.com/package/@haste-health/x-fhir-query), the admin app and the React components. The Haste Health server uses the Rust engine, [haste-fhirpath](https://crates.io/crates/haste-fhirpath).

## Install

```bash
npm install @haste-health/fhirpath
```

## Usage

```ts
import { evaluate, evaluateWithMeta } from "@haste-health/fhirpath";
import type { Patient, uri } from "@haste-health/fhir-types/r4/types";
import { R4 } from "@haste-health/fhir-types/versions";

const patient = {
  resourceType: "Patient",
  name: [{ given: ["Eve"], family: "Williams" }],
  deceasedBoolean: true,
} as Patient;

await evaluate("4 + 5", {}); // [9]
await evaluate("$this.name.given", patient); // ["Eve"]
await evaluate("$this.deceased", patient); // [true]

// Variables, as an object or an async lookup function
await evaluate(
  "%people.name.given",
  {},
  {
    variables: { people: [patient] },
  },
);

// Type-aware evaluation returns MetaValues with type and location
const values = await evaluateWithMeta("$this.name", patient, {
  fhirVersion: R4,
  type: "Patient" as uri,
});
values[0].meta()?.type; // "HumanName"
values[0].location(); // ["name", 0]
```

`evaluate(expression, context, options)` returns the result collection as plain values; `evaluateWithMeta` returns `IMetaValue`s. Options are `fhirVersion` (`R4` by default), `type` (the FHIR type of the context, otherwise taken from `resourceType`) and `variables`. `@haste-health/fhirpath/analyze` evaluates an expression against a type instead of a value and reports the types it would produce, for checking an expression before any data exists.

## Supported

| Category  | Supported                                                                                                                           |
| --------- | ----------------------------------------------------------------------------------------------------------------------------------- |
| Operators | `+`, `-`, `*`, `/`, `=`, `!=`, `and`, `or`, `\|` (union), `in`, `contains`, `is`, `as`                                              |
| Existence | `empty`, `exists`, `all`, `allTrue`, `anyTrue`, `allFalse`, `anyFalse`, `subsetOf`, `supersetOf`, `count`, `distinct`, `isDistinct` |
| Filtering | `where`, `select`, `repeat`, `ofType`, `as`, `children`, `descendants`                                                              |
| Other     | `replace`, `type`, `resolve` (returns the reference unchanged, so `resolve() is Patient` tests the target type without fetching)    |
| Terms     | literals, `$this`, `$index`, `$total`, `%variables`, indexing `[n]`, parentheses                                                    |

Comparison (`<`, `>`), `implies`, `xor`, the string and math function groups and date arithmetic are not implemented in this engine. The Rust engine, [haste-fhirpath](https://crates.io/crates/haste-fhirpath), implements a much larger part of the specification and is what the Haste Health server evaluates with.

## Links

- [API reference](https://typescript.haste.health/modules/_haste-health_fhirpath.html)
- [Source](https://github.com/HasteHealth/HasteHealth/tree/main/frontend/packages/fhirpath)
- [TypeScript SDK overview](https://haste.health/docs/api/sdks/typescript)
- Rust equivalent: [haste-fhirpath](https://crates.io/crates/haste-fhirpath)

Part of [Haste Health](https://haste.health), licensed under [Apache-2.0](https://github.com/HasteHealth/HasteHealth/blob/main/LICENSE).
