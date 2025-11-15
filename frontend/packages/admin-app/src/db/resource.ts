import { atom } from "jotai";
import { atomFamily } from "jotai/utils";

import { R4, ResourceType } from "@haste-health/fhir-types/versions";
import { id } from "@haste-health/fhir-types/lib/generated/r4/types";
import { isResponseError } from "@haste-health/client/lib/http";
import { Toaster } from "@haste-health/components";

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

type readResourceParam = {
  resourceType: ResourceType<R4>;
  id: id;
};

export const getResource = atomFamily(
  (param: readResourceParam) =>
    atom(async (get) => {
      try {
        const client = get(getClient);
        const resource = await client.read(
          {},
          R4,
          param.resourceType,
          param.id
        );
        return resource;
      } catch (e) {
        console.error(e);
        if (isResponseError(e)) {
          Toaster.error(
            e.response.body.issue?.[0]?.diagnostics ??
              "Failed to fetch resource."
          );
        } else {
          Toaster.error("Failed to fetch resource.");
        }
      }
    }),
  (a, b) => a.resourceType === b.resourceType && a.id === b.id
);
