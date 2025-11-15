import deepEqual from "fast-deep-equal";
import { atom } from "jotai";
import { atomFamily } from "jotai/utils";

import { uri } from "@haste-health/fhir-types/r4/types";
import { R4 } from "@haste-health/fhir-types/versions";
import { ValueSetExpand } from "@haste-health/generated-ops/r4";

import { getClient } from "./client";

// https://jotai.org/docs/utilities/family
export const getValueSetExpansion = atomFamily(
  (url: string) =>
    atom(async (get) => {
      const client = get(getClient);
      const expansion = client.invoke_type(
        ValueSetExpand.Op,
        {},
        R4,
        "ValueSet",
        {
          url: url as uri,
        }
      );
      return expansion;
    }),
  deepEqual
);
