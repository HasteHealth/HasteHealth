import { expect, test } from "@jest/globals";
import jsonpatch from "fast-json-patch";

import { descend, pointer } from "@haste-health/fhir-pointer";
import { Patient, id } from "@haste-health/fhir-types/r4/types";
import { R4 } from "@haste-health/fhir-types/versions";

import buildPatches, { applyMutationImmutable } from "./index.js";

test("Adding a value.", () => {
  const loc = pointer(R4, "Patient", "123" as id);
  const patient: Patient = { resourceType: "Patient", id: "123" } as Patient;

  descend(descend(descend(descend(loc, "name"), 0), "given"), 0);
  expect(
    buildPatches(patient, {
      op: "add",
      path: descend(descend(descend(descend(loc, "name"), 0), "given"), 0),
      value: "test",
    })
  ).toEqual([
    {
      op: "add",
      path: "/name",
      value: [],
    },
    {
      op: "add",
      path: "/name/0",
      value: {},
    },
    {
      op: "add",
      path: "/name/0/given",
      value: [],
    },
    {
      op: "add",
      path: "/name/0/given/0",
      value: "test",
    },
  ]);

  expect(
    buildPatches(patient, {
      op: "add",
      path: descend(loc, "name"),
      value: [{ given: ["John"] }],
    })
  ).toEqual([
    {
      op: "add",
      path: "/name",
      value: [{ given: ["John"] }],
    },
  ]);

  expect(
    buildPatches(
      { ...patient, name: [{ given: ["bob"] }] },
      {
        op: "add",
        path: descend(descend(descend(descend(loc, "name"), 0), "given"), 1),
        value: "Jake",
      }
    )
  ).toEqual([
    {
      op: "add",
      path: "/name/0/given/1",
      value: "Jake",
    },
  ]);

  expect(
    jsonpatch.applyPatch(
      { ...patient, name: [{ given: ["bob"] }] },
      buildPatches(
        { ...patient, name: [{ given: ["bob"] }] },
        {
          op: "add",
          path: descend(descend(descend(descend(loc, "name"), 0), "given"), 1),
          value: "Jake",
        }
      )
    ).newDocument
  ).toEqual({ ...patient, name: [{ given: ["bob", "Jake"] }] });

  expect(
    jsonpatch.applyPatch(
      patient,
      buildPatches(patient, {
        op: "add",
        path: descend(descend(descend(descend(loc, "name"), 0), "given"), 0),
        value: "test",
      })
    ).newDocument
  ).toEqual({ ...patient, name: [{ given: ["test"] }] });
});

test("replace", () => {
  const loc = pointer(R4, "Patient", "123" as id);
  const patient: Patient = { resourceType: "Patient", id: "123" } as Patient;
  expect(
    jsonpatch.applyPatch(
      { ...patient, name: [{ given: ["bob"] }] },
      buildPatches(
        { ...patient, name: [{ given: ["bob"] }] },
        {
          op: "replace",
          path: descend(descend(descend(descend(loc, "name"), 0), "given"), 0),
          value: "Jake",
        }
      )
    ).newDocument
  ).toEqual({ ...patient, name: [{ given: ["Jake"] }] });

  expect(
    jsonpatch.applyPatch(
      { ...patient, name: [{ given: ["bob"] }] },
      buildPatches(
        { ...patient, name: [{ given: ["bob"] }] },
        {
          op: "replace",
          path: descend(descend(descend(descend(loc, "name"), 0), "given"), 1),
          value: "Jake",
        }
      )
    ).newDocument
  ).toEqual({ ...patient, name: [{ given: ["bob", "Jake"] }] });
});

test("removal", () => {
  const loc = pointer(R4, "Patient", "123" as id);
  const patient: Patient = { resourceType: "Patient", id: "123" } as Patient;
  expect(
    jsonpatch.applyPatch(
      { ...patient, name: [{ given: ["bob"] }] },
      buildPatches(
        { ...patient, name: [{ given: ["bob"] }] },
        {
          op: "remove",
          path: descend(descend(descend(descend(loc, "name"), 0), "given"), 0),
        }
      )
    ).newDocument
  ).toEqual({ ...patient, name: [{ given: [] }] });

  expect(
    jsonpatch.applyPatch(
      { ...patient, name: [{ given: ["bob"] }] },
      buildPatches(
        { ...patient, name: [{ given: ["bob"] }] },
        {
          op: "remove",
          path: descend(descend(descend(descend(loc, "name"), 0), "given"), 1),
        }
      )
    ).newDocument
  ).toEqual({ ...patient, name: [{ given: ["bob"] }] });
  expect(
    jsonpatch.applyPatch(
      { ...patient, name: [{ given: ["bob"] }] },
      buildPatches(
        { ...patient, name: [{ given: ["bob"] }] },
        {
          op: "remove",
          path: descend(descend(descend(loc, "identifier"), 0), "system"),
        }
      )
    ).newDocument
  ).toEqual({ ...patient, name: [{ given: ["bob"] }] });
});

test("immutable Patch", () => {
  const loc = pointer(R4, "Patient", "123" as id);
  const patient: Patient = { resourceType: "Patient", id: "123" } as Patient;
  expect(
    applyMutationImmutable(
      { ...patient, name: [{ given: ["bob"] }] },
      {
        op: "replace",
        path: descend(descend(descend(descend(loc, "name"), 0), "given"), 1),
        value: "Jake",
      }
    )
  ).toEqual({ ...patient, name: [{ given: ["bob", "Jake"] }] });
});
