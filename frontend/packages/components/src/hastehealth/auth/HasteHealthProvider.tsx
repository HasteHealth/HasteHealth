import React, { useEffect, useReducer, useRef } from "react";

import { code } from "@haste-health/fhir-types/lib/generated/r4/types";
import { ProjectId, TenantId } from "@haste-health/jwt/types";

import { OperationOutcomeIssueDisplay } from "../../fhir/resources/OperationOutcome";
import HasteHealthContext, {
  AccessTokenResponse,
  HasteHealthContextState,
  InitialContext,
} from "./HasteHealthContext";
import { OIDC_WELL_KNOWN, HasteHealthReducer } from "./reducer";
import {
  conditionalAddTenant,
  generateRandomString,
  hasAuthQueryParams,
  sha256,
} from "./utilities";

const CODE_CHALLENGE_METHOD = "S256";
const state_key = (client_id: string) => `haste-health_state_${client_id}`;
const pkce_code_verifier_key = (client_id: string) =>
  `haste-health_pkce_code_${client_id}`;

function getParsedParameters(): Record<string, string> {
  const parameters = window.location.search
    .slice(1)
    .split("&")
    .map((v) => v.trim())
    .map((p) => p.split("=").map((v) => v.trim()))
    .reduce((acc, [key, v]) => ({ ...acc, [key]: decodeURIComponent(v) }), {});
  return parameters;
}

async function exchangeAuthCodeForToken({
  token_endpoint,
  redirect_uri,
  parameters,
  clientId,
}: {
  redirect_uri: string;
  token_endpoint: string;
  parameters: Record<string, string>;
  clientId: string;
}): Promise<AccessTokenResponse> {
  const code_verifier = window.sessionStorage.getItem(
    pkce_code_verifier_key(clientId)
  );
  const localStateParameter = window.sessionStorage.getItem(
    state_key(clientId)
  );

  if (!parameters.state) throw new Error();
  if (!parameters.code) throw new Error();
  if (parameters.state !== localStateParameter)
    throw new Error("Invalid State");

  window.sessionStorage.removeItem(state_key(clientId));
  window.sessionStorage.removeItem(pkce_code_verifier_key(clientId));

  if (!code_verifier) throw new Error("Invalid Code Verifier");

  window.history.replaceState(null, "", location.pathname);

  const response: AccessTokenResponse = await fetch(token_endpoint, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },

    body: JSON.stringify({
      grant_type: "authorization_code",
      code_verifier,
      redirect_uri,
      code: parameters.code,
      client_id: clientId,
    }),
  }).then((v) => v.json());
  return response;
}

export async function authorize({
  refresh,
  authorize_endpoint,
  clientId,
  scope,
  redirectUrl,
  method = "GET",
}: {
  refresh?: boolean;
  authorize_endpoint: string;
  clientId: string;
  scope: string;
  redirectUrl: string;
  method?: "GET" | "POST";
}) {
  const code_verifier = generateRandomString(43);
  const state = generateRandomString(30);
  const code_challenge = await sha256(code_verifier);
  window.sessionStorage.setItem(
    pkce_code_verifier_key(clientId),
    code_verifier
  );
  window.sessionStorage.setItem(state_key(clientId), state);

  const parameters: Record<string, string> = {
    client_id: clientId,
    redirect_uri: redirectUrl,
    scope:
      refresh && !scope.includes("offline_access")
        ? scope + " offline_access"
        : scope,
    state,
    response_type: "code",
    code_challenge: code_challenge,
    code_challenge_method: CODE_CHALLENGE_METHOD,
  };

  switch (method) {
    case "GET": {
      const queryParameters = Object.keys(parameters)
        .map((key) => `${key}=${parameters[key]}`)
        .join("&");

      const url = new URL(`?${queryParameters}`, authorize_endpoint);

      window.location.replace(url);
      return;
    }
    case "POST": {
      const response = await fetch(authorize_endpoint, {
        method: "POST",
        headers: {
          "Content-Type": "application/x-www-form-urlencoded",
        },
        body: Object.keys(parameters)
          .map((key) => `${key}=${parameters[key]}`)
          .join("&"),
      });

      if (!response.redirected) {
        throw new Error("Failed to redirect");
      }

      window.location.replace(response.url);

      return;
    }
    default: {
      throw new Error("Invalid Method");
    }
  }
}

export async function refreshToken({
  clientId,
  token_endpoint,
  payload,
}: {
  clientId: string;
  payload: AccessTokenResponse;
  token_endpoint: string;
}): Promise<AccessTokenResponse> {
  const parameters: Record<string, unknown> = {
    client_id: clientId,
    grant_type: "refresh_token",
    refresh_token: payload.refresh_token,
  };
  const response = await fetch(token_endpoint, {
    method: "POST",
    headers: {
      "Content-Type": "application/x-www-form-urlencoded",
    },
    body: Object.keys(parameters)
      .map((key) => `${key}=${parameters[key]}`)
      .join("&"),
  }).then((v) => v.json());

  return response as AccessTokenResponse;
}

export function HasteHealthProvider({
  clientId,
  tenant,
  project,
  redirectUrl,
  domain,
  scope,
  authorize_method,
  refresh,
  onRedirectCallback,
  children,
}: Readonly<{
  /**
   * The tenant to use for the authentication.
   */
  tenant: TenantId;
  /**
   * The project to use for the authentication.
   */
  project: ProjectId;
  /**
   * The client id to use for the authentication.
   */
  clientId: string;
  /**
   * The scope to use for the authentication.
   */
  scope: string;
  /**
   * The domain to use for the authentication.
   */
  domain: string;
  /**
   * The redirect url to use for the authentication.
   */
  redirectUrl: string;
  /**
   * The method to use for the authentication.
   */
  authorize_method?: "GET" | "POST";
  /**
   * Whether to use refresh grant for authentication
   */
  refresh?: boolean;
  /**
   * The children to render.
   */
  children: React.ReactNode;
  /**
   *  Allows for SPA to redirect back to initial page.
   * @param initialPath Initial path that started authentication
   */
  onRedirectCallback?: (initialPath: string) => void;
}>) {
  const [state, dispatch] = useReducer(HasteHealthReducer, InitialContext);
  const isInitialized = useRef(false);
  const isRefreshing = useRef(false);

  useEffect(() => {
    if (isInitialized.current) {
      return;
    }
    isInitialized.current = true;

    (async (): Promise<void> => {
      dispatch({ type: "SET_LOADING", loading: true });
      try {
        const well_known_uri = new URL(
          conditionalAddTenant(
            `/api/v1/${project}/oidc/.well-known/openid-configuration`,
            tenant
          ),
          domain
        ).toString();

        const well_known: OIDC_WELL_KNOWN = await fetch(well_known_uri).then(
          (v) => v.json()
        );

        if (hasAuthQueryParams()) {
          const parameters = getParsedParameters();

          if (parameters.error) {
            window.history.replaceState(null, "", location.pathname);
            dispatch({
              type: "ON_ERROR",
              error: parameters.error,
              error_description: parameters.error_description,
              error_uri: parameters.error_uri,
              state: parameters.state,
            });
            console.error("Failed to authenticate");
            return;
          }

          dispatch({
            type: "ON_SUCCESS",

            domain,
            well_known_uri,
            well_known,
            tenant,
            project,
            clientId,
            payload: await exchangeAuthCodeForToken({
              parameters,
              token_endpoint: well_known.token_endpoint,
              redirect_uri: redirectUrl,
              clientId,
            }),
            reAuthenticate: async (state: HasteHealthContextState) => {
              if (!state.payload) {
                throw new Error("Payload is missing");
              }

              if (refresh) {
                if (!isRefreshing.current) {
                  isRefreshing.current = true;
                  try {
                    const payload = await refreshToken({
                      clientId,
                      payload: state.payload,
                      token_endpoint: well_known.token_endpoint,
                    });
                    dispatch({
                      type: "ON_REFRESH",
                      payload,
                    });
                  } catch {
                    // In event that refresh fails we will reauthorize.
                    // This could happen if refresh token is expired.
                    authorize({
                      refresh,
                      method: authorize_method,
                      authorize_endpoint: well_known.authorization_endpoint,
                      scope,
                      clientId,
                      redirectUrl,
                    });
                  } finally {
                    isRefreshing.current = false;
                  }
                }
              } else {
                authorize({
                  refresh,
                  method: authorize_method,
                  authorize_endpoint: well_known.authorization_endpoint,
                  scope,
                  clientId,
                  redirectUrl,
                });
              }
            },
          });

          // Allows for SPA to redirect back.
          if (onRedirectCallback) {
            onRedirectCallback(window.sessionStorage.getItem("path") ?? "/");
          }
        } else {
          window.sessionStorage.setItem(
            "path",
            window.location.href.replace(window.location.origin, "")
          );
          authorize({
            refresh,
            method: authorize_method,
            authorize_endpoint: well_known.authorization_endpoint,
            clientId,
            scope,
            redirectUrl,
          });
        }
      } catch (error) {
        console.error(error);
        throw error;
      } finally {
        dispatch({ type: "SET_LOADING", loading: false });
      }
    })();
  }, [clientId, domain, redirectUrl]);

  return (
    <HasteHealthContext.Provider value={state}>
      {state.error ? (
        <OperationOutcomeIssueDisplay
          issue={{
            severity: "error" as code,
            code: state.error.code as code,
            diagnostics: state.error.description,
          }}
        />
      ) : (
        children
      )}
    </HasteHealthContext.Provider>
  );
}
