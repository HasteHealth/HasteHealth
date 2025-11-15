/* eslint-disable @typescript-eslint/no-explicit-any */
import * as fpointer from "@haste-health/fhir-pointer";
import { ElementDefinition } from "@haste-health/fhir-types/r4/types";
import { FHIR_VERSION, Resource } from "@haste-health/fhir-types/versions";
import { isObject } from "@haste-health/meta-value/utilities";
import {
  OperationError,
  issueError,
  outcomeError,
} from "@haste-health/operation-outcomes";

import { ValidationCTX } from "../types.js";

const REGEX: Record<string, RegExp> = {
  // base64Binary: /^(\s*([0-9a-zA-Z+=]){4}\s*)+$/,
  uuid: /^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$/,
  time: /^([01][0-9]|2[0-3]):[0-5][0-9]:([0-5][0-9]|60)(\.[0-9]+)?$/,
  oid: /^urn:oid:[0-2](\.(0|[1-9][0-9]*))+$/,
  unsignedInt: /^([0]|([1-9][0-9]*))$/,
  positiveInt: /^(\+?[1-9][0-9]*)$/,
  instant:
    /^([0-9]([0-9]([0-9][1-9]|[1-9]0)|[1-9]00)|[1-9]000)-(0[1-9]|1[0-2])-(0[1-9]|[1-2][0-9]|3[0-1])T([01][0-9]|2[0-3]):[0-5][0-9]:([0-5][0-9]|60)(\.[0-9]+)?(Z|(\+|-)((0[0-9]|1[0-3]):[0-5][0-9]|14:00))$/,
  id: /^[A-Za-z0-9\-.]{1,64}$/,
  date: /^([0-9]([0-9]([0-9][1-9]|[1-9]0)|[1-9]00)|[1-9]000)(-(0[1-9]|1[0-2])(-(0[1-9]|[1-2][0-9]|3[0-1]))?)?$/,
  dateTime:
    /^([0-9]([0-9]([0-9][1-9]|[1-9]0)|[1-9]00)|[1-9]000)(-(0[1-9]|1[0-2])(-(0[1-9]|[1-2][0-9]|3[0-1])(T([01][0-9]|2[0-3]):[0-5][0-9]:([0-5][0-9]|60)(\.[0-9]+)?(Z|(\+|-)((0[0-9]|1[0-3]):[0-5][0-9]|14:00)))?)?)?$/,
};

export async function validatePrimitive(
  ctx: ValidationCTX,
  element: ElementDefinition | undefined,
  rootValue: unknown,
  path: fpointer.Loc<object, any, any>,
  type: string
): Promise<Resource<FHIR_VERSION, "OperationOutcome">["issue"]> {
  let value;

  if (isObject(rootValue)) value = fpointer.get(path, rootValue);
  else if (path === fpointer.root(path)) value = rootValue;
  else {
    return [
      issueError(
        "structure",
        `Expected primitive type '${type}' at path '${fpointer.toJSONPointer(
          path
        )}'`,
        [fpointer.toJSONPointer(path)]
      ),
    ];
  }

  switch (type) {
    case "http://hl7.org/fhirpath/System.String":
    case "date":
    case "dateTime":
    case "time":
    case "instant":
    case "id":
    case "string":
    case "xhtml":
    case "markdown":
    case "base64Binary":
    case "uri":
    case "uuid":
    case "canonical":
    case "oid":
    case "url": {
      if (typeof value !== "string") {
        return [
          issueError(
            "structure",
            `Expected primitive type '${type}' at path '${fpointer.toJSONPointer(
              path
            )}'`,
            [fpointer.toJSONPointer(path)]
          ),
        ];
      }
      if (REGEX[type] && !REGEX[type].test(value)) {
        return [
          issueError(
            "value",
            `Invalid value '${value}' at path '${fpointer.toJSONPointer(
              path
            )}'. Value must conform to regex '${REGEX[type]}'`,
            [fpointer.toJSONPointer(path)]
          ),
        ];
      }

      return [];
    }

    case "boolean": {
      if (typeof value !== "boolean") {
        return [
          issueError(
            "structure",
            `Expected primitive type '${type}' at path '${fpointer.toJSONPointer(
              path
            )}'`,
            [fpointer.toJSONPointer(path)]
          ),
        ];
      }
      return [];
    }

    case "code": {
      const strength = element?.binding?.strength;
      const valueSet = element?.binding?.valueSet;
      if (typeof value !== "string") {
        return [
          issueError(
            "structure",
            `Expected primitive type '${type}' at path '${fpointer.toJSONPointer(
              path
            )}'`,
            [fpointer.toJSONPointer(path)]
          ),
        ];
      }

      if (strength === "required" && valueSet && ctx.validateCode) {
        const isValid = await ctx.validateCode(valueSet, value);
        if (!isValid) {
          return [
            issueError(
              "structure",
              `Code '${value}' is not in value set '${valueSet}' at path '${fpointer.toJSONPointer(
                path
              )}'`,
              [fpointer.toJSONPointer(path)]
            ),
          ];
        }
      }

      return [];
    }
    case "integer":
    case "positiveInt":
    case "unsignedInt":
    case "decimal": {
      if (typeof value !== "number") {
        return [
          issueError(
            "structure",
            `Expected primitive type '${type}' at path '${fpointer.toJSONPointer(
              path
            )}'`,
            [fpointer.toJSONPointer(path)]
          ),
        ];
      }
      if (REGEX[type] && !REGEX[type].test(value.toString())) {
        return [
          issueError(
            "value",
            `Invalid value '${value}' at path '${fpointer.toJSONPointer(
              path
            )}'. Value must conform to regex '${REGEX[type]}'`,
            [fpointer.toJSONPointer(path)]
          ),
        ];
      }
      return [];
    }
    default:
      throw new OperationError(
        outcomeError(
          "structure",
          `Unknown primitive type '${type}' at path '${fpointer.toJSONPointer(
            path
          )}'`,
          [fpointer.toJSONPointer(path)]
        )
      );
  }
}
