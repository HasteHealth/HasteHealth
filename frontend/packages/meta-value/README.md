# @haste-health/meta-value

Wraps a FHIR value together with its type information: the FHIR version, the element's declared type (`HumanName`, `code`, `BackboneElement` and so on), its cardinality and its location inside the containing resource. Descending through a `MetaValue` field by field keeps the metadata in step with the data, including choice types (`value[x]`), content references, and the `_field` companions that carry primitive extensions. The metadata tables are generated from the FHIR R4 and R4B StructureDefinitions.

This is what gives [`@haste-health/fhirpath`](https://www.npmjs.com/package/@haste-health/fhirpath) type-aware evaluation (`is`, `as`, `ofType`), lets [`@haste-health/fhir-pointer`](https://www.npmjs.com/package/@haste-health/fhir-pointer) turn a JSON Pointer into a FHIRPath, and drives element validation in [`@haste-health/fhir-validation`](https://www.npmjs.com/package/@haste-health/fhir-validation).

## Install

```bash
npm install @haste-health/meta-value
```

## Usage

```ts
import { R4 } from "@haste-health/fhir-types/versions";
import { flatten } from "@haste-health/meta-value/utilities";
import { metaValue } from "@haste-health/meta-value/v2";

const patient = metaValue(
  { fhirVersion: R4 },
  {
    resourceType: "Patient",
    name: [{ given: ["Bob"], _given: [{ id: "g1" }] }],
    deceasedBoolean: true,
  },
);

patient?.meta()?.type; // "Patient" (taken from resourceType)

const name = patient?.descend("name");
name?.meta(); // { fhirVersion: "4.0", type: "HumanName", cardinality: "array" }

const given = flatten(name)[0]?.descend("given")?.descend(0);
given?.getValue(); // "Bob"
given?.location(); // ["name", 0, "given", 0]
given?.descend("id")?.getValue(); // "g1", read from the _given companion

patient?.descend("deceased")?.meta()?.type; // "boolean", resolved from deceasedBoolean
```

Pass `type` in the options when the value is not a resource, for example `{ fhirVersion: R4, type: "HumanName" }`. Values with no matching metadata still work: they descend as plain JavaScript objects and `meta()` returns `undefined`.

## API

| Import                               | Contents                                                                                         |
| ------------------------------------ | ------------------------------------------------------------------------------------------------ |
| `@haste-health/meta-value/v2`        | `metaValue(options, value)`                                                                      |
| `@haste-health/meta-value/interface` | `IMetaValue`, `IMetaValueArray`, `TypeInfo`, `Location`                                          |
| `@haste-health/meta-value/utilities` | `flatten`, `isResourceType`, `isPrimitiveType`, `isObject`, `isArray` and friends                |
| `@haste-health/meta-value/meta`      | `getStartingMeta`, `getMeta`, `resolveMeta`: direct access to the metadata tables                |
| `@haste-health/meta-value/spoof`     | `spoof(version, type)`: a typed value with no data, for analysing expressions without a resource |
| `@haste-health/meta-value`           | `resolveTypeToCanonical(version, type)` plus the `v2`, `interfaces` and `utilities` namespaces   |

Every `IMetaValue` offers `meta()`, `getValue()`, `location()`, `descend(field)`, `keys()`, `isType(type)`, `asType(type)` and `isArray()`; arrays add `toArray()`.

## Links

- [API reference](https://typescript.haste.health/modules/_haste-health_meta-value.html)
- [Source](https://github.com/HasteHealth/HasteHealth/tree/main/frontend/packages/meta-value)
- [TypeScript SDK overview](https://haste.health/docs/api/sdks/typescript)
- Rust equivalent: the `MetaValue` trait in [haste-reflect](https://crates.io/crates/haste-reflect)

Part of [Haste Health](https://haste.health), licensed under [Apache-2.0](https://github.com/HasteHealth/HasteHealth/blob/main/LICENSE).
