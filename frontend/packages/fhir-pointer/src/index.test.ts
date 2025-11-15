import { expect, test } from "@jest/globals";

import { Patient, id } from "@haste-health/fhir-types/r4/types";
import { R4 } from "@haste-health/fhir-types/versions";

import {
  Loc,
  ascend,
  descend,
  fields,
  get,
  pathMeta,
  pointer,
  root,
  toJSONPointer,
  typedPointer,
} from "./index";

test("pointer", () => {
  const loc = pointer(R4, "Patient", "123" as id);
  expect(loc).toBe("4.0|Patient|123");
});

test("ascend pointer", () => {
  const loc = descend(pointer(R4, "Patient", "123" as id), "name");
  expect(ascend(loc)).toEqual({ parent: "4.0|Patient|123", field: "name" });
  // @ts-ignore
  expect(ascend(ascend(loc)?.parent)).toEqual(undefined);

  const nestedLoc = descend(
    descend(descend(pointer(R4, "Patient", "123" as id), "name"), 0),
    "given"
  );
  expect(nestedLoc).toEqual("4.0|Patient|123/name/0/given");
  expect(ascend(nestedLoc)).toEqual({
    parent: "4.0|Patient|123/name/0",
    field: "given",
  });

  // @ts-ignore
  expect(ascend(ascend(nestedLoc)?.parent)).toEqual({
    parent: "4.0|Patient|123/name",
    field: 0,
  });
});

test("get function", () => {
  const nestedLoc = descend(
    descend(descend(pointer(R4, "Patient", "123" as id), "name"), 0),
    "given"
  );

  const patient: Patient = {
    resourceType: "Patient",
    id: "123",
    name: [{ given: ["John"] }],
  } as Patient;

  expect(get(nestedLoc, patient)).toEqual(["John"]);
  expect(
    get(
      descend(descend(pointer(R4, "Patient", "123" as id), "name"), 1),
      patient
    )
  ).toEqual(undefined);

  expect(
    get(
      descend(
        descend(descend(pointer(R4, "Patient", "123" as id), "name"), 1),
        "given"
      ),
      patient
    )
  ).toEqual(undefined);

  expect(
    get(
      descend(
        descend(descend(pointer(R4, "Patient", "123" as id), "identifier"), 1),
        "system"
      ),
      patient
    )
  ).toEqual(undefined);
});

test("path meta", () => {
  const nestedLoc = descend(pointer(R4, "Patient", "123" as id), "name");
  expect(pathMeta(nestedLoc)).toEqual({
    version: "4.0",
    type: "Patient",
    id: "123",
  });
});

test("fields", () => {
  const nestedLoc = descend(
    descend(descend(pointer(R4, "Patient", "123" as id), "name"), 0),
    "given"
  );
  expect(fields(nestedLoc)).toEqual(["name", 0, "given"]);
});

test("root", () => {
  const nestedLoc = descend(
    descend(descend(pointer(R4, "Patient", "123" as id), "name"), 0),
    "given"
  );
  expect(root(nestedLoc)).toEqual("4.0|Patient|123");
  expect(toJSONPointer(root(nestedLoc))).toEqual("");
  expect(root(pointer(R4, "Patient", "123" as id))).toEqual("4.0|Patient|123");
});

test("escaping strings.", () => {
  const loc = pointer(R4, "Patient", "asdf" as id) as Loc<any, any>;
  const escapedLoc = descend(descend(loc, "~name"), "/test");
  expect(escapedLoc).toEqual("4.0|Patient|asdf/~0name/~1test");
  expect(toJSONPointer(escapedLoc)).toEqual("/~0name/~1test");
  expect(get(escapedLoc, { "~name": { "/test": "test" } })).toEqual("test");
});

test("typedpointer", () => {
  const loc = typedPointer() as Loc<any, any>;
  expect(toJSONPointer(loc)).toEqual("");
  const escapedLoc = descend(descend(loc, "~name"), "/test");
  expect(escapedLoc).toEqual("4.0|Unknown|unknown/~0name/~1test");
  expect(toJSONPointer(escapedLoc)).toEqual("/~0name/~1test");
  expect(get(escapedLoc, { "~name": { "/test": "test" } })).toEqual("test");
});
