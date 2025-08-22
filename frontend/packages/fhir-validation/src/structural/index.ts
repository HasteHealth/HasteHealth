/* eslint-disable @typescript-eslint/no-explicit-any */

import {
  ContentReferenceNode,
  FPPrimitiveNode,
  TypeChoiceNode,
  TypeNode,
} from "@oxidized-health/codegen/generate/meta-data";
import {
  Loc,
  descend,
  get,
  pointer,
  toJSONPointer,
} from "@oxidized-health/fhir-pointer";
import {
  ElementDefinition,
  OperationOutcome,
  OperationOutcomeIssue,
  Reference,
  id,
  uri,
} from "@oxidized-health/fhir-types/r4/types";
import {
  AllDataTypes,
  AllResourceTypes,
  FHIR_VERSION,
  Resource,
} from "@oxidized-health/fhir-types/versions";
import {
  ElementNode,
  getMeta,
  getStartingMeta,
  resolveMeta,
} from "@oxidized-health/meta-value/meta";
import { isObject } from "@oxidized-health/meta-value/utilities";
import {
  OperationError,
  issueError,
  outcomeFatal,
} from "@oxidized-health/operation-outcomes";

import { validatePrimitive } from "../elements/primitive.js";
import { validateCardinality } from "../elements/validators/cardinality.js";
import { validateFixedValue } from "../elements/validators/fixedValue.js";
import { validatePattern } from "../elements/validators/pattern.js";
import { ValidationCTX } from "../types.js";
import { isElementRequired, isResourceType } from "../utilities.js";

function isReferenceConformantToStructureDefinition(
  sd: Resource<FHIR_VERSION, "StructureDefinition">,
  value: Reference
): boolean {
  switch (true) {
    case sd?.type === "Resource" || sd?.type === "DomainResource": {
      return true;
    }
    case value.reference !== undefined: {
      const resourceType = value.reference.split("/")[0];

      switch (true) {
        // Could be ref in bundle so skip for now.
        case !isResourceType(resourceType): {
          return true;
        }
        case resourceType === sd?.type: {
          // If Reference also has type need to validate that as well
          if (value.type) return resourceType === value.type;
          return true;
        }
        default: {
          return false;
        }
      }
    }
    case value.type && value === sd?.type: {
      return true;
    }
    case value.type === undefined && value.reference === undefined: {
      return true;
    }
    default: {
      return false;
    }
  }
}

/**
 * Validate the Reference is conformant to the profile (only confirms the type)
 * @param ctx ValidationCTX
 * @param element ElementDefinition to confirm Reference types
 * @param root
 * @param path
 * @returns
 */
async function validateReferenceTypeConstraint(
  ctx: ValidationCTX,
  element: ElementDefinition,
  root: object,
  path: Loc<object, any, any>
): Promise<OperationOutcome["issue"]> {
  // [Note] because element should already be validated as reference this can be considered safe?
  const value = get(path, root) as Reference | undefined;
  if (!value) return [];

  const referenceProfiles = element.type?.find(
    (t) => t.code === "Reference"
  )?.targetProfile;
  if (referenceProfiles === undefined || referenceProfiles?.length === 0)
    return [];
  for (const profileURI of referenceProfiles ?? []) {
    const sd = await ctx.resolveCanonical(
      ctx.fhirVersion,
      "StructureDefinition",
      profileURI
    );
    if (!sd) {
      throw new OperationError(
        outcomeFatal(
          "not-found",
          `Could not find profile: '${profileURI}' to validate reference`
        )
      );
    }
    if (isReferenceConformantToStructureDefinition(sd, value)) {
      return [];
    }
  }

  return [
    issueError(
      "structure",
      `Expected reference to be constrained by one of the following profiles '${referenceProfiles?.join(
        ", "
      )}' at path '${toJSONPointer(path)}' found reference of type '${
        value.type ? value.type : value.reference?.split("/")[0]
      }' instead.`,
      [toJSONPointer(path)]
    ),
  ];
}

function checkResourceType(
  type: uri,
  root: Record<string, unknown>,
  path: Loc<any, any, any>
) {
  const resource = root as unknown as Resource<FHIR_VERSION, AllResourceTypes>;
  // Verify the resourceType aligns.
  if (get(descend(path, "resourceType"), resource) !== type) {
    return [
      issueError(
        "invalid",
        `ResourceType '${get(
          descend(path, "resourceType"),
          resource
        )}' does not match expected type '${type}'`,
        [toJSONPointer(path)]
      ),
    ];
  }
  return [];
}

type FieldMeta = {
  field: string;
  meta: ElementNode | (FPPrimitiveNode & { definition: ElementDefinition });
};

function findFields(
  ctx: ValidationCTX,
  meta:
    | ElementNode
    | TypeNode
    | TypeChoiceNode
    | FPPrimitiveNode
    | ContentReferenceNode,
  value: object,
  field: string
): FieldMeta[] {
  switch (meta._type_) {
    case "fp-primitive":
    case "resource":
    case "complex-type": {
      if (!("definition" in meta))
        throw new Error("Definition not found in meta");

      if (field in value) {
        return [{ field, meta }];
      }

      return [];
    }
    case "type": {
      return findFields(
        ctx,
        {
          ...getStartingMeta(ctx.fhirVersion, meta.type),
          definition: meta.definition,
        } as ElementNode,
        value,
        field
      );
    }
    case "primitive-type": {
      const foundFields: FieldMeta[] = [];
      if (field in value) {
        foundFields.push({ field, meta });
      }
      // Because Primitives have Element data under seperate key we must check multiple fields.
      if (`_${field}` in value) {
        const elementMeta = getStartingMeta(
          ctx.fhirVersion,
          "Element" as uri
        ) as ElementNode;
        foundFields.push({
          field: `_${field}`,
          meta: {
            ...elementMeta,
            // Reet the min max cardinaltiy to the field definition.
            definition: {
              ...elementMeta.definition,
              // Set both path and max,min,base to field definition to avoid issues with cardinality validation.
              path: meta.definition.path,
              max: meta.definition.max,
              min: meta.definition.min,
              base: meta.definition.base,
            },
          },
        });
      }

      return foundFields;
    }

    case "typechoice": {
      for (const fieldToType of Object.keys(meta.fieldsToType)) {
        const foundFields = findFields(
          ctx,
          {
            ...getStartingMeta(ctx.fhirVersion, meta.fieldsToType[fieldToType]),
            definition: meta.definition,
          } as ElementNode,
          value,
          fieldToType
        );
        if (foundFields.length > 0) return foundFields;
      }
      return [];
    }

    case "content-reference": {
      const nextMeta = resolveMeta(ctx.fhirVersion, meta, value, field);
      if (!nextMeta)
        throw new OperationError(
          outcomeFatal(
            "not-found",
            `Could not derive meta from content-reference '${meta.definition.path}'`
          )
        );

      return findFields(
        ctx,
        {
          ...nextMeta.meta,
          cardinality: meta.cardinality,
          definition: {
            ...nextMeta.meta.definition,
            min: meta.definition.min,
            max: meta.definition.max,
          },
        },
        value,
        field
      );
    }

    default: {
      // @ts-ignore
      throw new Error(`Unknown meta type: ${meta._type_}`);
    }
  }
}

/**
 * Validates singular complex item.
 * @param ctx
 * @param meta
 * @param root
 * @param path
 */
async function validateComplex(
  ctx: ValidationCTX,
  meta: ElementNode,
  root: object,
  path: Loc<any, any, any>
) {
  const value = get(path, root);
  if (!isObject(value)) {
    return [
      issueError(
        "structure",
        `Value must be an object when validating '${
          meta.type
        }'. Instead found type of '${typeof value}'`,
        [toJSONPointer(path)]
      ),
    ];
  }
  switch (meta._type_) {
    case "resource":
    case "complex-type": {
      const foundFields: Set<string> =
        meta._type_ === "resource" ? new Set(["resourceType"]) : new Set([]);

      const issues = (
        await Promise.all(
          Object.keys(meta.properties).map(async (property) => {
            const childMeta = getMeta(ctx.fhirVersion, meta, property);
            if (!childMeta) {
              throw new OperationError(
                outcomeFatal(
                  "not-found",
                  `Could not find meta for property '${property}' on type '${meta.type}'`
                )
              );
            }

            // Because Primitives can be extended under seperate key we must check multiple fields.
            const fields = findFields(ctx, childMeta, value, property);
            fields.forEach((f) => foundFields.add(f.field));

            if (
              isElementRequired(childMeta.definition) &&
              fields.length === 0
            ) {
              return [
                issueError(
                  "structure",
                  `Missing required field at path '${toJSONPointer(
                    descend(path, property)
                  )}'`,
                  [toJSONPointer(path)]
                ),
              ];
            }

            return (
              await Promise.all(
                fields.map((child) =>
                  validateElement(
                    ctx,
                    child.meta,
                    root,
                    descend(path, child.field)
                  )
                )
              )
            ).flat();
          })
        )
      ).flat();

      // Check for additional fields
      const additionalFields = new Set(Object.keys(value)).difference(
        foundFields
      );

      if (additionalFields.size > 0) {
        issues.push(
          issueError(
            "structure",
            `Additional fields found at path '${toJSONPointer(
              path
            )}': '${Array.from(additionalFields).join(", ")}'`,
            [toJSONPointer(path)]
          )
        );
      }

      return issues;
    }
    default: {
      throw new OperationError(outcomeFatal("invalid", "Invalid Meta type."));
    }
  }
}

/**
 * Validate cardinality for the element.
 * @param ctx Validation context.
 * @param meta Meta of the element to be validated.
 * @param root
 * @param path
 * @returns
 */
async function validateElement(
  ctx: ValidationCTX,
  meta: ElementNode | (FPPrimitiveNode & { definition: ElementDefinition }),
  root: object,
  path: Loc<any, any, any>
) {
  const value = get(path, root);
  const issues = validateCardinality(meta.definition, root, path);
  if (issues.length > 0) return issues;

  switch (true) {
    // If min is zero than allow undefined.
    case value === undefined: {
      return [];
    }
    case Array.isArray(value): {
      return (
        await Promise.all(
          (value || []).map((_v: any, i: number) => {
            return validateElementSingular(ctx, meta, root, descend(path, i));
          })
        )
      ).flat();
    }
    // Means single item.
    default: {
      return validateElementSingular(ctx, meta, root, path);
    }
  }
}

/**
 * Validates a singular item.
 * @param ctx Validation ctx which includes the version.
 * @param meta
 * @param root
 * @param path_
 * @returns
 */
async function validateElementSingular(
  ctx: ValidationCTX,
  meta: ElementNode | (FPPrimitiveNode & { definition: ElementDefinition }),
  root: unknown,
  path: Loc<any, any, any>
): Promise<OperationOutcomeIssue[]> {
  const patternIssues = await validatePattern(meta.definition, root, path);
  if (patternIssues.length > 0) {
    return patternIssues;
  }

  const fixedValueIssues = await validateFixedValue(
    meta.definition,
    root,
    path
  );
  if (fixedValueIssues.length > 0) {
    return fixedValueIssues;
  }

  switch (meta._type_) {
    case "resource": {
      let type = meta.type;
      if (!isObject(root))
        return [
          issueError(
            "structure",
            `Value must be an object when validating '${
              meta.type
            }'. Instead found type of '${typeof root}'`,
            [toJSONPointer(path)]
          ),
        ];

      if (meta.type === "Resource" || meta.type === "DomainResource") {
        type = get(descend(path, "resourceType"), root);
      }

      const typeIssues = checkResourceType(type, root, path);
      if (typeIssues.length > 0) return typeIssues;

      const resourceMeta = getStartingMeta(ctx.fhirVersion, type);
      if (!resourceMeta || resourceMeta._type_ !== "resource")
        throw new OperationError(
          outcomeFatal("not-found", `Could not find meta for type '${type}'`)
        );

      return validateComplex(ctx, resourceMeta, root, path);
    }
    case "complex-type": {
      if (!isObject(root))
        return [
          issueError(
            "structure",
            `Value must be an object when validating '${
              meta.type
            }'. Instead found type of '${typeof root}'`,
            [toJSONPointer(path)]
          ),
        ];

      if (meta.type === "Reference") {
        const issuesReferenceConstraint = await validateReferenceTypeConstraint(
          ctx,
          meta.definition,
          root,
          path
        );
        if (issuesReferenceConstraint.length > 0)
          return issuesReferenceConstraint;
      }

      return validateComplex(ctx, meta, root, path);
    }
    case "fp-primitive":
    case "primitive-type": {
      return validatePrimitive(
        ctx,
        meta.definition,
        root,
        path as Loc<any, any, any>,
        meta.type
      );
    }
  }
}

export default async function validate(
  ctx: ValidationCTX,
  type: uri,
  root: unknown,
  path_?: Loc<any, any, any>
): Promise<OperationOutcome["issue"]> {
  const path =
    path_ ?? pointer(ctx.fhirVersion, type as AllDataTypes, "id" as id);

  const meta = getStartingMeta(ctx.fhirVersion, type);
  if (!meta)
    throw new OperationError(
      outcomeFatal("not-found", `Could not find meta for type '${type}'`)
    );

  if (meta._type_ === "fp-primitive") throw new Error();

  return validateElementSingular(ctx, meta, root, path) ?? [];
}
