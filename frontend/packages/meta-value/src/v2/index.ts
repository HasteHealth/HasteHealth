/* eslint-disable @typescript-eslint/no-explicit-any */

import { FPPrimitiveNode } from "@haste-health/codegen/generate/meta-data";
import {
  Element,
  Resource as R4Resource,
  Reference,
  Resource,
  uri,
} from "@haste-health/fhir-types/lib/generated/r4/types";
import { FHIR_VERSION, R4 } from "@haste-health/fhir-types/versions";

import {
  IMetaValue,
  IMetaValueArray,
  Location,
  TypeInfo,
} from "../interface.js";
import { descendLoc } from "../loc.js";
import {
  FHIRPathPrimitive as FHIRPrimitive,
  RawPrimitive,
  isArray,
  isFPPrimitive,
  isObject,
  isPrimitiveType,
  toFPPrimitive,
} from "../utilities.js";
import { ElementNode, getMeta, getStartingMeta, resolveMeta } from "./meta.js";
import { SpoofMetaValueV2 } from "./spoof.js";

function conversion<T>(
  fhirVersion: FHIR_VERSION,
  meta: ElementNode,
  value: Array<unknown>,
  location: Location
): MetaValueV2Singular<T>[] {
  return value.map((v, i) => {
    if (v instanceof MetaValueV2Singular) return v;
    return new MetaValueV2Singular(
      fhirVersion,
      { ...meta, cardinality: "single" },
      v,
      [...location, i]
    );
  });
}

class MetaValueV2Array<T> implements IMetaValueArray<T> {
  private readonly _value: Array<MetaValueV2Singular<unknown>>;

  private _location: Location;

  constructor(value: MetaValueV2Singular<unknown>[], location: Location) {
    this._value = value;
    this._location = location;
  }
  keys(): (string | number)[] {
    throw new Error("Method not allowed on arrays.");
  }
  getValue(): Array<T> {
    return this._value.map((v) => v.getValue()) as Array<T>;
  }
  isType(_type: string): boolean {
    throw new Error("Method not allowed on arrays.");
  }
  asType(type: string): IMetaValue<unknown> | undefined {
    if (this.isType(type)) {
      return this;
    }
    return undefined;
  }
  toArray(): Array<MetaValueV2Singular<unknown>> {
    return this._value;
  }
  isArray(): this is MetaValueV2Array<Array<T>> {
    return true;
  }
  meta(): TypeInfo | undefined {
    const meta = this._value[0]?.meta();
    if (meta) {
      return {
        ...meta,
        cardinality: "array",
      };
    }
    return undefined;
  }
  location(): Location {
    return this._location;
  }
  descend(field: number): IMetaValue<unknown> | undefined {
    return this._value[field];
  }
}

function deriveFHIRPrimitives(
  value: RawPrimitive | RawPrimitive[] | undefined,
  element: Element | Element[] | undefined
): FHIRPrimitive<RawPrimitive> | FHIRPrimitive<RawPrimitive>[] | undefined {
  if (value === undefined && element === undefined) return undefined;
  if (isArray(value)) {
    return value.map((v, i) =>
      toFPPrimitive(v, (element as Element[] | undefined)?.[i])
    );
  }
  return toFPPrimitive(value, element as Element);
}

class MetaValueV2Singular<T> implements IMetaValue<T> {
  private _value: T | FHIRPrimitive<RawPrimitive>;
  private readonly _meta: ElementNode | FPPrimitiveNode;
  private _fhirVersion: FHIR_VERSION;

  private _location: Location;

  constructor(
    fhirVersion: FHIR_VERSION,

    meta: ElementNode | FPPrimitiveNode,
    value: T,
    location: Location
  ) {
    this._fhirVersion = fhirVersion;
    this._meta = meta;
    this._value = value;
    this._location = location;
  }
  get _internal_meta_() {
    return this._meta;
  }
  getValue(): T {
    if (isFPPrimitive(this._value)) {
      return this._value.value as T;
    }
    return this._value;
  }
  keys() {
    if ("properties" in this._meta) {
      return Object.keys(this._meta.properties ?? {});
    }
    return [];
  }
  meta(): TypeInfo | undefined {
    return {
      type: this._meta.type,
      fhirVersion: this._fhirVersion,
      cardinality: this._meta.cardinality,
    };
  }
  isArray(): this is MetaValueV2Array<T> {
    return false;
  }
  asType(type: string): IMetaValue<unknown> | undefined {
    if (this.isType(type)) {
      return this;
    }
    return undefined;
  }
  isType(type: string): boolean {
    switch (true) {
      case type === "Resource" || type === "DomainResource": {
        return (this.getValue() as Resource).resourceType !== undefined;
      }
      case this.meta()?.type === "Reference" && type !== "Reference": {
        return (this.getValue() as Reference).reference?.split("/")[0] === type;
      }
      default: {
        return this.meta()?.type === type;
      }
    }
  }
  location(): Location {
    return this._location;
  }
  descend(_field: string): IMetaValue<unknown> | undefined {
    const unresolvedMeta = getMeta(this._fhirVersion, this._meta, _field);
    if (!unresolvedMeta) return undefined;
    const nextMeta = resolveMeta(
      this._fhirVersion,
      unresolvedMeta,
      this._value,
      _field
    );
    if (!nextMeta?.meta) return undefined;

    switch (true) {
      case isPrimitiveType(
        this._fhirVersion,
        (nextMeta.meta as ElementNode).type
      ): {
        const value = (this._value as any)?.[nextMeta.field];
        const element = (this._value as any)?.[`_${nextMeta.field}`];

        switch (true) {
          case value === undefined && element === undefined: {
            return undefined;
          }
          case nextMeta.meta.cardinality === "array": {
            return new MetaValueV2Array(
              conversion(
                this._fhirVersion,
                nextMeta.meta as ElementNode,
                [
                  ...new Array(
                    Math.max(value?.length ?? 0, element?.length ?? 0)
                  ),
                ].map((_z, i) => toFPPrimitive(value?.[i], element?.[i])),
                descendLoc(this, nextMeta.field)
              ),
              descendLoc(this, nextMeta.field)
            );
          }
          default: {
            return new MetaValueV2Singular(
              this._fhirVersion,
              nextMeta.meta as ElementNode,
              toFPPrimitive(value, element),
              descendLoc(this, nextMeta.field)
            );
          }
        }
      }
      default: {
        const value = (this._value as any)?.[nextMeta.field];
        switch (true) {
          case value === undefined: {
            return undefined;
          }
          case nextMeta.meta.cardinality === "array": {
            return new MetaValueV2Array(
              conversion(
                this._fhirVersion,
                nextMeta.meta as ElementNode,
                (this._value as any)?.[nextMeta.field] ?? [],
                descendLoc(this, nextMeta.field)
              ),
              descendLoc(this, nextMeta.field)
            );
          }
          default: {
            return new MetaValueV2Singular(
              this._fhirVersion,
              nextMeta.meta as ElementNode,
              (this._value as any)?.[nextMeta.field],
              descendLoc(this, nextMeta.field)
            );
          }
        }
      }
    }
  }
}

class NonMetaValue<T> implements IMetaValue<T> {
  private _value: T;
  private _fhirVersion: FHIR_VERSION;
  private _location: Location;

  constructor(fhirVersion: FHIR_VERSION, value: T, location: Location) {
    this._value = value;
    this._fhirVersion = fhirVersion;
    this._location = location;
  }
  meta(): TypeInfo | undefined {
    return undefined;
  }
  isType(type: string): boolean {
    if (type === "Resource" || type === "DomainResource") {
      return (this.getValue() as Resource).resourceType !== undefined;
    }
    return (
      (this.getValue() as unknown as R4Resource | undefined)?.resourceType ===
      type
    );
  }
  keys() {
    if (isObject(this._value)) {
      return Object.keys(this._value);
    }
    return [];
  }
  getValue(): T {
    return this._value;
  }
  toArray(): IMetaValue<T>[] {
    if (this.isArray()) {
      return (this._value as any)?.map((v: any, i: number) => {
        return new NonMetaValue(this._fhirVersion, v, [...this._location, i]);
      });
    }
    return [this];
  }
  asType(type: string): IMetaValue<unknown> | undefined {
    if (this.isType(type)) {
      return this;
    }
    return undefined;
  }
  isArray(): this is IMetaValueArray<T> {
    return isArray(this._value);
  }
  location(): Location | undefined {
    return this._location;
  }
  descend(field: string | number): IMetaValue<unknown> | undefined {
    const value = (this._value as any)?.[field];

    switch (true) {
      // When we can with primitives set them to a type.
      case (this._value as any)?.[`_${field}`] !== undefined: {
        const fpPrimitive = deriveFHIRPrimitives(
          value,
          (this._value as any)?.[`_${field}`] as Element | Element[] | undefined
        );

        if (Array.isArray(fpPrimitive)) {
          return new MetaValueV2Array(
            conversion(
              R4,
              getStartingMeta(R4, "Element" as uri) as ElementNode,
              fpPrimitive,
              descendLoc(this, field.toString())
            ),
            descendLoc(this, field.toString())
          );
        } else {
          return new MetaValueV2Singular(
            R4,
            {
              ...(getStartingMeta(R4, "Element" as uri) as ElementNode),
              cardinality: "single",
            } as ElementNode,
            fpPrimitive,
            descendLoc(this, field.toString())
          );
        }
      }

      case value === undefined: {
        return undefined;
      }

      default: {
        // Should handle array for type determination?
        const type = attemptDetermineType(value);
        const loc = descendLoc(this, field.toString());
        if (type) {
          const meta = getStartingMeta(R4, type);
          if (isArray(value)) {
            return new MetaValueV2Array(
              conversion(R4, meta as ElementNode, value, loc),
              loc
            );
          } else {
            return new MetaValueV2Singular(R4, meta as ElementNode, value, loc);
          }
        } else {
          return new NonMetaValue(this._fhirVersion, value, loc);
        }
      }
    }
  }
}

function attemptDetermineType(value: unknown): uri | undefined {
  if (isObject(value) && typeof value.resourceType === "string") {
    return value.resourceType as uri;
  }
  return undefined;
}

export function metaValue<T>(
  metaOptions: Partial<TypeInfo> | undefined = { fhirVersion: R4 },
  value: T | T[],
  location: Location = []
): IMetaValue<NonNullable<T>> | IMetaValueArray<NonNullable<T>> | undefined {
  let { type, fhirVersion } = metaOptions;
  if (!fhirVersion) {
    fhirVersion = R4;
  }

  switch (true) {
    case value === undefined: {
      return undefined;
    }
    case value instanceof SpoofMetaValueV2 ||
      value instanceof MetaValueV2Array ||
      value instanceof MetaValueV2Singular: {
      return value;
    }
    default: {
      // Assign a type automatically if the value is a resourceType
      if (isArray(value)) {
        return new MetaValueV2Array(
          value.map(
            (v, i) =>
              metaValue(metaOptions, v, [
                ...location,
                i,
              ]) as MetaValueV2Singular<unknown>
          ),
          location
        );
      } else {
        type = type ?? attemptDetermineType(value);
        const meta = getStartingMeta(fhirVersion, type as uri);

        if (meta === undefined || type === undefined) {
          return new NonMetaValue(
            fhirVersion,
            value as NonNullable<T>,
            location
          );
        }
        return new MetaValueV2Singular(
          fhirVersion,
          meta,
          value as NonNullable<T>,
          location
        );
      }
    }
  }
}
