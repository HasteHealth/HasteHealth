import { FHIR_VERSION } from "@haste-health/fhir-types/versions";

import { AllInteractions, FHIRRequest, FHIRResponse } from "../types/index.js";

type Context<CTX, Request, Response> = {
  key?: string;
  ctx: CTX;
  request: Request;
  response?: Response;
};

export type MiddlewareAsyncChain<
  State,
  CTX,
  Request = FHIRRequest<FHIR_VERSION, AllInteractions>,
  Response = FHIRResponse<FHIR_VERSION, AllInteractions | "error">
> = (
  state: State,
  ctx: Context<CTX, Request, Response>,
  next: Next<State, CTX, Request, Response>
) => Promise<[State, Context<CTX, Request, Response>]>;

type Next<
  State,
  CTX,
  Request = FHIRRequest<FHIR_VERSION, AllInteractions>,
  Response = FHIRResponse<FHIR_VERSION, AllInteractions | "error">
> = (
  state: State,
  context: Context<CTX, Request, Response>
) => Promise<[State, Context<CTX, Request, Response>]>;

export type MiddlewareAsync<
  CTX,
  Request = FHIRRequest<FHIR_VERSION, AllInteractions>,
  Response = FHIRResponse<FHIR_VERSION, AllInteractions | "error">
> = (
  ctx: Context<CTX, Request, Response>
) => Promise<Context<CTX, Request, Response>>;

function createNext<
  State,
  CTX,
  Request = FHIRRequest<FHIR_VERSION, AllInteractions>,
  Response = FHIRResponse<FHIR_VERSION, AllInteractions | "error">
>(
  middlewareChain: MiddlewareAsyncChain<State, CTX, Request, Response>[],
  options: { logging?: boolean } = { logging: false }
): Next<State, CTX, Request, Response> {
  const [first, ...rest] = middlewareChain;

  if (first) {
    return async (state, context) => {
      if (options.logging) console.time(`${context.key}:${first.name}`);
      const response = await first(state, context, createNext(rest));
      if (options.logging) console.timeEnd(`${context.key}:${first.name}`);
      return response;
    };
  } else {
    // placeholder.
    return async (state, context) => {
      return [state, context];
    };
  }
}

export function createMiddlewareAsync<
  State,
  CTX,
  Request = FHIRRequest<FHIR_VERSION, AllInteractions>,
  Response = FHIRResponse<FHIR_VERSION, AllInteractions | "error">
>(
  _state: State,
  middlewareChain: MiddlewareAsyncChain<State, CTX, Request, Response>[],
  options: { logging?: boolean } = { logging: false }
): MiddlewareAsync<CTX, Request, Response> {
  let state = _state;

  return async (context) => {
    const [first, ...rest] = middlewareChain;
    context.key = context.key ?? Math.ceil(Math.random() * 10000).toString();

    if (options.logging) console.time(`${context.key}:${first.name}`);

    const [nextState, response] = await first(
      state,
      context,
      createNext(rest, options)
    );
    state = nextState;

    if (options.logging) console.timeEnd(`${context.key}:${first.name}`);
    return response;
  };
}
