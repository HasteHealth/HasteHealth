# @haste-health/fhir-patch-building

Builds JSON Patch (RFC 6902) operations for a FHIR resource from one typed mutation. You describe the change with a [`@haste-health/fhir-pointer`](https://www.npmjs.com/package/@haste-health/fhir-pointer) location and an `add`, `replace` or `remove` op. The builder inspects the current resource and prepends the `add` operations needed to create any missing arrays and objects along the path, so a patch to `name/0/given/0` applies cleanly to a Patient that has no `name` yet.

The Haste Health admin app and [`@haste-health/components`](https://www.npmjs.com/package/@haste-health/components) use it to turn form edits into FHIR `PATCH` requests.

## Install

```bash
npm install @haste-health/fhir-patch-building
```

## Usage

```ts
import buildPatches, {
  applyMutation,
  applyMutationImmutable,
} from "@haste-health/fhir-patch-building";
import { descend, pointer } from "@haste-health/fhir-pointer";
import type { Patient, id } from "@haste-health/fhir-types/r4/types";
import { R4 } from "@haste-health/fhir-types/versions";

const patient = { resourceType: "Patient", id: "123" } as Patient;
const loc = pointer(R4, "Patient", "123" as id);
const firstGiven = descend(
  descend(descend(descend(loc, "name"), 0), "given"),
  0,
);

buildPatches(patient, { op: "add", path: firstGiven, value: "John" });
// [
//   { op: "add", path: "/name", value: [] },
//   { op: "add", path: "/name/0", value: {} },
//   { op: "add", path: "/name/0/given", value: [] },
//   { op: "add", path: "/name/0/given/0", value: "John" },
// ]

const named = { ...patient, name: [{ given: ["Bob"] }] } as Patient;
const renamed = applyMutationImmutable(named, {
  op: "replace",
  path: firstGiven,
  value: "Jake",
});
// { resourceType: "Patient", id: "123", name: [{ given: ["Jake"] }] }, `named` unchanged
```

- `buildPatches(resource, mutation)` returns the operations; send them as the body of a FHIR `PATCH`, for example with `client.patch` from `@haste-health/client`.
- `replace` with an `undefined` or `null` value becomes a `remove`, and `remove` of a path that does not exist produces no operations.
- `applyMutation` applies the patch in place with `fast-json-patch`; `applyMutationImmutable` uses `immer` and returns a new resource.

## Links

- [API reference](https://typescript.haste.health/modules/_haste-health_fhir-patch-building.html)
- [Source](https://github.com/HasteHealth/HasteHealth/tree/main/frontend/packages/fhir-patch-building)
- [TypeScript SDK overview](https://haste.health/docs/api/sdks/typescript)
- Related: [@haste-health/fhir-pointer](https://www.npmjs.com/package/@haste-health/fhir-pointer), [@haste-health/client](https://www.npmjs.com/package/@haste-health/client)

Part of [Haste Health](https://haste.health), licensed under [Apache-2.0](https://github.com/HasteHealth/HasteHealth/blob/main/LICENSE).
