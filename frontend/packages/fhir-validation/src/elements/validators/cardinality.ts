/* eslint-disable @typescript-eslint/no-explicit-any */
import { Loc, get, toJSONPointer } from "@haste-health/fhir-pointer";
import {
  ElementDefinition,
  OperationOutcomeIssue,
} from "@haste-health/fhir-types/r4/types";
import { issueError } from "@haste-health/operation-outcomes";

/**
 * Returns whether the value should be respresented as an array given max cardinality
 * @param max either a string number or *
 * @returns
 */
function isArray(element: ElementDefinition): boolean {
  // If base is an array should check that first (because profile could restrict to singular value).
  if (!element.path.includes(".")) return false;
  const base = element.base?.max ?? "1";
  const max = element.max ?? "1";

  return base === "*" || max === "*" || parseInt(base) > 1 || parseInt(max) > 1;
}

function getMin(elementDefinition: ElementDefinition) {
  return elementDefinition.min ?? 0;
}

function getMax(elementDefinition: ElementDefinition) {
  return elementDefinition.max ?? "1";
}

export function validateCardinality(
  element: ElementDefinition,
  root: object,
  path: Loc<any, any, any>
): Array<OperationOutcomeIssue> {
  const value = get(path, root);

  const max = getMax(element);
  const min = getMin(element);

  // Value could be undefined if min is allowed to be zero.
  if (value === undefined && min === 0) return [];
  const isElementAnArray = isArray(element);

  if (Array.isArray(value) !== isElementAnArray) {
    return [
      issueError(
        "structure",
        `Element is expected to be ${
          isElementAnArray ? "an array" : "a singular value"
        }.`,
        [toJSONPointer(path)]
      ),
    ];
  }
  if (isElementAnArray) {
    if (value.length < min) {
      return [
        issueError(
          "structure",
          `Element is expected to have at least ${min} items.`,
          [toJSONPointer(path)]
        ),
      ];
    }
    if (max !== "*" && value > parseInt(max))
      return [
        issueError(
          "structure",
          `Element is expected to have at most ${max} items.`,
          [toJSONPointer(path)]
        ),
      ];

    return [];
  } else {
    if (min > 0 && value === undefined) {
      return [
        issueError(
          "structure",
          `Element is expected to have at least ${min} items.`,
          [toJSONPointer(path)]
        ),
      ];
    }
    return [];
  }
}
