import { expect, test } from "@jest/globals";

import {
  Bundle,
  code,
} from "@oxidized-health/fhir-types/lib/generated/r4/types";
import { FHIR_VERSION, R4 } from "@oxidized-health/fhir-types/lib/versions";

import { AllInteractions, FHIRResponse } from "../types/index.js";
import type { ParsedParameter } from "../url";
import { createMiddlewareAsync } from "./index.js";

test("Test middleware Async", async () => {
  const middleware = createMiddlewareAsync<{}, {}>({}, [
    async (state, context, next) => {
      if (next) {
        const [nextState, res] = await next(state, context);
        return [
          nextState,
          {
            ...res,
            response: {
              ...res.response,
              body: {
                ...((res as any).response?.body as any),
                entry: (res.response as any).body?.entry?.map((e: any) => ({
                  ...e,
                  resource: {
                    id: "123",
                    ...e.resource,
                  },
                })),
              } as Bundle,
            } as FHIRResponse<FHIR_VERSION, AllInteractions>,
          },
        ];
      }
      return [
        state,
        {
          ...context,

          response: {
            fhirVersion: R4,
            parameters: (context.request as any).parameters
              ? ((context.request as any).parameters as ParsedParameter<
                  string | number
                >[])
              : [],
            type: "search-response",
            level: "system",
            body: {
              resourceType: "Bundle",
              type: "searchset" as code,
              entry: [],
            } as Bundle,
          },
        },
      ];
    },
    async (state, context, next) => {
      return [
        state,
        {
          ...context,
          response: {
            fhirVersion: R4,
            parameters: (context.request as any).parameters
              ? ((context.request as any).parameters as ParsedParameter<
                  string | number
                >[])
              : [],
            type: "search-response",
            level: "system",
            body: {
              resourceType: "Bundle",
              type: "searchset" as code,
              entry: [{ resource: { resourceType: "Patient" } }],
            } as Bundle,
          },
        },
      ];
    },
  ]);

  const result = middleware({
    key: "test",
    ctx: {},
    request: {} as any,
  });
  expect(result).toBeInstanceOf(Promise);
  expect(await result).toEqual({
    key: "test",
    request: {},
    ctx: {},
    response: {
      fhirVersion: R4,
      parameters: [],
      type: "search-response",
      level: "system",
      body: {
        resourceType: "Bundle",
        type: "searchset",
        entry: [{ resource: { id: "123", resourceType: "Patient" } }],
      },
    },
  });
});

test("Test state", async () => {
  const middleware = createMiddlewareAsync<number, {}, number, number>(5, [
    async (state, context, next) => {
      const [nextState, res] = await next(state, context);
      return [
        res.request + nextState,
        { ...res, response: state + context.request },
      ];
    },
  ]);

  expect((await middleware({ ctx: {}, request: 5 })).response).toEqual(10);
  expect((await middleware({ ctx: {}, request: 5 })).response).toEqual(15);
});
