/* eslint-disable @typescript-eslint/no-explicit-any */

import { eleIndexToChildIndices } from "@haste-health/codegen/traversal/structure-definition";
import { Loc, descend, get, toJSONPointer } from "@haste-health/fhir-pointer";
import {
  ElementDefinition,
  OperationOutcomeIssue,
  StructureDefinition,
  uri,
} from "@haste-health/fhir-types/r4/types";
import { FHIR_VERSION, Resource } from "@haste-health/fhir-types/versions";
import { isPrimitiveType } from "@haste-health/meta-value/utilities";
import {
  OperationError,
  issueError,
  outcomeFatal,
} from "@haste-health/operation-outcomes";

import { validatePrimitive } from "../elements/primitive.js";
import { validateCardinality } from "../elements/validators/cardinality.js";
import { validateFixedValue } from "../elements/validators/fixedValue.js";
import { validatePattern } from "../elements/validators/pattern.js";
import { ElementLoc, ValidationCTX } from "../types.js";
import {
  ascendElementLoc,
  getFoundFieldsForElement,
  isElementRequired,
} from "../utilities.js";
import { validateProfileCanonical } from "./index.js";
import { getSliceIndices, validateSliceDescriptor } from "./slicing/index.js";

/**
 * For descendants we want to ignore those slices which are validated seperately.
 * This is done by filtering off sliceName and slicing property.
 *
 * @param elements list of ElementDefinitions to filter
 * @param indices The indices to filter out of the list if they are a part of a lice.
 * @returns Indices that are not a part of a slice.
 */
function ignoreSliceElements(
  elements: ElementDefinition[],
  indices: number[]
): number[] {
  return indices.filter((index) => {
    const element = elements[index];
    return element.sliceName === undefined && element.slicing === undefined;
  });
}

/**
 * Check if the element is constrained to profiles type.
 * @param element ElementDefinition to check
 * @param type The type found on the element.
 * @returns true|false as to whether the element is constrained to the type.
 */
function validateTypeIfMultipleTypesConstrained(
  ctx: ValidationCTX,
  element: ElementDefinition,
  type: uri
): boolean {
  if (element.type) {
    if (element.type.find((t) => t.code === type) !== undefined) {
      return true;
    } else if (
      type === "Element" &&
      element.type.find((t) => isPrimitiveType(ctx.fhirVersion, t.code))
    ) {
      return true;
    }
    return false;
  }
  return true;
}

export async function validateSingularProfileElement(
  ctx: ValidationCTX,
  profile: Resource<FHIR_VERSION, "StructureDefinition">,
  elementLoc: ElementLoc,
  root: object,
  path: Loc<any, any, any>,
  type: uri
) {
  const { parent: elementsLoc, field: elementIndex } =
    ascendElementLoc(elementLoc);
  const elements = get(elementsLoc, profile as StructureDefinition);

  const element = get(elementLoc, profile as StructureDefinition);
  if (!element) {
    throw new OperationError(
      outcomeFatal(
        "not-found",
        `Unable to resolve element at location '${toJSONPointer(elementLoc)}'`,
        [toJSONPointer(path)]
      )
    );
  }

  const value = get(path, root);

  const foundFields: Set<string> =
    profile.kind === "resource" && elementIndex === 0
      ? new Set(["resourceType"])
      : new Set([]);

  // Slice Validation
  const sliceIndices = getSliceIndices(profile, elementLoc);
  const sliceIssues = (
    await Promise.all(
      sliceIndices.map((sliceIndex) =>
        validateSliceDescriptor(ctx, profile, sliceIndex, root, path)
      )
    )
  ).flat();
  sliceIndices.forEach((sliceIndex) => {
    const discriminatorElement = elements[sliceIndex.discriminator];
    const fields = getFoundFieldsForElement(ctx, discriminatorElement, value);
    fields.forEach((f) => foundFields.add(f.field));
  });

  if (sliceIssues.length > 0) {
    return sliceIssues;
  }
  // [END] Slice Validation

  const children = ignoreSliceElements(
    elements,
    eleIndexToChildIndices(elements, elementIndex as number)
  );

  const profileUrlsToValidate =
    element.type?.find((t) => t.code === type)?.profile ?? [];

  for (let i = 0; i < profileUrlsToValidate.length; i++) {
    const profileIssues = await validateProfileCanonical(
      ctx,
      profileUrlsToValidate[i],
      root,
      path
    );
    // Only need one profile to be conformant so continue if invalid and break if valid.
    if (profileIssues.length === 0) break;
    else if (i === profileUrlsToValidate.length - 1) {
      return profileIssues;
    }
  }

  // Profile can further constrain typechoices check that here.
  if (!validateTypeIfMultipleTypesConstrained(ctx, element, type)) {
    throw new OperationError(
      outcomeFatal(
        "invalid",
        `Valid types: '${element.type
          ?.map((t) => t.code)
          .join(",")}' is not constrained to type '${type}'`,
        [toJSONPointer(path)]
      )
    );
  }

  const patternIssues = await validatePattern(element, root, path);
  if (patternIssues.length > 0) {
    return patternIssues;
  }

  const fixedValueIssues = await validateFixedValue(element, root, path);
  if (fixedValueIssues.length > 0) {
    return fixedValueIssues;
  }

  let issues: OperationOutcomeIssue[] = [];

  if (children.length > 0) {
    for (const childIndex of children) {
      const childElement = elements[childIndex];
      const childElementLoc = descend(elementsLoc, childIndex);
      const fields = getFoundFieldsForElement(ctx, childElement, value);
      fields.forEach((f) => foundFields.add(f.field));

      // Confirm if no fields are found and the element is required then issue an error.
      if (isElementRequired(childElement) && fields.length === 0) {
        issues = issues.concat([
          issueError(
            "structure",
            `Missing required field '${
              childElement.path
            }' at path '${toJSONPointer(path)}'`,
            [toJSONPointer(path)]
          ),
        ]);
      }

      for (const field of fields) {
        const childValueLoc = descend(path, field.field);
        issues = issues.concat(
          await validateProfileElement(
            ctx,
            profile,
            childElementLoc as unknown as ElementLoc,
            root,
            childValueLoc,
            field.type
          )
        );
      }
    }
    const additionalFields = new Set(Object.keys(value)).difference(
      foundFields
    );

    if (additionalFields.size > 0) {
      issues = issues.concat([
        issueError(
          "structure",
          `Additional fields found at path '${toJSONPointer(
            path
          )}': '${Array.from(additionalFields).join(", ")}'`,
          [toJSONPointer(path)]
        ),
      ]);
    }
  } else if (type === "code") {
    issues = issues.concat(
      await validatePrimitive(ctx, element, root, path, type)
    );
  }

  return issues;
}

/**
 * Because profile is merely a constraint on the base we only need to validate select pieces.
 *
 * [ALLOWED]:
 * --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
 * Restricting the cardinality of the element; e.g. the base might allow 0..*, and a particular application might support 1..2
 * Ruling out use of an element by setting its maximum cardinality to 0
 * Restricting the contents of an element to a single fixed value
 * Making additional constraints on the content of nested elements within the resource (expressed as FHIRPath statements)
 * Restricting the types for an element that allows multiple types
 * Requiring a typed element or the target of a resource reference to conform to another structure profile (declared in the same profile, or elsewhere)
 * Specifying a binding to a different terminology value set (see below)
 * Providing refined definitions, comments/usage notes and examples for the elements defined in a Resource to reflect the usage of the element within the context of the Profile
 * Providing more specific or additional mappings (e.g. to HL7 V2 icon or HL7 v3 icon) for the resource when used in a particular context
 * Declaring that one or more elements in the structure must be 'supported' (see below)
 * Restricting / clarifying a mapping by providing a new mapping with the same identity, which means that the new mapping replaces a mapping with the same identity in the element being profiled
 * --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
 *
 * [DISSALLOWED]:
 * --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
 * Profiles cannot break the rules established in the base specification (e.g. cardinality as described above)
 * Profiles cannot specify default values or meanings for elements defined in the base specification (note that datatypes and resources do not
 *   define default values at all, but default values may be defined for logical models
 * Profiles cannot change the name of elements defined in the base specification, or add new elements
 * It must be safe to process a resource without knowing the profile
 * --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
 * @param ctx Validation CTX
 * @param profile Profile SD
 * @param elementLoc Current ElementDefinition validating
 * @param root Root value to be validated
 * @param path Path inside of value to validate
 * @param type
 * @returns
 */
export async function validateProfileElement(
  ctx: ValidationCTX,
  profile: Resource<FHIR_VERSION, "StructureDefinition">,
  elementLoc: ElementLoc,
  root: object,
  path: Loc<any, any, any>,
  type: uri
): Promise<OperationOutcomeIssue[]> {
  const value = get(path, root);
  const element = get(elementLoc, profile as StructureDefinition);

  if (!element) {
    throw new OperationError(
      outcomeFatal(
        "not-found",
        `Unable to resolve element at location '${toJSONPointer(elementLoc)}'`,
        [toJSONPointer(path)]
      )
    );
  }

  // [Slicing Validation]
  let issues: OperationOutcomeIssue[] = [];

  // [Cardinality validation]
  issues = issues.concat(validateCardinality(element, root, path));

  switch (true) {
    case Array.isArray(value): {
      issues = issues.concat(
        (
          await Promise.all(
            value.map((_v, index) =>
              validateSingularProfileElement(
                ctx,
                profile,
                elementLoc,
                root,
                descend(path, index),
                type
              )
            )
          )
        ).flat()
      );
      break;
    }
    default: {
      issues = issues.concat(
        await validateSingularProfileElement(
          ctx,
          profile,
          elementLoc,
          root,
          path,
          type
        )
      );
      break;
    }
  }

  return issues;
}
