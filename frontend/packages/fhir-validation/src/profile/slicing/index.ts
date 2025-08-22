/* eslint-disable @typescript-eslint/no-explicit-any */
import { eleIndexToChildIndices } from "@oxidized-health/codegen/traversal/structure-definition";
import {
  Loc,
  descend,
  get,
  pathMeta,
  pointer,
  toFHIRPath,
  toJSONPointer,
} from "@oxidized-health/fhir-pointer";
import {
  ElementDefinition,
  OperationOutcome,
  OperationOutcomeIssue,
  StructureDefinition,
  canonical,
  id,
  uri,
} from "@oxidized-health/fhir-types/r4/types";
import {
  FHIR_VERSION,
  R4,
  Resource,
} from "@oxidized-health/fhir-types/versions";
import * as fp from "@oxidized-health/fhirpath";
import { metaValue } from "@oxidized-health/meta-value/v2";
import {
  OperationError,
  issueError,
  outcomeError,
} from "@oxidized-health/operation-outcomes";

import { conformsToValue } from "../../elements/validators/fixedValue.js";
import { conformsToPattern } from "../../elements/validators/pattern.js";
import { ElementLoc, ValidationCTX } from "../../types.js";
import { ascendElementLoc, fieldName, isTypeChoice } from "../../utilities.js";
import { validateSingularProfileElement } from "../element.js";
import {
  Discriminator,
  convertPathToElementPointer,
  joinPaths,
  removeTypeOnPath,
} from "../utilities.js";

function isSliced(element: ElementDefinition) {
  return element.slicing !== undefined;
}

type SlicingDescriptor = {
  discriminator: number;
  slices: number[];
};

/**
 * Filter elements for discriminator slice + slicing elements.
 * @param elements
 */
export function getSliceIndices(
  profile: Resource<FHIR_VERSION, "StructureDefinition">,
  elementLoc: ElementLoc
): SlicingDescriptor[] {
  const { parent: elementsLoc, field: index } = ascendElementLoc(elementLoc);
  const elements = get(elementsLoc, profile as StructureDefinition);
  const children = eleIndexToChildIndices(elements, index as number);

  const slices: SlicingDescriptor[] = [];

  let i = 0;
  while (i < children.length) {
    const childIndex = children[i++];
    if (isSliced(elements[childIndex])) {
      const discriminatorIndex = childIndex;
      const sliceIndexes = [];

      while (i < children.length && elements[children[i]].sliceName) {
        sliceIndexes.push(children[i]);
        i++;
      }

      slices.push({
        discriminator: discriminatorIndex,
        slices: sliceIndexes,
      });
    }
  }

  return slices;
}

/**
 * Returns the elementDefinition associated with the given discriminator. This is used to extract the value the slice is expecting.
 * @param discriminator The discriminator to use to distinguish slices.
 * @param elements The elements to search for the slice value.
 * @param currentIndex The current index to search for the slice value.
 * @returns The index of elementdefinition to use to slice the value for the discriminator.
 */
async function findElementDefinitionForSliceDiscriminator(
  ctx: ValidationCTX,
  searchingForElementPath: string,
  elements: ElementDefinition[],
  currentIndex: number,
  parentCurrentElementPath: string | undefined = undefined
): Promise<ElementDefinition | undefined> {
  const element = elements[currentIndex];
  const currentElementPath = parentCurrentElementPath
    ? joinPaths(parentCurrentElementPath, removeTypeOnPath(element.path))
    : removeTypeOnPath(element.path);

  if (searchingForElementPath === currentElementPath) {
    return element;
  }

  // No need for further processing if the discriminator path is not a prefix of the current element path.
  if (searchingForElementPath.startsWith(currentElementPath)) {
    const profiles = element.type
      ?.map((t) => t.profile)
      .flat()
      .filter((t): t is canonical => t !== undefined);

    if (profiles) {
      for (const profile of profiles) {
        const profileSD = await ctx.resolveCanonical(
          ctx.fhirVersion,
          "StructureDefinition",
          profile
        );
        if (!profileSD) {
          throw new OperationError(
            outcomeError("not-found", `Could not resolve profile '${profile}'`)
          );
        }

        const elementInProfile =
          await findElementDefinitionForSliceDiscriminator(
            ctx,
            searchingForElementPath,
            profileSD.snapshot?.element ?? [],
            0,
            // Use current element path as parent path.
            currentElementPath
          );

        if (elementInProfile !== undefined) return elementInProfile;
      }
    }

    const childIndexes = eleIndexToChildIndices(elements, currentIndex);
    for (const childIndex of childIndexes) {
      const elementDefinitionForSliceDiscriminator =
        await findElementDefinitionForSliceDiscriminator(
          ctx,
          searchingForElementPath,
          elements,
          childIndex,
          parentCurrentElementPath
        );
      if (elementDefinitionForSliceDiscriminator !== undefined) {
        return elementDefinitionForSliceDiscriminator;
      }
    }
  }

  return undefined;
}

async function isConformantToSlicesDiscriminator(
  discriminator: Discriminator,
  sliceValueElementDefinition: ElementDefinition,
  root: object,
  locs: Loc<any, any, any>[]
): Promise<Loc<any, any, any>[]> {
  switch (discriminator.type) {
    case "value": {
      const expectedValue = (
        await fp.evaluate("$this.fixed", sliceValueElementDefinition, {
          type: "ElementDefinition" as uri,
        })
      )[0];
      const conformantLocs: Loc<any, any, any>[] = [];

      for (const loc of locs) {
        const value = get(loc, root);
        const evaluation = await fp.evaluate(discriminator.path, value);
        if (
          evaluation.find((v) => conformsToValue(expectedValue, v)) !==
          undefined
        ) {
          conformantLocs.push(loc);
        }
      }
      return conformantLocs;
    }
    case "exists":
    case "pattern": {
      const pattern = (
        await fp.evaluate(
          "$this.pattern",
          metaValue(
            { type: "ElementDefinition" as uri, fhirVersion: R4 },
            sliceValueElementDefinition
          )
        )
      )[0];

      if (!pattern) {
        throw new OperationError(
          outcomeError("not-found", "Could not find a pattern for slice.")
        );
      }

      const conformantLocs: Loc<any, any, any>[] = [];

      for (const loc of locs) {
        const value = get(loc, root);
        const evaluation = await fp.evaluate(discriminator.path, value);

        if (evaluation.find((v) => conformsToPattern(pattern, v))) {
          conformantLocs.push(loc);
        }
      }

      return conformantLocs;
    }
    case "type": {
      const expectedTypes =
        sliceValueElementDefinition.type?.map((t) => t.code) ?? [];

      const conformantLocs: Loc<any, any, any>[] = [];

      for (const loc of locs) {
        const meta = pathMeta(loc);
        const value = await fp.evaluateWithMeta(toFHIRPath(loc), root, {
          fhirVersion: meta?.version,
          type: meta?.type as uri,
        });

        // Map over to use the metavaluesingular.
        const evaluation = (
          await Promise.all(
            value.map((v) => fp.evaluateWithMeta(discriminator.path, v))
          )
        ).flat();

        const type = evaluation?.[0]?.meta()?.type;

        if (type && expectedTypes.includes(type)) {
          conformantLocs.push(loc);
        }
      }

      return conformantLocs;
    }
    case "profile": {
      throw new Error("Not supported");
    }
  }
  throw new Error("invalid");
}

export async function validateSlice(
  elements: ElementDefinition[],
  sliceIndex: number,
  root: object,
  sliceValueLocs: Loc<any, any, any>[]
) {
  for (const sliceValueLoc of sliceValueLocs) {
    const sliceElement = elements[sliceIndex];
    const patternCheck = await fp.evaluate("$this.pattern", sliceElement);
    const sliceValue = get(sliceValueLoc, root);
    const fixedValueCheck = await fp.evaluate("$this.fixed", sliceElement);

    if (patternCheck && !conformsToPattern(patternCheck[0], sliceValue)) {
      throw new OperationError(outcomeError("exception", "pattern mismatch"));
    }
    if (fixedValueCheck) {
      if (JSON.stringify(fixedValueCheck[0]) !== JSON.stringify(sliceValue)) {
        throw new OperationError(
          outcomeError("exception", "fixed value mismatch")
        );
      }

      const childrenIndices = eleIndexToChildIndices(elements, sliceIndex);

      for (const child of childrenIndices) {
        const field = fieldName(elements[child]);
        const fieldLoc = descend(sliceValueLoc, field);
        validateSlice(elements, child, root, [fieldLoc]);
      }
    }
  }
}

async function splitSlicing(
  ctx: ValidationCTX,
  elements: ElementDefinition[],
  sliceDescriptor: SlicingDescriptor,
  root: object,
  locsToCheck: Loc<any, any, any>[]
): Promise<Record<number, Loc<any, any, any>[]>> {
  const sliceSplit = sliceDescriptor.slices.reduce(
    (acc: Record<number, Loc<any, any, any>[]>, sliceIndex) => {
      acc[sliceIndex] = [];
      return acc;
    },
    {}
  );

  // [Serialized as a group] The entries must be adjacent for a given slice.
  const discriminatorElement = elements[sliceDescriptor.discriminator];
  const discriminators = discriminatorElement.slicing?.discriminator ?? [];
  const discriminatorElementPaths = discriminators.map((d) =>
    convertPathToElementPointer(discriminatorElement, d)
  );

  for (const sliceIndex of sliceDescriptor.slices) {
    let sliceValueLocs = locsToCheck;
    for (let d = 0; d < discriminators.length; d++) {
      const discriminator = discriminators[d];
      const sliceDiscriminatorElementValue =
        await findElementDefinitionForSliceDiscriminator(
          ctx,
          discriminatorElementPaths[d],
          elements,
          sliceIndex
        );

      if (!sliceDiscriminatorElementValue) {
        throw new OperationError(
          outcomeError(
            "invalid",
            `Could not find element definition to use for slice discriminator '${discriminatorElementPaths[d]}'.`
          )
        );
      }

      sliceValueLocs = await isConformantToSlicesDiscriminator(
        discriminator,
        sliceDiscriminatorElementValue,
        root,
        sliceValueLocs
      );
    }

    locsToCheck = locsToCheck.filter((l) => !sliceValueLocs.includes(l));
    sliceSplit[sliceIndex] = sliceValueLocs;
  }

  return sliceSplit;
}

function validateSliceCardinality(
  parentLoc: Loc<any, any, any>,
  sliceElement: ElementDefinition,
  paths: Loc<any, any, any>[]
): OperationOutcomeIssue[] {
  const issues: OperationOutcomeIssue[] = [];
  if (paths.length < (sliceElement.min ?? 0)) {
    issues.push(
      issueError(
        "structure",
        `Slice '${sliceElement.sliceName}' does not have the minimum number of values.`,
        [toJSONPointer(parentLoc)]
      )
    );
  }

  if (!isNaN(parseInt(sliceElement.max ?? "1"))) {
    const max = parseInt(sliceElement.max ?? "1");
    if (paths.length > max) {
      issues.push(
        issueError(
          "structure",
          `Slice '${sliceElement.sliceName}' has more than the maximum number of values.`,
          [toJSONPointer(parentLoc)]
        )
      );
    }
  }

  return issues;
}

/**
 * Returns the location for the value to be sliced. Note this could be an array or a singular value (in the case of slicing by type).
 * @param discriminatorElement The element definition that is the discriminator. This defines what method is used for slicing.
 * @param root The value of the root object
 * @param loc Location in the root object of the slice.
 * @returns Location of the value to be sliced against the root.
 */
export function getSliceLocs(
  discriminatorElement: ElementDefinition,
  root: object,
  loc: Loc<any, any, any>
): Loc<any, any, any>[] {
  if (isTypeChoice(discriminatorElement)) {
    return (
      discriminatorElement.type?.map((t) =>
        descend(loc, fieldName(discriminatorElement, t.code))
      ) ?? []
    );
  }
  const valueLoc = descend(loc, fieldName(discriminatorElement));
  const value = get(valueLoc, root);
  if (Array.isArray(value)) {
    return value.map((_v, i) => descend(valueLoc, i));
  } else {
    return [valueLoc];
  }
}

export async function validateSliceDescriptor(
  ctx: ValidationCTX,
  profile: Resource<FHIR_VERSION, "StructureDefinition">,
  sliceDescriptor: SlicingDescriptor,
  root: object,
  loc: Loc<any, any, any>
): Promise<OperationOutcome["issue"]> {
  const snapshot = profile.snapshot?.element ?? [];
  const discriminatorElement = snapshot[sliceDescriptor.discriminator];
  const sliceValueLocs = getSliceLocs(discriminatorElement, root, loc);

  const slices = await splitSlicing(
    ctx,
    snapshot,
    sliceDescriptor,
    root,
    sliceValueLocs
  );

  let issues: OperationOutcome["issue"] = [];
  const snapshotLoc = descend(
    descend(
      pointer(ctx.fhirVersion, "StructureDefinition", profile.id as id),
      "snapshot"
    ),
    "element"
  );

  for (const slice of sliceDescriptor.slices) {
    const sliceLoc = descend(snapshotLoc, slice) as unknown as ElementLoc;
    const sliceElement = get(sliceLoc, profile);

    if (!sliceElement) {
      throw new OperationError(
        outcomeError("not-found", `Slice element not found at '${sliceLoc}'`)
      );
    }

    const type = sliceElement.type ?? [];
    issues = issues.concat(
      validateSliceCardinality(loc, sliceElement, slices[slice])
    );

    if (type.length > 1) {
      throw new OperationError(
        outcomeError("not-supported", "typechoices not supported for slicing.")
      );
    }

    for (const path of slices[slice]) {
      issues = issues.concat(
        await validateSingularProfileElement(
          ctx,
          profile,
          sliceLoc,
          root,
          path,
          type[0].code
        )
      );
    }
  }

  return issues;
}
