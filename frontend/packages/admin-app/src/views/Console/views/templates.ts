/**
 * What a new resource starts from, for types whose custom view needs more
 * than the required elements.
 *
 * Without a template the create form starts from the type's StructureDefinition,
 * which is right for most types but leaves these two with nothing to show.
 */
import {
  OperationDefinition,
  Resource,
} from "@haste-health/fhir-types/r4/types";

import { withCode } from "./OperationDefinition";

const DEFAULT_OPERATION_CODE = `interface Context {
  request: {
    id?: string;
    resource?: string;
    parameters: unknown;
  };
}

export default async function (context: Context) {
  // \`parameters\` holds the inputs declared on this OperationDefinition.
  const { input } = context.request.parameters as { input?: string };

  // Each output is a named parameter, so the resource is returned inside a
  // Parameters under the name this operation declares.
  return {
    resourceType: "Parameters",
    parameter: [
      {
        name: "basic",
        resource: {
          resourceType: "Basic",
          code: { text: \`echo: \${input ?? ""}\` },
        },
      },
    ],
  };
}
`;

/** A new operation: one declared input, one output, and runnable source. */
function newOperation(): Resource {
  return withCode(
    {
      resourceType: "OperationDefinition",
      name: "NewOperation",
      status: "draft",
      kind: "operation",
      code: "new-operation",
      system: true,
      type: false,
      instance: false,
      parameter: [
        // Only declared inputs are sent; an undeclared name is dropped.
        {
          name: "input",
          use: "in",
          min: 0,
          max: "1",
          type: "string",
          documentation: "Replace with the inputs this operation takes.",
        },
        // Each output is a named parameter in the returned Parameters. The
        // exception is a lone output named `return` typed as a resource, which
        // FHIR treats as the response body itself.
        {
          name: "basic",
          use: "out",
          min: 1,
          max: "1",
          type: "Basic",
          documentation:
            "The resource this operation returns. Rename it and narrow the type to what it produces.",
        },
      ],
    } as unknown as OperationDefinition,
    DEFAULT_OPERATION_CODE,
  ) as Resource;
}

/** A new view: a Patient projection with two columns. */
function newViewDefinition(): Resource {
  return {
    resourceType: "ViewDefinition",
    status: "draft",
    resource: "Patient",
    select: [
      {
        column: [
          {
            name: "id",
            path: "id",
            type: "http://hl7.org/fhirpath/System.String",
          },
          { name: "birth_date", path: "$this.birthDate", type: "date" },
        ],
      },
    ],
  } as unknown as Resource;
}

/** Templates by resource type. Add an entry to give a type a starting point. */
const TEMPLATES: Record<string, () => Resource> = {
  OperationDefinition: newOperation,
  ViewDefinition: newViewDefinition,
};

/** What a new `resourceType` starts from, or undefined for the default. */
export function typeTemplate(resourceType: string): Resource | undefined {
  return TEMPLATES[resourceType]?.();
}
