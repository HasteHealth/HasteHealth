# @haste-health/fhir-pointer

Typesafe pointers into FHIR resources. A `Loc` is a string of the form `<fhirVersion>|<resourceType>|<id>/<json pointer>` whose type parameters remember the root resource type and the type of the element it points at, so `descend` only accepts fields that exist on the current element and `get` returns the element's type. A `Loc` converts to a plain JSON Pointer (RFC 6901) and, with the type metadata from [`@haste-health/meta-value`](https://www.npmjs.com/package/@haste-health/meta-value), to a FHIRPath expression.

[`@haste-health/fhir-patch-building`](https://www.npmjs.com/package/@haste-health/fhir-patch-building) builds JSON Patches from these pointers, [`@haste-health/fhir-validation`](https://www.npmjs.com/package/@haste-health/fhir-validation) and [`@haste-health/operation-execution`](https://www.npmjs.com/package/@haste-health/operation-execution) use them to say where an error occurred, and the Haste Health admin app and React components bind form fields to resource elements with them.

## Install

```bash
npm install @haste-health/fhir-pointer
```

## Usage

```ts
import {
  ascend,
  descend,
  fields,
  get,
  pointer,
  toFHIRPath,
  toJSONPointer,
} from "@haste-health/fhir-pointer";
import type { Patient, id } from "@haste-health/fhir-types/r4/types";
import { R4 } from "@haste-health/fhir-types/versions";

const patient = {
  resourceType: "Patient",
  id: "123",
  name: [{ given: ["John"] }],
} as Patient;

const root = pointer(R4, "Patient", "123" as id); // "4.0|Patient|123"
const given = descend(descend(descend(root, "name"), 0), "given");
// typed as Loc<Patient, string[] | undefined, ...>, "4.0|Patient|123/name/0/given"

get(given, patient); // ["John"]
fields(given); // ["name", 0, "given"]
ascend(given); // { parent: "4.0|Patient|123/name/0", field: "given" }
toJSONPointer(given); // "/name/0/given"

const coding = descend(
  descend(
    descend(pointer(R4, "Observation", "o1" as id), "valueCodeableConcept"),
    "coding",
  ),
  0,
);
toFHIRPath(coding); // "$this.value.coding[0]"
```

Field names are escaped per RFC 6901 (`~` and `/`), and `get` returns `undefined` rather than throwing when part of the path is missing.

## API

| Function                                    | Purpose                                                                 |
| ------------------------------------------- | ----------------------------------------------------------------------- |
| `pointer(version, resourceType, id)`        | Root `Loc` for a resource                                               |
| `typedPointer<V, T>()`                      | Root `Loc` for any value, without FHIR metadata                         |
| `descend(loc, field)`, `ascend(loc)`        | Move down one field or array index, or back up to the parent            |
| `get(loc, value)`                           | Read the element the pointer addresses                                  |
| `fields(loc)`, `root(loc)`, `pathMeta(loc)` | Path segments, the root pointer, and `{ version, type, id }`            |
| `toJSONPointer(loc)`, `toFHIRPath(loc)`     | Conversions; `toFHIRPath` folds choice types (`valueCoding` to `value`) |
| `Loc<Root, Element, Parent>`, `NullGuard`   | The pointer type and the helper that propagates optional parents        |

## Links

- [API reference](https://typescript.haste.health/modules/_haste-health_fhir-pointer.html)
- [Source](https://github.com/HasteHealth/HasteHealth/tree/main/frontend/packages/fhir-pointer)
- [TypeScript SDK overview](https://haste.health/docs/api/sdks/typescript)
- Rust equivalent: [haste-pointer](https://crates.io/crates/haste-pointer)

Part of [Haste Health](https://haste.health), licensed under [Apache-2.0](https://github.com/HasteHealth/HasteHealth/blob/main/LICENSE).
