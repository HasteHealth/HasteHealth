/* eslint-disable @typescript-eslint/no-explicit-any */

import { Loc } from "@haste-health/fhir-pointer";
import {
  OperationOutcomeIssue,
  uri,
} from "@haste-health/fhir-types/lib/generated/r4/types";
import {
  AllResourceTypes,
  FHIR_VERSION,
  Resource,
} from "@haste-health/fhir-types/versions";
import { isObject } from "@haste-health/meta-value/utilities";

import validateResourceProfiles from "./profile/index.js";
import structuralValidation from "./structural/index.js";
import { ValidationCTX } from "./types.js";

export default async function validate(
  ctx: ValidationCTX,
  type: uri,
  root: unknown,
  path_?: Loc<any, any, any>
): Promise<OperationOutcomeIssue[]> {
  let issues = await structuralValidation(ctx, type, root, path_);

  if (isObject(root) && "resourceType" in root) {
    const profileIssues = await validateResourceProfiles(
      ctx,
      root as unknown as Resource<FHIR_VERSION, AllResourceTypes>,
      path_
    );
    issues = issues.concat(profileIssues);
  }

  return issues;
}
