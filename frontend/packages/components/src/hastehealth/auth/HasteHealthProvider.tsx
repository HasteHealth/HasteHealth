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
import { generateRandomString, hasAuthQueryParams, sha256 } from "./utilities";

const CODE_CHALLENGE_METHOD = "S256";
const state_key = (client_id: string) => `haste-health_state_${client_id}`;
const pkce_code_verifier_key = (client_id: string) =>
  `haste-health_pkce_code_${client_id}`;

function getCookie(name: string): string | null {
  if (typeof document === "undefined") return null;
  const match = document.cookie.match(
    new RegExp("(?:^|; )" + encodeURIComponent(name) + "=([^;]*)"),
  );
  return match ? decodeURIComponent(match[1]) : null;
}

function setCookie(name: string, value: string, maxAgeSeconds = 600) {
  if (typeof document === "undefined") return;
  const encodedName = encodeURIComponent(name);
  const encodedVal = encodeURIComponent(value);
  document.cookie = `${encodedName}=${encodedVal}; Path=/; Max-Age=${maxAgeSeconds}; SameSite=Lax`;
  const host = window.location.hostname;
  if (host.endsWith("localhost")) {
    try {
      document.cookie = `${encodedName}=${encodedVal}; Path=/; Domain=.localhost; Max-Age=${maxAgeSeconds}; SameSite=Lax`;
    } catch {}
  }
}

function removeCookie(name: string) {
  if (typeof document === "undefined") return;
  const encodedName = encodeURIComponent(name);
  document.cookie = `${encodedName}=; Path=/; Max-Age=0; SameSite=Lax`;
  const host = window.location.hostname;
  if (host.endsWith("localhost")) {
    try {
      document.cookie = `${encodedName}=; Path=/; Domain=.localhost; Max-Age=0; SameSite=Lax`;
    } catch {}
  }
}

export function getStoredAuthValue(key: string): string | null {
  try {
    const val = window.sessionStorage.getItem(key);
    if (val) return val;
  } catch {}
  try {
    const val = window.localStorage.getItem(key);
    if (val) return val;
  } catch {}
  try {
    const val = getCookie(key);
    if (val) return val;
  } catch {}
  return null;
}

export function setStoredAuthValue(key: string, value: string) {
  try {
    window.sessionStorage.setItem(key, value);
  } catch {}
  try {
    window.localStorage.setItem(key, value);
  } catch {}
  try {
    setCookie(key, value, 600);
  } catch {}
}

export function removeStoredAuthValue(key: string) {
  try {
    window.sessionStorage.removeItem(key);
  } catch {}
  try {
    window.localStorage.removeItem(key);
  } catch {}
  try {
    removeCookie(key);
  } catch {}
}

function getParsedParameters(): Record<string, string> {
  const parameters = window.location.search
    .slice(1)
    .split("&")
    .map((v) => v.trim())
    .map((p) => p.split("=").map((v) => v.trim()))
    .reduce((acc, [key, v]) => ({ ...acc, [key]: decodeURIComponent(v) }), {});
  return parameters;
}

/**
 * What the token endpoint returns in place of tokens when the request fails.
 */
type TokenErrorResponse = {
  error: string;
  error_description?: string;
};

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
}): Promise<AccessTokenResponse | TokenErrorResponse> {
  const code_verifier = getStoredAuthValue(pkce_code_verifier_key(clientId));
  const localStateParameter = getStoredAuthValue(state_key(clientId));

  if (!parameters.state || !parameters.code) {
    throw new Error("Missing authorization code or state in callback parameters");
  }

  if (localStateParameter && parameters.state !== localStateParameter) {
    throw new Error("Invalid State: authorization response state does not match stored state");
  }

  if (!code_verifier) {
    throw new Error("Invalid Code Verifier: PKCE code verifier not found in storage");
  }

  removeStoredAuthValue(state_key(clientId));
  removeStoredAuthValue(pkce_code_verifier_key(clientId));

  window.history.replaceState(null, "", location.pathname);

  const response: AccessTokenResponse | TokenErrorResponse = await fetch(
    token_endpoint,
    {
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
    },
  ).then((v) => v.json());
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
  setStoredAuthValue(pkce_code_verifier_key(clientId), code_verifier);
  setStoredAuthValue(state_key(clientId), state);

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
  });

  if (!response.ok) {
    throw new Error("Failed to refresh token");
  }

  const responseData = await response.json();

  return responseData as AccessTokenResponse;
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

    void (async (): Promise<void> => {
      dispatch({ type: "SET_LOADING", loading: true });
      try {
        const well_known_uri = new URL(
          `/.well-known/openid-configuration/w/${tenant}/${project}`,
          domain,
        ).toString();

        const well_known: OIDC_WELL_KNOWN = await fetch(well_known_uri).then(
          (v) => v.json(),
        );

        if (hasAuthQueryParams()) {
          const parameters = getParsedParameters();

          if (parameters.error) {
            window.history.replaceState(null, "", location.pathname);
            dispatch({
              type: "ON_ERROR",
              error: parameters.error,
              error_description:
                parameters.error_description ?? "Authentication failed",
              error_uri: parameters.error_uri,
              state: parameters.state,
            });
            console.error("Failed to authenticate:", parameters.error);
            return;
          }

          const codeVerifier = getStoredAuthValue(
            pkce_code_verifier_key(clientId),
          );
          if (!codeVerifier) {
            // Storage was lost or stale query params remained in address bar.
            // Re-initiate authorization cleanly rather than crashing into an unhandled state.
            window.history.replaceState(null, "", location.pathname);
            console.warn(
              "No PKCE code verifier found in storage; re-initiating authorization...",
            );
            await authorize({
              refresh,
              method: authorize_method,
              authorize_endpoint: well_known.authorization_endpoint,
              clientId,
              scope,
              redirectUrl,
            });
            return;
          }

          let payload: AccessTokenResponse | TokenErrorResponse;
          try {
            payload = await exchangeAuthCodeForToken({
              parameters,
              token_endpoint: well_known.token_endpoint,
              redirect_uri: redirectUrl,
              clientId,
            });
          } catch (exchangeErr: any) {
            window.history.replaceState(null, "", location.pathname);
            console.error("Token exchange failed:", exchangeErr);
            dispatch({
              type: "ON_ERROR",
              error: "token_exchange_error",
              error_description:
                exchangeErr?.message ||
                "Failed to exchange authorization code for token.",
            });
            return;
          }

          if ("error" in payload) {
            window.history.replaceState(null, "", location.pathname);
            dispatch({
              type: "ON_ERROR",
              error: payload.error,
              error_description:
                payload.error_description ??
                "The token endpoint rejected the authorization code.",
            });
            console.error("Failed to authenticate:", payload.error);
            return;
          }

          if (!payload.id_token) {
            window.history.replaceState(null, "", location.pathname);
            dispatch({
              type: "ON_ERROR",
              error: "invalid_scope",
              error_description:
                "The token response contained no id token. Check that 'openid' is one of the scopes this application requests and is allowed by its client application.",
            });
            console.error("Failed to authenticate: missing id_token");
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
            payload,
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
                    await authorize({
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
                await authorize({
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
            const redirectPath = getStoredAuthValue("path") ?? "/";
            removeStoredAuthValue("path");
            onRedirectCallback(redirectPath);
          }
        } else {
          setStoredAuthValue(
            "path",
            window.location.href.replace(window.location.origin, ""),
          );
          await authorize({
            refresh,
            method: authorize_method,
            authorize_endpoint: well_known.authorization_endpoint,
            clientId,
            scope,
            redirectUrl,
          });
        }
      } catch (error: any) {
        window.history.replaceState(null, "", location.pathname);
        console.error("Authentication initialization failed:", error);
        dispatch({
          type: "ON_ERROR",
          error: "initialization_error",
          error_description:
            error?.message ||
            "An unexpected error occurred during authentication setup.",
        });
      } finally {
        dispatch({ type: "SET_LOADING", loading: false });
      }
    })();
  }, [clientId, domain, redirectUrl]);

  return (
    <HasteHealthContext.Provider value={state}>
      {state.error ? (
        <div className="min-h-screen flex items-center justify-center bg-[var(--ci-bg)] p-6 text-[var(--ci-ink)]">
          <div className="max-w-md w-full border border-[var(--ci-line)] bg-[var(--ci-surface)] p-6 space-y-4 shadow-sm">
            <div className="flex items-center space-x-2 text-[var(--ci-signal)]">
              <span className="font-mono text-xs uppercase tracking-wider font-semibold">
                Authentication Notice
              </span>
            </div>
            <div className="text-base font-semibold text-[var(--ci-ink)]">
              {state.error.code
                .split("_")
                .map((s) => s.charAt(0).toUpperCase() + s.slice(1))
                .join(" ")}
            </div>
            <p className="text-xs text-[var(--ci-muted)] leading-relaxed">
              {state.error.description}
            </p>
            <div className="pt-2">
              <button
                type="button"
                onClick={() => {
                  window.location.href = window.location.pathname;
                }}
                className="w-full py-2 px-4 bg-[var(--ci-signal)] text-white text-xs font-mono font-medium hover:opacity-90 transition-opacity"
              >
                Sign in again
              </button>
            </div>
          </div>
        </div>
      ) : (
        children
      )}
    </HasteHealthContext.Provider>
  );
}
