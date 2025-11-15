/* eslint-disable @typescript-eslint/no-explicit-any */
import { Loc, get, toJSONPointer } from "@haste-health/fhir-pointer";
import {
  ElementDefinition,
  OperationOutcomeIssue,
  uri,
} from "@haste-health/fhir-types/r4/types";
import * as fp from "@haste-health/fhirpath";
import { issueError } from "@haste-health/operation-outcomes";

export function conformsToValue(
  expectedValue: unknown,
  foundValue: unknown
): boolean {
  return JSON.stringify(expectedValue) === JSON.stringify(foundValue);
}

export async function validateFixedValue(
  element: ElementDefinition,
  root: unknown,
  path: Loc<any, any, any>
): Promise<Array<OperationOutcomeIssue>> {
  const expectedValue = (
    await fp.evaluate("value", element, { type: "ElementDefinition" as uri })
  )[0];

  const value = get(path, root);
  if (!expectedValue) return [];

  if (!conformsToValue(expectedValue, value)) {
    return [
      issueError(
        "structure",
        `Value does not conform to fixed value ${JSON.stringify(
          expectedValue
        )}.`,
        [toJSONPointer(path)]
      ),
    ];
  }

  return [];
}
