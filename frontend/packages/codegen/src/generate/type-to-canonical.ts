import { uri } from "@haste-health/fhir-types/lib/generated/r4/types";
import { FHIR_VERSION, Resource } from "@haste-health/fhir-types/versions";

import { filterSDForTypes } from "../utilities.js";

type CanonicalMap = Record<uri, uri>;

export function generateTypeToCanonicalMap<Version extends FHIR_VERSION>(
  sds: Resource<Version, "StructureDefinition">[]
): CanonicalMap {
  return filterSDForTypes(sds).reduce((acc: CanonicalMap, sd) => {
    acc[sd.type] = sd.url;
    return acc;
  }, {});
}
