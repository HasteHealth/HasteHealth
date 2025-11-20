import { expect, test } from "@jest/globals";

import {
  OperationDefinition,
  Parameters,
} from "@haste-health/fhir-types/lib/generated/r4/types";

import { Operation } from "./index";

test("TEST Parse to object", async () => {
  const op = new Operation({
    id: "ebabc714-f229-4d1b-bc7f-720209d3aae7",
    code: "worker-testing",
    kind: "operation",
    meta: {
      extension: [
        {
          url: "https://haste.health/version-sequence",
          valueInteger: 7,
        },
        {
          url: "https://haste.health/author",
          valueString: "public-user",
        },
      ],
      versionId: "7",
      lastUpdated: "2023-10-26T18:49:16.187+00:00",
    },
    name: "Sub testing",
    type: false,
    status: "active",
    system: true,
    instance: false,
    parameter: [
      {
        max: "*",
        min: 1,
        use: "in",
        name: "payload",
        type: "Any",
      },
    ],
    resourceType: "OperationDefinition",
  } as OperationDefinition);

  expect(
    op.parseToObject("in", {
      resourceType: "Parameters",
      parameter: [
        {
          name: "payload",
          resource: {
            id: "b9339845-d0db-433e-b003-cf4214395077",
            meta: {
              extension: [
                {
                  url: "https://haste.health/version-sequence",
                  valueInteger: 20,
                },
                {
                  url: "https://haste.health/author",
                  valueString: "public-user",
                },
              ],
              versionId: "20",
              lastUpdated: "2023-10-26T19:21:52.048+00:00",
            },
            status: "amended",
            resourceType: "QuestionnaireResponse",
          },
        },
      ],
    } as Parameters)
  ).toEqual({
    payload: [
      {
        id: "b9339845-d0db-433e-b003-cf4214395077",
        meta: {
          extension: [
            {
              url: "https://haste.health/version-sequence",
              valueInteger: 20,
            },
            {
              url: "https://haste.health/author",
              valueString: "public-user",
            },
          ],
          versionId: "20",
          lastUpdated: "2023-10-26T19:21:52.048+00:00",
        },
        status: "amended",
        resourceType: "QuestionnaireResponse",
      },
    ],
  });
});
