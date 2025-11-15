/* eslint-disable @typescript-eslint/no-explicit-any */
import { Loc, ascend } from "@haste-health/fhir-pointer";
import { resourceTypes } from "@haste-health/fhir-types/r4/sets";
import {
  ElementDefinition,
  StructureDefinition,
  uri,
} from "@haste-health/fhir-types/r4/types";
import { FHIR_VERSION, Resource } from "@haste-health/fhir-types/versions";
import { resolveTypeToCanonical } from "@haste-health/meta-value";
import { isObject, isPrimitiveType } from "@haste-health/meta-value/utilities";
import { OperationError, outcomeFatal } from "@haste-health/operation-outcomes";

import { ElementLoc, ValidationCTX } from "./types.js";

function capitalize(str: string) {
  return str.charAt(0).toUpperCase() + str.slice(1);
}

export function notNullable<T, Z extends T | undefined>(
  value: Z
): value is NonNullable<Z> {
  if (value === undefined || value === null) return false;
  return true;
}

/**
 * Per spec the ID can be used to determine if typechoice
 * See https://hl7.org/fhir/R4/elementdefinition.html#id
 * For type choice elements, the id reflects the type slice. e.g. For path = Patient.deceasedBoolean, the id is Patient.deceased[x]:deceasedBoolean
 * @param element
 * @returns
 */
export function isTypeChoice(element: ElementDefinition) {
  return element.id?.endsWith("[x]");
}

export function isResourceType(type: string) {
  return resourceTypes.has(type);
}

export function fieldName(elementDefinition: ElementDefinition, type?: string) {
  const field = elementDefinition.path.split(".").pop() as string;
  if (isTypeChoice(elementDefinition)) {
    if (!type)
      throw new Error("deriving field from typechoice requires a type");
    return field.replace("[x]", capitalize(type));
  }
  return field;
}

export async function resolveTypeToStructureDefinition(
  ctx: ValidationCTX,
  type: uri
): Promise<Resource<FHIR_VERSION, "StructureDefinition">> {
  const canonical = resolveTypeToCanonical(ctx.fhirVersion, type);

  if (!canonical) {
    throw new OperationError(
      outcomeFatal(
        "structure",
        `Unable to resolve canonical for type '${type}'`,
        []
      )
    );
  }

  const sd = await ctx.resolveCanonical(
    ctx.fhirVersion,
    "StructureDefinition",
    canonical
  );

  if (!sd)
    throw new OperationError(
      outcomeFatal(
        "structure",
        `Unable to resolve canonical for type '${type}'`
      )
    );

  return sd;
}

export function ascendElementLoc(loc: ElementLoc): {
  parent: Loc<StructureDefinition, ElementDefinition[]>;
  field: NonNullable<keyof ElementDefinition[]>;
} {
  const parent = ascend(loc);

  if (!parent) {
    throw new OperationError(
      outcomeFatal("invalid", `Invalid element path ${loc}`)
    );
  }

  return parent;
}

export type PropertyAndType = { field: string; type: uri };

function _findBaseFieldAndType(
  element: ElementDefinition,
  value: object
): PropertyAndType | undefined {
  if (element.contentReference) {
    const baseField = fieldName(element);
    if (baseField in value) {
      return {
        field: fieldName(element),
        type: (element.type?.[0].code ?? "") as uri,
      };
    }
  }

  for (const type of element.type?.map((t) => t.code) ?? []) {
    const field = fieldName(element, type);
    if (field in value) {
      return { field, type };
    }
  }
}

export function isElementRequired(element: ElementDefinition) {
  return (element.min ?? 0) > 0;
}

/**
 * Returns fields associated to an element. Because it could be a primitive it may result in multiple fields.
 * IE _field for Element values for field primitive value.
 *
 * @param element The ElementDefinition to find fields for
 * @param value Value to check for fields.
 * @returns
 */
export function getFoundFieldsForElement(
  ctx: ValidationCTX,
  element: ElementDefinition,
  value: unknown
): PropertyAndType[] {
  if (!isObject(value)) return [];
  const properties: PropertyAndType[] = [];
  const base = _findBaseFieldAndType(element, value);

  if (base) {
    properties.push(base);
    const { field, type } = base;
    if (isPrimitiveType(ctx.fhirVersion, type)) {
      const primitiveElementField = {
        field: `_${field}`,
        type: "Element" as uri,
      };
      if (`_${field}` in value) properties.push(primitiveElementField);
    }
  } else {
    // Check for primitive extensions when non existent values
    const primitives =
      element.type?.filter((type) =>
        isPrimitiveType(ctx.fhirVersion, type.code)
      ) ?? [];
    for (const primType of primitives) {
      if (`_${fieldName(element, primType.code)}` in value) {
        const primitiveElementField = {
          field: `_${fieldName(element, primType.code)}`,
          type: "Element" as uri,
        };
        properties.push(primitiveElementField);
      }
    }
  }
  return properties;
}
