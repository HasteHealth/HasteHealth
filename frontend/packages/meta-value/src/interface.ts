import * as r4 from "@haste-health/fhir-types/r4/types";
import { FHIR_VERSION } from "@haste-health/fhir-types/versions";

export type Location = (string | number)[];

export interface TypeInfo {
  fhirVersion: FHIR_VERSION;
  type: r4.uri;
  cardinality: "single" | "array";
}

export interface IMetaValue<T> {
  meta(): TypeInfo | undefined;
  getValue(): T;
  isArray(): this is IMetaValueArray<T>;
  location(): Location | undefined;
  descend(field: string | number): IMetaValue<unknown> | undefined;
  isType(type: string): boolean;
  asType(type: string): IMetaValue<unknown> | undefined;
  keys(): (string | number)[];
}

export interface IMetaValueArray<T> extends IMetaValue<T[]> {
  toArray(): IMetaValue<unknown>[];
}
