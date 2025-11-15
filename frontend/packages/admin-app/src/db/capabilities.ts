import { atom } from "jotai";

import { isResponseError } from "@haste-health/client/http";
import { Toaster } from "@haste-health/components";
import { R4 } from "@haste-health/fhir-types/versions";

import { getClient } from "./client";

export const getCapabilities = atom(async (get) => {
  try {
    const client = get(getClient);
    const capabilityStatement = await client.capabilities({}, R4);
    return capabilityStatement;
  } catch (e) {
    console.error(e);
    if (isResponseError(e)) {
      Toaster.error(
        e.response.body.issue?.[0]?.diagnostics ??
          "Failed to fetch server capabilities."
      );
    } else {
      Toaster.error("Failed to fetch server capabilities.");
    }
  }
});
