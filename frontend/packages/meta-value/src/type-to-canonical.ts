import {
  canonical,
  uri,
} from "@oxidized-health/fhir-types/lib/generated/r4/types";
import { FHIR_VERSION, R4, R4B } from "@oxidized-health/fhir-types/versions";

import r4TypeToCanonical from "./v2/generated/type-canonical/r4.js";
import r4bTypeToCanonical from "./v2/generated/type-canonical/r4b.js";

export function resolveTypeToCanonical(
  version: FHIR_VERSION,
  type: uri
): canonical | undefined {
  switch (version) {
    case R4:
      return r4TypeToCanonical[type] as canonical | undefined;
    case R4B:
      return r4bTypeToCanonical[type] as canonical | undefined;
    default:
      throw new Error(`Unsupported FHIR version: ${version}`);
  }
}
