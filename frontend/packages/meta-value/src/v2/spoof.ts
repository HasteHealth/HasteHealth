import {
  FPPrimitiveNode,
  MetaNode,
} from "@oxidized-health/codegen/generate/meta-data";
import { uri } from "@oxidized-health/fhir-types/lib/generated/r4/types";
import * as r4sets from "@oxidized-health/fhir-types/r4/sets";
import * as r4bsets from "@oxidized-health/fhir-types/r4b/sets";
import { FHIR_VERSION, R4, R4B } from "@oxidized-health/fhir-types/versions";

import {
  IMetaValue,
  IMetaValueArray,
  Location,
  TypeInfo,
} from "../interface.js";
import {
  ElementNode,
  TypeChoiceNode,
  getMeta,
  getStartingMeta,
  resolveContentReference,
  resolveTypeNode,
} from "./meta.js";

function resolve(fhirVersion: FHIR_VERSION, meta: MetaNode, field: string) {
  switch (meta._type_) {
    case "content-reference": {
      return resolve(
        fhirVersion,
        resolveContentReference(fhirVersion, meta),
        field
      );
    }
    case "type": {
      return resolveTypeNode(fhirVersion, meta, meta.type, field);
    }
    case "typechoice":
    case "resource":
    case "primitive-type":
    case "complex-type": {
      return { meta, field };
    }
    case "fp-primitive":
    default: {
      // @ts-ignore
      throw new Error(`Unknown meta type: ${meta._type_}`);
    }
  }
}

function descendMeta(
  fhirVersion: FHIR_VERSION,
  meta: ElementNode,
  field: string
) {
  const nextMeta = getMeta(fhirVersion, meta, field);
  if (nextMeta === undefined) return undefined;
  return resolve(fhirVersion, nextMeta, field);
}

export function isResourceType(version: FHIR_VERSION, type: string) {
  switch (version) {
    case R4: {
      return r4sets.resourceTypes.has(type);
    }
    case R4B: {
      return r4bsets.resourceTypes.has(type);
    }
  }
}

export class SpoofMetaValueV2<T> implements IMetaValue<T> {
  private readonly _fhirVersion: FHIR_VERSION;
  private readonly _meta: ElementNode | TypeChoiceNode | FPPrimitiveNode;
  private readonly _nestedCardinality: "single" | "array";

  constructor(
    fhirVersion: FHIR_VERSION,
    fullCardinality: "single" | "array",
    meta: ElementNode | TypeChoiceNode | FPPrimitiveNode
  ) {
    this._fhirVersion = fhirVersion;
    this._meta = meta;
    this._nestedCardinality = fullCardinality;
  }

  cardinality(): "single" | "array" {
    return this._nestedCardinality;
  }

  isTypeChoice(): boolean {
    if (this._meta._type_ === "typechoice") {
      return true;
    }
    return false;
  }

  types(): TypeInfo[] | undefined {
    if (this._meta._type_ === "typechoice") {
      return Object.values(this._meta.fieldsToType).map((type) => {
        return {
          type: type as uri,
          fhirVersion: this._fhirVersion,
          cardinality: this._meta.cardinality,
        };
      });
    }
    const meta = this.meta();
    return meta ? [meta] : undefined;
  }

  meta(): TypeInfo | undefined {
    if (this._meta._type_ === "typechoice") {
      return undefined;
    }
    return {
      type: this._meta.type,
      fhirVersion: this._fhirVersion,
      cardinality: this._meta.cardinality,
    };
  }
  getValue(): T {
    return {} as T;
  }
  isArray(): this is IMetaValueArray<T> {
    return false;
  }
  toArray(): IMetaValue<T>[] {
    return [this];
  }
  location(): Location | undefined {
    throw new Error("Method not implemented.");
  }
  asType(type: string): IMetaValue<undefined> | undefined {
    if (this.isType(type)) {
      const nextMeta = getStartingMeta(this._fhirVersion, type as uri);
      if (!nextMeta) return undefined;

      return new SpoofMetaValueV2(
        this._fhirVersion,
        this._nestedCardinality,
        nextMeta
      );
    }
  }

  isType(type: string): boolean {
    switch (this._meta._type_) {
      case "resource": {
        // Special Handle for Resource +  Domain Resource Type.
        if (
          this._meta.type === "Resource" ||
          this._meta.type === "DomainResource"
        ) {
          return true;
        }
        return this._meta.type === type;
      }

      case "typechoice": {
        return Object.values(this._meta.fieldsToType).includes(type as uri);
      }
      case "fp-primitive":
      case "primitive-type":
      case "complex-type": {
        return this._meta.type === type;
      }
      default: {
        // @ts-ignore
        throw new Error(`Unknown meta type: ${this._meta._type_}`);
      }
    }
  }

  descend(field: string | number): IMetaValue<unknown> | undefined {
    if (this._meta._type_ === "typechoice") {
      throw new Error("Cannot descend on a type choice");
    }

    if (this._meta._type_ === "fp-primitive") {
      return undefined;
    }

    if (typeof field === "number") {
      if (this._meta.cardinality !== "array") return undefined;
      return new SpoofMetaValueV2(this._fhirVersion, this._nestedCardinality, {
        ...this._meta,
        cardinality: "single",
      });
    }

    const nextMeta = descendMeta(this._fhirVersion, this._meta, field);

    if (!nextMeta?.meta) return undefined;

    return new SpoofMetaValueV2(
      this._fhirVersion,
      // If descending a field from an array that means full cardinality must be array.
      nextMeta.meta.cardinality === "array" ? "array" : this._nestedCardinality,
      nextMeta.meta
    );
  }
  keys(): (string | number)[] {
    throw new Error("Method not implemented.");
  }
}

export default function spoof(
  fhirVersion: FHIR_VERSION,
  type: uri
): IMetaValue<undefined> {
  const meta = getStartingMeta(fhirVersion, type);

  if (!meta) {
    throw new Error(`No meta found for ${type}`);
  }

  return new SpoofMetaValueV2(fhirVersion, "single", meta);
}
