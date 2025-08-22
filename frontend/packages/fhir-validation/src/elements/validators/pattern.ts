/* eslint-disable @typescript-eslint/no-explicit-any */
import { Loc, get, toJSONPointer } from "@oxidized-health/fhir-pointer";
import {
  ElementDefinition,
  OperationOutcomeIssue,
  uri,
} from "@oxidized-health/fhir-types/r4/types";
import * as fp from "@oxidized-health/fhirpath";
import { isObject } from "@oxidized-health/meta-value/utilities";
import { issueError } from "@oxidized-health/operation-outcomes";

/**
 * If primitive: it must match exactly the pattern value
 * If a complex object: it must match (recursively) the pattern value
 * If an array: it must match (recursively) the pattern value.
 * @param pattern
 * @param value
 */
export function conformsToPattern(pattern: unknown, value: unknown): boolean {
  if (isObject(pattern)) {
    if (!isObject(value)) return false;
    if (Array.isArray(pattern)) {
      if (!Array.isArray(value)) return false;
      for (const singularPattern of pattern) {
        const valueExists = value.filter((v) =>
          conformsToPattern(singularPattern, v)
        );
        // Per spec as long as a single value matches in the pattern then it's truthy
        if (valueExists.length === 0) return false;
      }
      return true;
    } else {
      const patternKeys = Object.keys(pattern);
      for (const key of patternKeys) {
        if (!conformsToPattern(pattern[key], value[key])) return false;
      }
      return true;
    }
  } else {
    return pattern === value;
  }
}

export async function validatePattern(
  element: ElementDefinition,
  root: unknown,
  path: Loc<any, any, any>
): Promise<Array<OperationOutcomeIssue>> {
  const pattern = (
    await fp.evaluate("pattern", element, { type: "ElementDefinition" as uri })
  )[0];
  const value = get(path, root);
  if (!pattern) return [];

  if (!conformsToPattern(pattern, value)) {
    return [
      issueError(
        "structure",
        `Value does not conform to pattern ${JSON.stringify(pattern)}.`,
        [toJSONPointer(path)]
      ),
    ];
  }

  return [];
}
