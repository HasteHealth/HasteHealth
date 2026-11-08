# @haste-health/fhir-validation

Validates FHIR R4 and R4B resources in TypeScript. `validate` checks a value against the base StructureDefinition of its type (element structure, cardinality, primitive formats, required bindings, reference targets) and then against every profile listed in `meta.profile` (cardinality, fixed values, patterns, slicing, type constraints, nested profiles). It returns `OperationOutcome` issues that name the failing element, ready to show in a UI or return as a FHIR `OperationOutcome`.

[`@haste-health/operation-execution`](https://www.npmjs.com/package/@haste-health/operation-execution) uses it to validate operation parameters. The Haste Health server validates writes with the Rust equivalent, [haste-fhir-profiling](https://crates.io/crates/haste-fhir-profiling).

## Install

```bash
npm install @haste-health/fhir-validation
```

## Usage

```ts
import validate from "@haste-health/fhir-validation";
import type { ValidationCTX } from "@haste-health/fhir-validation/types";
import { R4 } from "@haste-health/fhir-types/versions";
import type { uri } from "@haste-health/fhir-types/r4/types";

const ctx: ValidationCTX = {
  fhirVersion: R4,
  // Return the resource a canonical URL points at (StructureDefinitions for profiles).
  // Resolve from a FHIR server, or offline from the FHIR packages with @haste-health/artifacts.
  resolveCanonical: async (fhirVersion, type, url) =>
    structureDefinitions.find((sd) => sd.url === url),
  // Optional: check codes with a terminology service, called for elements whose
  // binding strength is "required" with the binding's ValueSet URL and the code.
  validateCode: async (valueSet, code) => true,
};

const issues = await validate(ctx, "Patient" as uri, {
  resourceType: "Patient",
  meta: {
    profile: [
      "http://hl7.org/fhir/us/core/StructureDefinition/us-core-patient",
    ],
  },
  name: [{ family: "Doe" }],
});
// [] when valid; otherwise issues such as
// { severity: "error", code: "required", diagnostics: "...", expression: ["/identifier"] }
```

## Entry points

| Import                                     | Exports                                                                                                  |
| ------------------------------------------ | -------------------------------------------------------------------------------------------------------- |
| `@haste-health/fhir-validation`            | `validate(ctx, type, value, path?)`: structural validation plus the profiles in `meta.profile`           |
| `@haste-health/fhir-validation/structural` | `validate(ctx, type, value, path?)`: structural validation only, also for data types such as `HumanName` |
| `@haste-health/fhir-validation/types`      | `ValidationCTX`, `ElementLoc`                                                                            |

`type` is the FHIR type name (`Patient`, `Observation`, `HumanName`, ...). The optional `path` is a `@haste-health/fhir-pointer` location used to prefix the issue locations when validating part of a larger document.

## Profile support

- Cardinality, `fixed[x]`, `pattern[x]`, and type restrictions on choice elements.
- Slicing with `value`, `pattern`, `exists` and `type` discriminators; `profile` discriminators are not supported yet. Exercised against the US Core Patient and Blood Pressure profiles in the test suite.
- Element-level `type.profile` references, resolved through `resolveCanonical` and validated recursively.
- A profile whose canonical cannot be resolved is skipped rather than reported.

## Links

- [API reference](https://typescript.haste.health/modules/_haste-health_fhir-validation.html)
- [Source](https://github.com/HasteHealth/HasteHealth/tree/main/frontend/packages/fhir-validation)
- [Profiling and validation guide](https://haste.health/docs/guides/profiling) and the [TypeScript SDK overview](https://haste.health/docs/api/sdks/typescript)
- Rust equivalent: [haste-fhir-profiling](https://crates.io/crates/haste-fhir-profiling)

Part of [Haste Health](https://haste.health), licensed under [Apache-2.0](https://github.com/HasteHealth/HasteHealth/blob/main/LICENSE).
