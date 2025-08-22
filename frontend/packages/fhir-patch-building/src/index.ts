import jsonpatch, { Operation } from "fast-json-patch";
import { produce } from "immer";

import * as fpt from "@oxidized-health/fhir-pointer";
import { Resource, id } from "@oxidized-health/fhir-types/r4/types";
import { R4 } from "@oxidized-health/fhir-types/versions";

export interface Mutation<T, R> {
  path: fpt.Loc<T, R, any>;
  op: "add" | "remove" | "replace";
  value?: R;
}

function getValue<T extends object, R>(
  value: T,
  pointer: fpt.Loc<T, R, any>
): R {
  return fpt.get(pointer, value);
}

function valueExists<T extends object, R>(
  value: T,
  json_pointer: fpt.Loc<T, R, any>
) {
  return getValue(value, json_pointer) !== undefined;
}

function deriveNextValuePlaceHolder(
  fields: (string | number | symbol)[]
): Array<unknown> | Record<string, unknown> {
  if (typeof fields[1] === "number") {
    return [];
  }
  return {};
}

function createPatchesNonExistantFields<T extends Record<string, any>, R>(
  resource: T,
  path: fpt.Loc<T, R, any>
) {
  const fields = fpt.fields(path);

  let patches: Operation[] = [];
  let curValue = resource as unknown;
  let curPointer: fpt.Loc<T, any, any> = fpt.pointer(
    R4,
    resource.resourceType,
    resource.id as id
  );
  for (let i = 0; i < fields.length; i++) {
    curPointer = fpt.descend(curPointer, fields[i]);
    curValue = getValue(resource, curPointer);
    if (curValue === undefined) {
      const nextValue = deriveNextValuePlaceHolder(fields.slice(i));
      patches = [
        ...patches,
        { op: "add", path: fpt.toJSONPointer(curPointer), value: nextValue },
      ];
    }
  }
  return patches;
}

export default function buildPatches<T extends Resource, R>(
  value: T,
  mutation: Mutation<T, R>
): Operation[] {
  // Builds patches with a given mutation to include non existant values up to the point in the path
  // where the mutation occurs

  switch (mutation.op) {
    case "remove": {
      if (valueExists(value, mutation.path)) {
        return [
          {
            op: "remove",
            path: fpt.toJSONPointer(mutation.path),
          },
        ];
      }
      return [];
    }
    case "add": {
      const patches = createPatchesNonExistantFields(value, mutation.path);
      //If last is adding value remove here as collection will only add once.
      if (
        patches[patches.length - 1]?.op === "add" &&
        patches[patches.length - 1]?.path === fpt.toJSONPointer(mutation.path)
      ) {
        return [
          ...patches.slice(0, patches.length - 1),
          {
            op: "add",
            path: fpt.toJSONPointer(mutation.path),
            value: mutation.value,
          },
        ];
      }
      return [
        ...patches,
        {
          op: "add",
          path: fpt.toJSONPointer(mutation.path),
          value: mutation.value,
        },
      ];
    }
    case "replace": {
      if (mutation.value === undefined || mutation.value === null) {
        return buildPatches(value, {
          op: "remove",
          path: mutation.path,
        });
      }
      const patches = createPatchesNonExistantFields(value, mutation.path);
      return [
        ...patches,
        {
          op: "replace",
          path: fpt.toJSONPointer(mutation.path),
          value: mutation.value,
        },
      ];
    }
    default:
      throw new Error(`Invalid operation '${mutation.op}'`);
  }
}

export function applyMutation<T extends Resource, R>(
  value: T,
  mutation: Mutation<T, R>
): T {
  return jsonpatch.applyPatch(value, buildPatches(value, mutation)).newDocument;
}

export function applyMutationImmutable<T extends Resource, R>(
  value: T,
  mutation: Mutation<T, R>
): T {
  return produce(value, (v) => {
    applyMutation(v as T, mutation);
  });
}
