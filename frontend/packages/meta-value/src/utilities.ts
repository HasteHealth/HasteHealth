import * as r4sets from "@oxidized-health/fhir-types/r4/sets";
import { Element, uri } from "@oxidized-health/fhir-types/r4/types";
import * as r4bsets from "@oxidized-health/fhir-types/r4b/sets";
import { FHIR_VERSION, R4, R4B } from "@oxidized-health/fhir-types/versions";

import { IMetaValue, IMetaValueArray } from "./interface.js";

export type RawPrimitive = string | number | boolean | undefined;
export type FHIRPathPrimitive<T extends RawPrimitive> = Element & {
  _type_: "primitive";
  value: T;
};

export function isResourceType(version: FHIR_VERSION, type: uri) {
  switch (version) {
    case R4: {
      return r4sets.resourceTypes.has(type);
    }
    case R4B: {
      return r4bsets.resourceTypes.has(type);
    }
    default: {
      throw new Error(`Unsupported FHIR version: ${version}`);
    }
  }
}

export function isPrimitiveType(version: FHIR_VERSION, type: string) {
  switch (version) {
    case R4: {
      return r4sets.primitiveTypes.has(type);
    }
    case R4B: {
      return r4bsets.primitiveTypes.has(type);
    }
    default: {
      throw new Error(`Unsupported FHIR version: ${version}`);
    }
  }
}

export function isFPPrimitive(
  v: unknown
): v is FHIRPathPrimitive<RawPrimitive> {
  return isObject(v) && v._type_ === "primitive";
}

export function toFPPrimitive<T extends RawPrimitive>(
  value: T,
  element?: Element
): FHIRPathPrimitive<T> {
  return { ...element, value, _type_: "primitive" };
}

export function isRawPrimitive(v: unknown): v is RawPrimitive {
  return (
    typeof v === "string" ||
    typeof v === "number" ||
    typeof v === "boolean" ||
    v === undefined
  );
}

export function isObject(value: unknown): value is Record<string, unknown> {
  return value !== null && typeof value === "object";
}

export function isArray<T>(v: T | T[]): v is T[] {
  return Array.isArray(v);
}

export function flatten<T>(
  node: IMetaValue<T> | IMetaValueArray<T> | undefined
): IMetaValue<T>[] {
  switch (true) {
    case node === undefined: {
      return [];
    }
    case node?.isArray(): {
      return node.toArray() as IMetaValue<T>[];
    }
    case node && node.isArray() === false: {
      return [node];
    }
    default: {
      throw new Error("Should not reach here");
    }
  }
}
