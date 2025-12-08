import { atom } from "jotai";

import { R4 } from "@haste-health/fhir-types/versions";
import { TenantEndpointInformation } from "@haste-health/generated-ops/r4";

import { getClient } from "./client";

export const getEndpointMetadata = atom(async (get) => {
  const client = get(getClient);
  const endpointMetadata = client.invoke_system(
    TenantEndpointInformation.Op,
    {},
    R4,
    {}
  );
  return endpointMetadata;
});
