# @haste-health/x-fhir-query

Evaluates [x-fhir-query](https://build.fhir.org/fhir-xquery.html) templates: FHIR search URLs with embedded `{{ FHIRPath }}` expressions. Each expression is evaluated with [`@haste-health/fhirpath`](https://www.npmjs.com/package/@haste-health/fhirpath) and its results replace the placeholder as a comma separated list, leaving a query string that is ready to send to a FHIR server.

[`@haste-health/testscript-runner`](https://www.npmjs.com/package/@haste-health/testscript-runner) uses it to build TestScript requests whose URLs depend on fixtures created earlier in the run.

## Install

```bash
npm install @haste-health/x-fhir-query
```

## Usage

```ts
import execute from "@haste-health/x-fhir-query";

const patient = {
  resourceType: "Patient",
  id: "123",
  name: [{ given: ["Eve"] }],
};

const query = await execute("Observation?subject=Patient/{{%patient.id}}", {
  variables: { patient },
});
// "Observation?subject=Patient/123"
```

`execute(template, options)` takes the template and the same options object as `fhirpath.evaluate`: `variables` as a record or a lookup function, and optional `meta` for type-aware evaluation. Expressions run without a `$this` context, so reach data through `%variables`. An expression that yields several values becomes a comma separated list, which FHIR search reads as OR.

## Links

- [API reference](https://typescript.haste.health/modules/_haste-health_x-fhir-query.html)
- [Source](https://github.com/HasteHealth/HasteHealth/tree/main/frontend/packages/x-fhir-query)
- [TypeScript SDK overview](https://haste.health/docs/api/sdks/typescript)
- Rust equivalent: [haste-x-fhir-query](https://crates.io/crates/haste-x-fhir-query)

Part of [Haste Health](https://haste.health), licensed under [Apache-2.0](https://github.com/HasteHealth/HasteHealth/blob/main/LICENSE).
