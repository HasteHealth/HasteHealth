/* eslint-disable @typescript-eslint/no-explicit-any */

import {
  Loc,
  descend,
  pointer,
  toJSONPointer,
} from "@haste-health/fhir-pointer";
import {
  OperationOutcomeIssue,
  canonical,
  id,
} from "@haste-health/fhir-types/r4/types";
import {
  AllDataTypes,
  AllResourceTypes,
  FHIR_VERSION,
  Resource,
} from "@haste-health/fhir-types/versions";
import { OperationError, outcomeFatal } from "@haste-health/operation-outcomes";

import { ElementLoc, ValidationCTX } from "../types.js";
import { validateProfileElement } from "./element.js";

export async function validateProfile(
  ctx: ValidationCTX,
  profile: Resource<FHIR_VERSION, "StructureDefinition">,
  root: object,
  path_?: Loc<any, any, any>
): Promise<OperationOutcomeIssue[]> {
  const path =
    path_ ??
    pointer(ctx.fhirVersion, profile?.type as AllDataTypes, "id" as id);

  if (profile.derivation !== "constraint") {
    throw new OperationError(
      outcomeFatal(
        "invalid",
        `Profile must be a constraint profile, got ${profile.derivation}`,
        [toJSONPointer(path)]
      )
    );
  }

  const startingLoc = descend(
    descend(
      descend(
        pointer(ctx.fhirVersion, "StructureDefinition", profile.id as id),
        "snapshot"
      ),
      "element"
    ),
    0
  );

  return validateProfileElement(
    ctx,
    profile,
    startingLoc as unknown as ElementLoc,
    root,
    path,
    profile.type
  );
}

/**
 *
 * @param ctx_ Validation CTX
 * @param profileURL Profile canonical to validate
 * @param value the value to check
 * @param path The current path of the value.
 */
export async function validateProfileCanonical(
  ctx: ValidationCTX,
  profileURL: canonical,
  root: object,
  path_?: Loc<any, any, any>
): Promise<OperationOutcomeIssue[]> {
  const profile = await ctx.resolveCanonical(
    ctx.fhirVersion,
    "StructureDefinition",
    profileURL
  );

  const path =
    path_ ??
    pointer(ctx.fhirVersion, profile?.type as AllDataTypes, "id" as id);

  // If we can't resolve profile just ignore for now.
  if (!profile) {
    return [];
  }

  const issues = await validateProfile(ctx, profile, root, path);

  return issues;
}

/**
 * Validates a resources profiles found on meta.profile.
 * @param ctx Validation Context
 * @param resource The resource with profiles to validate
 * @returns Any issues found during profile validation.
 */
export default async function validateResourceProfiles(
  ctx: ValidationCTX,
  resource: Resource<FHIR_VERSION, AllResourceTypes>,
  path_?: Loc<any, any, any>
): Promise<OperationOutcomeIssue[]> {
  const profiles = resource.meta?.profile ?? [];
  const path =
    path_ ??
    pointer(
      ctx.fhirVersion,
      resource.resourceType,
      resource.id ?? ("new" as id)
    );

  const issues = (
    await Promise.all(
      profiles.map((profile) =>
        validateProfileCanonical(ctx, profile, resource, path)
      )
    )
  ).flat();

  return issues;
}
