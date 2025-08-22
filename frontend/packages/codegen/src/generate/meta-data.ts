import {
  ElementDefinition,
  ElementDefinitionType,
  uri,
} from "@oxidized-health/fhir-types/r4/types";
import {
  Data,
  FHIR_VERSION,
  Resource,
} from "@oxidized-health/fhir-types/versions";

import { traversalBottomUp } from "../sdTraversal.js";
import { filterSDForTypes } from "../utilities.js";

interface Node {
  _type_:
    | "fp-primitive"
    | "primitive-type"
    | "complex-type"
    | "resource"
    | "type"
    | "typechoice"
    | "content-reference";
  cardinality: "array" | "single";
  base: uri;
}

export interface TypeChoiceNode extends Node {
  _type_: "typechoice";
  definition: Data<FHIR_VERSION, "ElementDefinition">;
  fieldsToType: Record<string, uri>;
}

export interface ElementNode extends Node {
  _type_: "primitive-type" | "complex-type" | "resource";
  type: uri;
  definition: Data<FHIR_VERSION, "ElementDefinition">;
  properties: Record<string, number>;
}

export interface FPPrimitiveNode extends Node {
  _type_: "fp-primitive";
  type: uri;
}

export interface TypeNode extends Node {
  _type_: "type";
  type: uri;
  definition: Data<FHIR_VERSION, "ElementDefinition">;
}

export interface ContentReferenceNode extends Node {
  _type_: "content-reference";
  definition: Data<FHIR_VERSION, "ElementDefinition">;
  reference: number;
}

export type MetaNode =
  | ElementNode
  | TypeNode
  | ContentReferenceNode
  | TypeChoiceNode
  | FPPrimitiveNode;

export interface MetaV2Compiled {
  [key: string]: Array<MetaNode>;
}

function createFPPrimitiveNode(type: uri): MetaNode[] {
  return [
    {
      _type_: "fp-primitive",
      type,
      cardinality: "single",
      base: type,
    },
  ];
}

const FP_PRIMITIVES_TYPES: uri[] = [
  "http://hl7.org/fhirpath/System.Boolean",
  "http://hl7.org/fhirpath/System.String",
  "http://hl7.org/fhirpath/System.Date",
  "http://hl7.org/fhirpath/System.DateTime",
  "http://hl7.org/fhirpath/System.Decimal",
  "http://hl7.org/fhirpath/System.Integer",
  "http://hl7.org/fhirpath/System.Time",
] as uri[];

/**
 *
 * @param type
 * @returns
 */
function addFPPrimitiveNodes(metadata: MetaV2Compiled) {
  return FP_PRIMITIVES_TYPES.reduce((metadata, type) => {
    metadata[type] = createFPPrimitiveNode(type);
    return metadata;
  }, metadata);
}

function determineIsTypeChoice(element: ElementDefinition): boolean {
  return (element.type ?? []).length > 1;
}

type ElementPath = string & { _elementPathBrand: unknown };
type ElementPathType = string & { _elementPathTypeBrand: unknown };

function combineWithPath(path: ElementPath, type: uri): ElementPathType {
  return `${path}:${type}` as ElementPathType;
}

// Generate indices around the element path and it's type which correlate to location in the meta.
function preGenerateIndices(
  sd: Resource<FHIR_VERSION, "StructureDefinition">
): Record<ElementPath | ElementPathType, number> {
  let curIndex = 0;
  // Record
  const indices: Record<ElementPath | ElementPathType, number> = {};

  for (let i = 0; i < (sd.snapshot?.element ?? []).length; i++) {
    const element = sd.snapshot?.element[i];
    if (!element) throw new Error("No Element");
    indices[element.path as ElementPath] = curIndex++;
  }

  return indices;
}

// function to capaitalize a string
function capitalize(str: string) {
  return str.charAt(0).toUpperCase() + str.slice(1);
}

function getElementField(element: ElementDefinition, type?: string) {
  const path = element.path;
  const pathSplit = path.split(".");
  const field = pathSplit[pathSplit.length - 1];
  if (type) {
    return field.replace("[x]", capitalize(type));
  }
  return (element.type ?? []).length > 1 ? field.replace("[x]", "") : field;
}

function getAllTypeChoice(
  beginning: string,
  element: ElementDefinition
): Partial<ElementDefinition> {
  return Object.keys(element)
    .filter((k): k is keyof ElementDefinition => k.startsWith(beginning))
    .reduce((acc: Record<string, unknown>, k) => {
      acc[k] = element[k];
      return acc;
    }, {});
}

function simpleElementDefinition(
  element: Data<FHIR_VERSION, "ElementDefinition">
): Data<FHIR_VERSION, "ElementDefinition"> {
  return {
    path: element.path,
    min: element.min,
    max: element.max,
    ...getAllTypeChoice("pattern", element),
    ...getAllTypeChoice("minValue", element),
    ...getAllTypeChoice("maxValue", element),
    ...getAllTypeChoice("fixed", element),
    maxLength: element.maxLength,
    type: element.type,
    base: element.base
      ? {
          path: element.base.path,
          max: element.base.max,
          min: element.base.min,
        }
      : undefined,
    binding: element.binding
      ? {
          strength: element.binding.strength,
          valueSet: element.binding.valueSet,
        }
      : undefined,
  };
}

function getElementCardinality(element: ElementDefinition): "array" | "single" {
  const cardinality = element.max === "1" ? "single" : "array";
  return cardinality;
}

function createContentReferenceNode<Version extends FHIR_VERSION>(
  sd: Resource<Version, "StructureDefinition">,
  element: Data<Version, "ElementDefinition">,
  reference: number
): ContentReferenceNode {
  return {
    _type_: "content-reference",
    definition: simpleElementDefinition(element),
    base: sd.type,
    cardinality: getElementCardinality(element),
    reference,
  };
}

function createSingularNode<Version extends FHIR_VERSION>(
  sd: Resource<Version, "StructureDefinition">,
  index: number,
  element: Data<Version, "ElementDefinition">,
  children: { index: number; field: string }[]
): MetaNode {
  if (index === 0) {
    if (!["primitive-type", "complex-type", "resource"].includes(sd.kind)) {
      throw new Error(
        "sd must be of type 'primitive-type', 'complex-type' or 'resource'"
      );
    }

    return {
      _type_: sd.kind as "primitive-type" | "complex-type" | "resource",
      base: sd.type,
      type: sd.type,
      // Set as singular to ensure that the first element is always singular.
      // Also removing base to ensure singular.
      definition: simpleElementDefinition(element),
      cardinality: "single",
      properties: children.reduce((acc: Record<string, number>, k) => {
        acc[k.field] = k.index;
        return acc;
      }, {}),
    };
  }

  const type: ElementDefinitionType | undefined = element.type?.[0];
  if ((element.type ?? []).length > 1) {
    throw new Error(
      "Multiple types are not supported for creating singular node."
    );
  }
  const cardinality = getElementCardinality(element);

  if (children.length === 0) {
    return {
      _type_: "type",
      base: sd.type,
      type: type?.code as uri,
      definition: simpleElementDefinition(element),
      cardinality,
    };
  }

  return {
    _type_: "complex-type",
    base: sd.type,
    type: type?.code as uri,
    definition: simpleElementDefinition(element),
    properties: children.reduce((acc: Record<string, number>, k) => {
      acc[k.field] = k.index;
      return acc;
    }, {}),
    cardinality,
  };
}

function createTypeChoiceNode<Version extends FHIR_VERSION>(
  sd: Resource<Version, "StructureDefinition">,
  element: Data<Version, "ElementDefinition">
): TypeChoiceNode {
  return {
    _type_: "typechoice",
    base: sd.type,
    definition: simpleElementDefinition(element),
    cardinality: getElementCardinality(element),
    fieldsToType: (element.type ?? []).reduce(
      (acc: Record<string, uri>, type) => {
        acc[getElementField(element, type.code)] = type.code;
        return acc;
      },
      {}
    ),
  };
}

function SDToMetaData<Version extends FHIR_VERSION>(
  sd: Resource<Version, "StructureDefinition">
) {
  // Used to handle content references so know ahead of time where to point.
  const indices = preGenerateIndices(sd);
  const metaInfo: Array<MetaNode> = [
    ...new Array(Math.max(...Object.values(indices))),
  ];

  traversalBottomUp<{ index: number; field: string }>(
    sd,
    (element: ElementDefinition, children) => {
      const index = indices[element.path as ElementPath];
      if (element.contentReference) {
        const elementIndex =
          indices[
            element.contentReference.substring(1) as unknown as ElementPath
          ];
        metaInfo[index] = createContentReferenceNode(
          sd,
          element as Data<Version, "ElementDefinition">,
          elementIndex
        );

        return [
          {
            index,
            field: getElementField(element),
          },
        ];
      }

      if (determineIsTypeChoice(element)) {
        metaInfo[index] = createTypeChoiceNode(
          sd,
          element as Data<Version, "ElementDefinition">
        );

        return [
          ...(element.type ?? []).map((type) => {
            const indexKey = combineWithPath(
              element.path as ElementPath,
              type.code
            );

            const index = indices[indexKey];
            return {
              index,
              field: getElementField(element, type.code),
            };
          }),
          {
            index,
            field: getElementField(element),
          },
        ];
        // For Type choices include a typechoice node which points to the types loc.
      } else {
        metaInfo[index] = createSingularNode(
          sd,
          index,
          element as Data<Version, "ElementDefinition">,
          children
        );

        return [
          {
            index,
            field: getElementField(element),
          },
        ];
      }
    }
  );

  return metaInfo;
}

export function generateMetaData<Version extends FHIR_VERSION>(
  sds: Resource<Version, "StructureDefinition">[]
): MetaV2Compiled {
  const metav2compiled = filterSDForTypes(sds).reduce(
    (acc: MetaV2Compiled, sd) => {
      acc[sd.type] = SDToMetaData(sd);
      return acc;
    },
    {} as MetaV2Compiled
  );

  return addFPPrimitiveNodes(metav2compiled);
}
