/* eslint-disable @typescript-eslint/no-explicit-any */
import { Loc, get, toJSONPointer } from "@oxidized-health/fhir-pointer";
import {
  ElementDefinition,
  OperationOutcomeIssue,
  uri,
} from "@oxidized-health/fhir-types/r4/types";
import * as fp from "@oxidized-health/fhirpath";
import { issueError } from "@oxidized-health/operation-outcomes";

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
