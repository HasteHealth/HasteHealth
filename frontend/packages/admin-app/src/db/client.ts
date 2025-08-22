import { atom } from "jotai";

import { AsynchronousClient } from "@oxidized-health/client";
import createHTTPClient, { HTTPContext } from "@oxidized-health/client/http";
import { createMiddlewareAsync } from "@oxidized-health/client/middleware";
import { AllInteractions, FHIRResponse } from "@oxidized-health/client/types";
import { FHIR_VERSION } from "@oxidized-health/fhir-types/versions";

type CachedClient = AsynchronousClient<HTTPContext>;

export const getClient = atom<ReturnType<typeof createAdminAppClient>>(
  // Q Hack to avoid uneccessary checks.
  undefined as unknown as ReturnType<typeof createAdminAppClient>
);

const cachedResponse: Record<
  string,
  Promise<FHIRResponse<FHIR_VERSION, AllInteractions | "error">>
> = {};

/*
 ** Cache select calls for performance improvements (notably expansions).
 */
export function createAdminAppClient(
  client: ReturnType<typeof createHTTPClient>
): CachedClient {
  return new AsynchronousClient(
    createMiddlewareAsync<
      { client: ReturnType<typeof createHTTPClient> },
      HTTPContext
    >({ client: client }, [
      async (state, context) => {
        switch (context.request.type) {
          case "invoke-request": {
            switch (context.request.operation) {
              case "expand": {
                const requestString = JSON.stringify(context.request);
                if (!cachedResponse[requestString]) {
                  cachedResponse[requestString] = state.client.request(
                    context.ctx,
                    context.request
                  );
                }

                return [
                  state,
                  {
                    ...context,
                    response: await cachedResponse[requestString],
                  },
                ];
              }

              default:
                return [
                  state,
                  {
                    ...context,
                    response: await state.client.request(
                      context.ctx,
                      context.request
                    ),
                  },
                ];
            }
          }

          default:
            return [
              state,
              {
                ...context,
                response: await state.client.request(
                  context.ctx,
                  context.request
                ),
              },
            ];
        }
      },
    ])
  );
}
