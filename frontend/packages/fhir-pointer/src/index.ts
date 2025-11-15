/* eslint-disable @typescript-eslint/no-explicit-any */
import jsonpointer from "jsonpointer";

import { id } from "@haste-health/fhir-types/r4/types";
import {
  AllDataTypes,
  Data,
  DataType,
  FHIR_VERSION,
  R4,
} from "@haste-health/fhir-types/versions";

import { toJSONPointer } from "./conversions.js";
import { Loc, NullGuard, Parent } from "./types.js";
import { escapeField, unescapeField } from "./utilities.js";

export type { Loc, NullGuard } from "./types.js";
export * from "./conversions.js";

/*
 ** Descend Loc pointer with field.
 */
export function descend<
  T,
  R,
  P extends Parent<T>,
  Field extends keyof NonNullable<R>
>(loc: Loc<T, R, P>, field: Field): Loc<T, NullGuard<R, Field>, typeof loc> {
  return `${loc}/${escapeField(String(field))}` as Loc<
    T,
    NullGuard<R, Field>,
    typeof loc
  >;
}

type ReturnType<L> = L extends Loc<infer B, infer T, infer P> ? T : any;

/*
 ** Ascend Loc pointer to parent.
 */
export function ascend<T, R, P extends Parent<T>>(
  loc: Loc<T, R, P>
):
  | { parent: NonNullable<P>; field: NonNullable<keyof ReturnType<P>> }
  | undefined {
  // At root so return undefined.
  const lastIndexSlash = loc.lastIndexOf("/");
  if (lastIndexSlash === -1) return undefined;
  const field = unescapeField(loc.substring(lastIndexSlash + 1));
  return {
    parent: loc.slice(0, lastIndexSlash) as NonNullable<P>,
    field: Number.isNaN(parseInt(field))
      ? field
      : (parseInt(field) as keyof NonNullable<ReturnType<P>>),
  };
}

export function pathMeta<
  Version extends FHIR_VERSION,
  T extends Data<Version, AllDataTypes>,
  R,
  P extends Parent<T>
>(
  loc: Loc<T, R, P>
): { version: FHIR_VERSION; type: DataType<Version>; id: id } {
  const indexOfLastSlash = loc.indexOf("/");
  const [fhirVersion, type, id] = loc
    .substring(0, indexOfLastSlash === -1 ? loc.length : indexOfLastSlash)
    .split("|");
  return {
    version: fhirVersion as FHIR_VERSION,
    type: type as DataType<Version>,
    id: id as id,
  };
}

export function get<T, R, P extends Parent<T>>(loc: Loc<T, R, P>, v: T): R {
  const pointer = toJSONPointer(loc);

  if (typeof v !== "object" || v === null) {
    if (pointer === "") {
      return v as unknown as R;
    }
    throw new Error(`Invalid pointer for primitive value ${typeof v}`);
  }

  return jsonpointer.get(v, pointer) as R;
}

export function fields<T extends object, R, P extends Parent<T>>(
  loc: Loc<T, R, P>
) {
  let asc = ascend(loc);
  const fields = [];
  while (asc) {
    fields.unshift(asc.field);
    asc = ascend(asc.parent);
  }
  return fields;
}

export function root<
  Version extends FHIR_VERSION,
  T extends Data<Version, AllDataTypes>,
  R,
  P extends Parent<T>
>(loc: Loc<T, R, P>): Loc<T, T> {
  const { version, type, id } = pathMeta(loc);
  return pointer(version, type, id) as any as Loc<T, T>;
}

function metaString<Version extends FHIR_VERSION>(
  fhirVersion: FHIR_VERSION,
  type: DataType<Version>,
  id: id
) {
  return `${fhirVersion}|${type}|${id}`;
}

/*
 ** Creates a Loc pointer for a resource.
 */
export function pointer<
  Version extends FHIR_VERSION,
  T extends DataType<Version>
>(
  fhir_version: FHIR_VERSION,
  resourceType: T,
  resourceId: id
): Loc<Data<Version, T>, Data<Version, T>> {
  return `${metaString(fhir_version, resourceType, resourceId)}` as Loc<
    Data<Version, T>,
    Data<Version, T>
  >;
}

export function typedPointer<V, T>(): Loc<V, T, Parent<V>> {
  return metaString(R4, "Unknown" as DataType<R4>, "unknown" as id) as Loc<
    V,
    T,
    Parent<V>
  >;
}
