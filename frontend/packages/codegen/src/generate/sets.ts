import { FHIR_VERSION, Resource } from "@oxidized-health/fhir-types/versions";

function generateTypeSet(
  name: string,
  sds: Readonly<Array<Resource<FHIR_VERSION, "StructureDefinition">>>
) {
  return `export const ${name}: Set<string>  = new Set([${sds
    .map((sd) => `"${sd.id}"`)
    .join(",")}])\n`;
}

export function generateSets<Version extends FHIR_VERSION>(
  version: Version,
  structureDefinitions: Readonly<
    Array<Resource<Version, "StructureDefinition">>
  >
): string {
  // Ignore templates for now during type generation.
  structureDefinitions = structureDefinitions
    .filter((sd) => sd.kind !== "logical")
    .filter((sd) => sd.derivation !== "constraint");

  const primitiveTypes = structureDefinitions
    .filter((sd) => sd.kind === "primitive-type")
    .filter((sd) => sd.derivation !== "constraint");

  const complexTypes = structureDefinitions
    .filter((sd) => sd.kind === "complex-type")
    .filter((sd) => sd.derivation !== "constraint");
  const resourceTypes = structureDefinitions.filter(
    (sd) => sd.kind === "resource"
  );

  return `${generateTypeSet("resourceTypes", resourceTypes)}\n${generateTypeSet(
    "complexTypes",
    complexTypes
  )}\n${generateTypeSet("primitiveTypes", primitiveTypes)}\n`;
}
