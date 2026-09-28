/**
 * Starting bodies for creating a resource.
 *
 * An empty `{"resourceType": "X"}` is a poor place to start: it tells you
 * nothing about what the server will reject. The StructureDefinition already
 * knows which elements are required and what type each one is, so the
 * template is generated from it rather than maintained as a list per resource
 * type.
 *
 * A handful of types are worth more than their required elements, because
 * the shape that is actually useful is not the minimum one - a ViewDefinition
 * with no columns is valid and useless. Those keep the worked examples the
 * resource specific editors used to open with, in {@link TEMPLATES}.
 */
import {
  ElementDefinition,
  Resource,
  StructureDefinition,
} from "@haste-health/fhir-types/r4/types";

/** The example code a new OperationDefinition opens with. */
const OPERATION_CODE = `
interface Context {
  request: {
    id?: string;
    resource?: string;
    parameters: unknown;
  }
}

export default async function(context: Context) {
  const sd = await fhir.read("StructureDefinition", context.request.parameters.parameter.filter(p => p.name === "id")[0].valueString);

  return {
      resourceType: 'Parameters',
      parameter: [
          {
              name: 'sd',
              resource: sd
          }
      ]
  };
}
`;

/**
 * Worked examples for the types where the minimum resource teaches nothing.
 * These are the bodies the resource specific editors used to seed.
 */
const TEMPLATES: Record<string, Record<string, unknown>> = {
  ViewDefinition: {
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
          { name: "date of birth", path: "$this.birthDate", type: "date" },
        ],
      },
      {
        forEach: "$this.name",
        column: [
          {
            name: "name",
            path: "$this.given",
            type: "string",
            collection: true,
          },
          { name: "family", path: "$this.family", type: "string" },
        ],
      },
    ],
  },

  OperationDefinition: {
    resourceType: "OperationDefinition",
    extension: [
      {
        extension: [
          {
            url: "https://haste.health/Extension/custom-code-type",
            valueString: "text/typescript",
          },
        ],
        url: "https://haste.health/Extension/custom-code",
        valueString: OPERATION_CODE,
      },
    ],
    name: "New Operation",
    status: "draft",
    kind: "operation",
    code: "new",
    system: true,
    type: false,
    instance: false,
  },

  Subscription: {
    resourceType: "Subscription",
    meta: { profile: ["https://haste.health/resource/Subscription"] },
    status: "active",
    criteria: "",
    reason: "",
    channel: { type: "rest-hook", endpoint: "" },
  },
};

/**
 * The first code in a `short` like `registered | preliminary | final +`.
 *
 * Required `code` elements carry their value set inline in the short
 * description, which is enough to fill in a plausible default without a
 * terminology round trip.
 */
function firstCode(short: string | undefined): string | undefined {
  if (!short?.includes("|")) return undefined;
  const first = short.split("|")[0].trim();
  // A short that is prose rather than a code list will not look like a code.
  return /^[A-Za-z][A-Za-z0-9-]*$/.test(first) ? first : undefined;
}

/** A placeholder value for one element, by its FHIR type. */
function sampleFor(type: string, element: ElementDefinition): unknown {
  switch (type) {
    case "code":
      return firstCode(element.short as string | undefined) ?? "";
    case "string":
    case "markdown":
    case "id":
    case "uri":
    case "url":
    case "canonical":
    case "oid":
    case "uuid":
      return "";
    case "boolean":
      return false;
    case "integer":
    case "positiveInt":
    case "unsignedInt":
    case "decimal":
      return 0;
    case "date":
      return "2026-01-01";
    case "dateTime":
    case "instant":
      return "2026-01-01T00:00:00Z";
    case "time":
      return "00:00:00";
    case "Reference":
      return { reference: "" };
    case "CodeableConcept":
      return { coding: [{ system: "", code: "" }] };
    case "Coding":
      return { system: "", code: "" };
    case "Identifier":
      return { system: "", value: "" };
    case "Quantity":
      return { value: 0, unit: "" };
    case "Period":
      return { start: "" };
    case "HumanName":
      return { family: "", given: [""] };
    case "ContactPoint":
      return { system: "phone", value: "" };
    case "Address":
      return { line: [""], city: "", postalCode: "" };
    default:
      // A backbone element or a type we have no sample for; an empty object
      // still shows the field is expected.
      return {};
  }
}

/**
 * Builds a starting resource for `resourceType` from its definition.
 *
 * Only required top level elements are filled in. Going deeper would produce
 * a wall of scaffolding that is mostly wrong, and FHIR marks few elements
 * required, so what comes out is short and close to the minimum the server
 * will accept.
 */
export function templateFor(
  resourceType: string,
  structure: StructureDefinition | undefined,
): Resource {
  // A hand written example, where one says more than the minimum resource.
  // Copied, so that editing one create cannot alter the next one's starting
  // point through the shared object.
  const example = TEMPLATES[resourceType];
  if (example) {
    return structuredClone(example) as unknown as Resource;
  }

  const template: Record<string, unknown> = { resourceType };
  const elements =
    structure?.snapshot?.element ?? structure?.differential?.element ?? [];

  for (const element of elements) {
    const path = element.path as string;
    // Top level only: `Observation.status`, not `Observation.code.coding`.
    if (!path.startsWith(`${resourceType}.`)) continue;
    const name = path.slice(resourceType.length + 1);
    if (name.includes(".")) continue;
    if ((element.min ?? 0) < 1) continue;

    const types = (element.type ?? []).map((t) => t.code as string);
    const type = types[0];
    if (!type) continue;

    // A choice element is written with the type in its name.
    const key = name.endsWith("[x]")
      ? `${name.slice(0, -3)}${type[0].toUpperCase()}${type.slice(1)}`
      : name;

    const value = sampleFor(type, element);
    const repeats = element.max === "*" || Number(element.max) > 1;
    template[key] = repeats ? [value] : value;
  }

  // The template is deliberately partial - it is a starting point to edit,
  // not a resource that would validate yet.
  return template as unknown as Resource;
}
