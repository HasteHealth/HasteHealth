import { FHIR_VERSION, Resource } from "@oxidized-health/fhir-types/versions";

export function filterSDForTypes<Version extends FHIR_VERSION>(
  sds: Resource<Version, "StructureDefinition">[]
): Resource<Version, "StructureDefinition">[] {
  return (
    sds
      // Filter out constraints
      .filter((sd) => sd.derivation !== "constraint")
      // Logicals are not concrete types
      .filter((sd) => sd.kind !== "logical")
  );
}
