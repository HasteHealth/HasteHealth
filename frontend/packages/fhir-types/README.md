# @haste-health/fhir-types

TypeScript types for every FHIR R4 (4.0.1) and R4B (4.3.0) resource, complex type and primitive, generated from the FHIR StructureDefinitions in the Haste Health [artifacts](https://github.com/HasteHealth/HasteHealth/tree/main/artifacts) packages. The same generation covers Haste Health's own resources, so `User`, `Membership`, `Project`, `ClientApplication`, `AccessPolicyV2`, `IdentityProvider` and `ViewDefinition` are typed alongside `Patient` and `Observation`.

The package is almost entirely types. The runtime exports are the version constants and three `Set`s of type names. Every other Haste Health TypeScript package builds on it, from [`@haste-health/client`](https://www.npmjs.com/package/@haste-health/client) to the React components.

## Install

```bash
npm install @haste-health/fhir-types
```

## Usage

```ts
import type { Patient, date, id } from "@haste-health/fhir-types/r4/types";
import {
  R4,
  type Resource,
  type ResourceType,
} from "@haste-health/fhir-types/versions";

const patient: Patient = {
  resourceType: "Patient",
  id: "123" as id,
  name: [{ family: "Williams", given: ["Eve"] }],
  birthDate: "1973-05-31" as date,
};

// Version-generic code: Resource<Version, Type> is the matching interface.
function summarize<T extends ResourceType<R4>>(
  type: T,
  resource: Resource<R4, T>,
) {
  return `${type}/${resource.id}`;
}
```

FHIR primitives other than `string` and `boolean` are branded types (`id`, `code`, `date`, `uri`, `decimal` and so on), so a `uri` is not accepted where a `code` is expected and literals need an `as code` style cast. Primitive extensions live in the `_field` companion properties, as in FHIR JSON.

## Modules

| Import                                          | Contents                                                                                                               |
| ----------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------- |
| `@haste-health/fhir-types/r4/types`             | R4 interfaces and primitives, `ResourceType`, `AResource<T>`, `Resource`, `DataType`, `AData<T>`                       |
| `@haste-health/fhir-types/r4b/types`            | The same for R4B                                                                                                       |
| `@haste-health/fhir-types/r4/sets`, `/r4b/sets` | Runtime `Set<string>`s: `resourceTypes`, `complexTypes`, `primitiveTypes`                                              |
| `@haste-health/fhir-types/versions`             | `R4` (`"4.0"`), `R4B` (`"4.3"`), `FHIR_VERSION`, and version-generic `Resource<V, T>`, `ResourceType<V>`, `Data<V, T>` |

The generated files carry a `@generated` header. Regenerate them from the `frontend/` workspace with `pnpm --filter @haste-health/fhir-types run generate` after changing the artifacts.

## Links

- [API reference](https://typescript.haste.health/modules/_haste-health_fhir-types.html)
- [Source](https://github.com/HasteHealth/HasteHealth/tree/main/frontend/packages/fhir-types)
- [TypeScript SDK overview](https://haste.health/docs/api/sdks/typescript)
- [FHIR R4 model reference](https://haste.health/docs/reference/fhir/model/resources/Patient) on haste.health, one page per resource
- Rust equivalent: [haste-fhir-model](https://crates.io/crates/haste-fhir-model)

Part of [Haste Health](https://haste.health), licensed under [Apache-2.0](https://github.com/HasteHealth/HasteHealth/blob/main/LICENSE).
