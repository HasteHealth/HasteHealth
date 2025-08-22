/* eslint-disable @typescript-eslint/no-explicit-any */
import type {
  ContentReferenceNode,
  ElementNode,
  FPPrimitiveNode,
  MetaNode,
  MetaV2Compiled,
  TypeChoiceNode,
  TypeNode,
} from "@oxidized-health/codegen/generate/meta-data";
import { uri } from "@oxidized-health/fhir-types/r4/types";
import { FHIR_VERSION, R4, R4B } from "@oxidized-health/fhir-types/versions";

import {
  FHIRPathPrimitive as FHIRPrimitive,
  RawPrimitive,
  isObject,
} from "../utilities.js";
import _r4Meta from "./generated/meta/r4.js";
import _r4bMeta from "./generated/meta/r4b.js";

const R4_META: MetaV2Compiled = _r4Meta as unknown as MetaV2Compiled;
const R4B_META: MetaV2Compiled = _r4bMeta as unknown as MetaV2Compiled;

export { MetaNode, ElementNode, TypeNode, TypeChoiceNode };

function getGlobalMeta(fhirVersion: FHIR_VERSION): MetaV2Compiled {
  switch (fhirVersion) {
    case R4:
      return R4_META;
    case R4B:
      return R4B_META;
    default:
      throw new Error(`Unsupported FHIR version: ${fhirVersion}`);
  }
}

export function getStartingMeta(
  fhirVersion: FHIR_VERSION,
  type: uri
): ElementNode | FPPrimitiveNode | undefined {
  return getGlobalMeta(fhirVersion)[type]?.[0] as ElementNode | undefined;
}

export function getMeta(
  fhirVersion: FHIR_VERSION,
  meta: ElementNode | FPPrimitiveNode,
  field: string
): ElementNode | TypeNode | TypeChoiceNode | undefined {
  if (meta._type_ === "fp-primitive") return undefined;

  const globalMeta = getGlobalMeta(fhirVersion);
  const fieldIndex = meta.properties?.[field];

  if (!fieldIndex) {
    return undefined;
  }

  return globalMeta[meta.base][fieldIndex] as
    | ElementNode
    | TypeNode
    | TypeChoiceNode;
}

export function resolveTypeNode(
  fhirVersion: FHIR_VERSION,
  meta: TypeNode | TypeChoiceNode,
  type: uri,
  field: string
) {
  return {
    field,
    base: type,
    meta: {
      ...(getStartingMeta(fhirVersion, type) as ElementNode),
      cardinality: meta.cardinality,
    },
  };
}

function resolveResourceType(
  fhirVersion: FHIR_VERSION,
  meta: TypeNode,
  value: any,
  field: string
) {
  const type =
    isObject(value) && isObject(value?.[field])
      ? (value?.[field].resourceType as uri)
      : meta.type;

  return resolveTypeNode(fhirVersion, meta, type, field);
}

export function resolveContentReference(
  fhirVersion: FHIR_VERSION,
  meta: ContentReferenceNode
) {
  return getGlobalMeta(fhirVersion)[meta.base][meta.reference];
}

export function resolveMeta<T>(
  fhirVersion: FHIR_VERSION,
  meta: MetaNode,
  value: T | FHIRPrimitive<RawPrimitive>,
  field: string
): { meta: ElementNode; field: string } | undefined {
  switch (true) {
    case meta === undefined: {
      return undefined;
    }
    case meta._type_ === "type": {
      // Special handling for Bundle.entry.resource which is abstract Resource;
      // Descend into the resourceType field to get the actual type.
      if (meta.type === "Resource" || meta.type === "DomainResource") {
        return resolveResourceType(fhirVersion, meta, value, field);
      } else {
        return resolveTypeNode(fhirVersion, meta, meta.type, field);
      }
    }
    case meta._type_ === "typechoice": {
      for (const typeChoiceField of Object.keys(meta.fieldsToType)) {
        if ((value as any)?.[typeChoiceField] !== undefined) {
          return resolveTypeNode(
            fhirVersion,
            meta,
            meta.fieldsToType[typeChoiceField],
            typeChoiceField
          );
        }
      }
      return undefined;
    }
    case meta._type_ === "content-reference": {
      const nextMeta = resolveMeta(
        fhirVersion,
        resolveContentReference(fhirVersion, meta),
        value,
        field
      );
      if (!nextMeta)
        throw new Error(
          `Could not derive meta from content-reference '${meta.definition.path}'`
        );

      return nextMeta;
    }
    case meta._type_ === "complex-type":
    case meta._type_ === "resource":
    case meta._type_ === "primitive-type": {
      return { meta, field };
    }
    default: {
      // @ts-ignore
      throw new Error(`Unknown meta type: ${meta._type_}`);
    }
  }
}
