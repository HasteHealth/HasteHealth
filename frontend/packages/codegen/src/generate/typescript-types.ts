import {
  ElementDefinition,
  unsignedInt,
} from "@haste-health/fhir-types/r4/types";
import { FHIR_VERSION, Resource } from "@haste-health/fhir-types/versions";

import { traversalBottomUp } from "../sdTraversal.js";

export const primitiveTypes: Set<string> = new Set([
  "base64Binary",
  "boolean",
  "canonical",
  "code",
  "date",
  "dateTime",
  "decimal",
  "id",
  "instant",
  "integer",
  "markdown",
  "oid",
  "positiveInt",
  "string",
  "time",
  "unsignedInt",
  "uri",
  "url",
  "uuid",
  "xhtml",
]);

function fhirSystemTypePredicate(type: string) {
  switch (type) {
    case "http://hl7.org/fhirpath/System.Boolean": {
      return "boolean";
    }
    case "http://hl7.org/fhirpath/System.String": {
      return "string";
    }
    case "http://hl7.org/fhirpath/System.Date": {
      return "string";
    }
    case "http://hl7.org/fhirpath/System.DateTime": {
      return "string";
    }
    case "http://hl7.org/fhirpath/System.Decimal": {
      return "number";
    }
    case "http://hl7.org/fhirpath/System.Integer": {
      return "number";
    }
    case "http://hl7.org/fhirpath/System.Time": {
      return "string";
    }
  }
}

// function to capaitalize a string
function capitalize(str: string) {
  return str.charAt(0).toUpperCase() + str.slice(1);
}

function isRoot(
  sd: Resource<FHIR_VERSION, "StructureDefinition">,
  elementDefinition: ElementDefinition
) {
  return elementDefinition.path === sd.id;
}

function documentation(element: ElementDefinition) {
  if (element.short) return `  /** \n   * ${element.short}\n   */\n`;
  return "";
}

function getElementField(element: ElementDefinition, type?: string) {
  const path = element.path;
  const isRequired = element.min && (element.min as unknown as number) > 0;
  const pathSplit = path.split(".");
  let field = pathSplit[pathSplit.length - 1];
  if (type) {
    field = field.replace("[x]", capitalize(type));
  }
  // Type choice is always optional
  if (!isRequired || type) {
    field = field + "?";
  }
  return `${field}`;
}

function primitiveToTypescriptType(
  primitiveSd: Resource<FHIR_VERSION, "StructureDefinition">
): string | void {
  const primitiveValueType = primitiveSd.snapshot?.element.filter((element) =>
    element.path.endsWith(".value")
  )[0]?.type?.[0]?.code;
  // http://hl7.org/fhir/StructureDefinition/uri
  // Skip over these primitive types as already exist in typescript
  if (primitiveValueType) {
    const primitiveType = `export type ${
      primitiveSd.id
    } = ${fhirSystemTypePredicate(primitiveValueType)}`;

    // Avoid compiler issues by ts-ignoring boolean and string
    // which don't allow type aliasing
    if (primitiveSd.id === "boolean" || primitiveSd.id === "string") {
      return `// @ts-ignore\n${primitiveType};`;
    } else {
      let extension = "";
      if (
        primitiveSd.baseDefinition &&
        primitiveSd.baseDefinition !==
          "http://hl7.org/fhir/StructureDefinition/Element"
      ) {
        extension = ` & ${primitiveSd.baseDefinition.replace(
          "http://hl7.org/fhir/StructureDefinition/",
          ""
        )}`;
      }
      // For primitives that aren't string or boolean, add branding to have more type constraints.
      return `${primitiveType} & { _${primitiveSd.id}: "fhir_${primitiveSd.id}"; } ${extension};`;
    }
  } else {
    throw new Error("No type found for primitive");
  }
}

function isCollection(elementDefinition: ElementDefinition) {
  switch (elementDefinition.max) {
    case "*":
      return true;
    default:
      if (!elementDefinition.max) return false;
      return parseInt(elementDefinition.max) > 1;
  }
}

function wrapAsCollection(
  elementDefinition: ElementDefinition,
  typescriptString: string
) {
  if (isCollection(elementDefinition)) {
    return `Array<${typescriptString}>`;
  } else {
    return typescriptString;
  }
}

function _typeToTypescriptType(type: string): string {
  // .id fields are often typed as http://hl7.org/fhirpath/System.String
  // setting them to fhir primitive id instead
  if (
    fhirSystemTypePredicate(type) &&
    type !== "http://hl7.org/fhirpath/System.String"
  ) {
    throw new Error("System type not supported '${type}'");
  } else if (type === "http://hl7.org/fhirpath/System.String") {
    return "id";
  }
  return type;
}

function typeToTypescriptType(
  elementDefinition: ElementDefinition,
  type: string
) {
  const typescriptType = _typeToTypescriptType(type);
  return wrapAsCollection(elementDefinition, typescriptType);
}

function isNested(element: ElementDefinition) {
  return (
    element.type?.[0]?.code === "BackboneElement" ||
    element.type?.[0]?.code === "Element"
  );
}
function getInterfaceName(element: ElementDefinition) {
  if (!element.id)
    throw new Error("Cannot create interface name for '" + element.path + "'");
  return element.id.split(".").map(capitalize).join("");
}

/*
  Resolves an elements content Reference
*/
function contentReference(
  sd: Resource<FHIR_VERSION, "StructureDefinition">,
  element: ElementDefinition
) {
  const contentReference = element.contentReference?.split("#")[1];
  const referenceElement = sd.snapshot?.element.filter(
    (element) => element.id === contentReference
  )[0];
  if (!referenceElement)
    throw new Error(
      "unable to resolve contentreference: '" + element.contentReference + "'"
    );
  let referenceTypescriptType;
  if (isNested(referenceElement)) {
    referenceTypescriptType = getInterfaceName(referenceElement);
  } else {
    const type = referenceElement.type?.[0]?.code;
    if (type === undefined) {
      throw new Error(
        "No type found for content reference: '" + referenceElement.id + "'"
      );
    }
    referenceTypescriptType = typeToTypescriptType(referenceElement, type);
  }

  // Handle cardinality on reference type.
  if (element.max !== "1") {
    referenceTypescriptType = `Array<${referenceTypescriptType}>`;
  }

  return [
    `${documentation(element)}  ${getElementField(
      element
    )}: ${referenceTypescriptType};`,
  ];
}

function getPrimitiveExtension(element: ElementDefinition, type: string) {
  return `${documentation(element)}  _${getElementField(
    { ...element, min: 0 as unsignedInt },
    type
  )}: ${typeToTypescriptType(element, "Element")}`;
}

function processLeaf(
  sd: Resource<FHIR_VERSION, "StructureDefinition">,
  element: ElementDefinition
) {
  if (element.contentReference) {
    return contentReference(sd, element);
  } else if (element.type?.length && element.type?.length > 1) {
    return element.type
      ?.map((type) => {
        const fields = [
          `${documentation(element)}  ${getElementField(
            element,
            type.code
          )}: ${typeToTypescriptType(element, type.code)};`,
        ];
        if (primitiveTypes.has(type.code)) {
          fields.push(getPrimitiveExtension(element, type.code));
        }
        return fields;
      })
      .flat();
  }
  const fields = [
    `${documentation(element)}  ${getElementField(
      element
    )}: ${typeToTypescriptType(element, element.type?.[0]?.code as string)};`,
  ];
  if (primitiveTypes.has(element.type?.[0]?.code as string)) {
    fields.push(
      getPrimitiveExtension(element, element.type?.[0]?.code as string)
    );
  }
  return fields;
}

interface ComplexTypeOutput {
  typescriptInterface: string;
  field: string[];
}

function processComplexToTypescript(
  sd: Resource<FHIR_VERSION, "StructureDefinition">,
  element: ElementDefinition,
  children: string[]
): ComplexTypeOutput {
  const interfaceName = getInterfaceName(element);
  // for resources include the resourceType filed
  if (isRoot(sd, element) && sd.kind === "resource") {
    children.unshift(`resourceType: "${sd.id}"`);
  }

  return {
    typescriptInterface: `export interface ${interfaceName} {
${children.join("\n")}
}`,
    field: [
      `${documentation(element)}  ${getElementField(
        element
      )}: ${wrapAsCollection(element, interfaceName)};`,
    ],
  };
}

function resourceOrComplexFhirToTypescript(
  sd: Resource<FHIR_VERSION, "StructureDefinition">
): string | undefined {
  let typescriptTypes = "";
  traversalBottomUp(sd, (element, children: string[]): string[] => {
    if (children.length === 0) {
      return processLeaf(sd, element);
    } else {
      const { typescriptInterface, field } = processComplexToTypescript(
        sd,
        element,
        children
      );
      typescriptTypes = `${typescriptTypes}\n${typescriptInterface}`;
      return field;
    }
  });
  return typescriptTypes;
}

function getNonAbstractResourceTypes(
  sds: Resource<FHIR_VERSION, "StructureDefinition">[]
) {
  return sds.filter((sd) => !sd.abstract);
}

// Handle DomainResource and Resource by union joining existing generated types.
function generateAbstracts({
  resourceSDs,
  complexSDs,
}: {
  resourceSDs: Resource<FHIR_VERSION, "StructureDefinition">[];
  complexSDs: Resource<FHIR_VERSION, "StructureDefinition">[];
}) {
  // Resources
  const abstractResourceTypes = resourceSDs.filter((sd) => sd.abstract);
  const nonAbstractResourceTypes = resourceSDs.filter((sd) => !sd.abstract);
  const ResourceMap = `export type ResourceMap = {\n${nonAbstractResourceTypes
    .map((resource) => {
      return `  ${resource.id}: ${resource.id};`;
    })
    .join("\n")}\n}\n`;

  const ResourceType = `export type ResourceType = keyof ResourceMap`;
  const AResource = `export type AResource<T extends keyof ResourceMap> = ResourceMap[T];`;
  const ConcreteType = `export type ConcreteType = ResourceMap[keyof ResourceMap]`;
  const abstractResourceTypesTypescript = abstractResourceTypes
    .map(
      (abstractResource) => `export type ${abstractResource.id} = ConcreteType`
    )
    .join("\n");

  // Complex
  const ComplexMap = `type ComplexMap = {\n${complexSDs
    .map((complex) => {
      return `  ${complex.id}: ${complex.id};`;
    })
    .join("\n")}\n}\n`;

  const ComplexTypes = `type ComplexTypes = keyof ComplexMap`;
  const DataMap = `type DataMap = ComplexMap & ResourceMap`;
  const DataType = `export type DataType = keyof DataMap`;
  const AData = `export type AData<T extends DataType> = DataMap[T];`;

  return `${ResourceMap}\n
${ResourceType}\n
${AResource}\n
${ConcreteType}\n
${abstractResourceTypesTypescript}\n
${ComplexMap}\n
${ComplexTypes}\n
${DataMap}\n
${DataType}\n
${AData}`;
}

export function generateTypes<Version extends FHIR_VERSION>(
  fhirVersion: Version,
  structureDefinitions: Readonly<
    Array<Resource<Version, "StructureDefinition">>
  >
): string {
  const primitiveSDs = structureDefinitions.filter(
    (sd) => sd.kind === "primitive-type" && sd.derivation !== "constraint"
  );

  const complexSDs = structureDefinitions
    .filter((sd) => sd.kind === "complex-type")
    .filter((sd) => sd.derivation !== "constraint");

  const resourceSDs = structureDefinitions
    .filter((sd) => sd.kind === "resource")
    .filter((sd) => sd.derivation !== "constraint");

  const typescriptTypes: string = primitiveSDs
    .map(primitiveToTypescriptType)
    .filter((type) => type)
    // Generate Complex types
    .concat(complexSDs.map(resourceOrComplexFhirToTypescript))
    // Generate Resources
    .concat(
      getNonAbstractResourceTypes(resourceSDs)
        .map(resourceOrComplexFhirToTypescript)
        .filter((type) => type)
    )
    // Generate Maps
    .concat(generateAbstracts({ resourceSDs, complexSDs }))
    .join("\n");
  return typescriptTypes;
}
