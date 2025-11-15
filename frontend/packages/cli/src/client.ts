import httpClient from "@haste-health/client/http";
import { R4, R4B } from "@haste-health/fhir-types/versions";

import { Tenant, getCurrentTenant, loadConfig } from "./config.js";

function configureAuthHeader(tenant: Tenant) {
  return {
    headers: {
      Authorization: `Basic ${btoa(
        `${tenant.auth.client_id}:${tenant.auth.client_secret}`
      )}`,
    },
  };
}

type TokenResponse = {
  access_token: string;
  token_type: string;
  expires_in: number;
};

function isTokenResponseValid(
  timeReceived: number,
  tokenResponse: TokenResponse | undefined,
  // Auto setting window to 10 minutes.
  window: number = 600
) {
  if (!tokenResponse) return false;
  const curSeconds = new Date().getTime() / 1000;

  return curSeconds - timeReceived < tokenResponse.expires_in - window;
}

export async function createClient(location: string) {
  const config = loadConfig(location);
  const tenant = getCurrentTenant(location, config);
  if (!tenant) throw new Error("No tenant configured run config add-tenant.");

  let accessTokenResponse: TokenResponse | undefined = undefined;
  let timeSent = new Date().getTime() / 1000;

  let wellknown = undefined;

  if (tenant.oidc_discovery_uri) {
    wellknown = await fetch(tenant.oidc_discovery_uri).then((res) =>
      res.json()
    );
  }

  return httpClient({
    url: (fhirversion) => {
      switch (fhirversion) {
        case R4:
          return tenant.r4_url;
        case R4B:
          return tenant.r4b_url;
        default:
          throw new Error(`Unsupported FHIR version: ${fhirversion}`);
      }
    },
    getAccessToken:
      tenant.auth.type === "public"
        ? undefined
        : async function () {
            if (
              accessTokenResponse &&
              isTokenResponseValid(timeSent, accessTokenResponse)
            ) {
              return accessTokenResponse.access_token;
            }

            timeSent = new Date().getTime() / 1000;
            const response = await fetch(wellknown.token_endpoint, {
              method: "POST",
              body: new URLSearchParams({
                grant_type: "client_credentials",
              }),
              ...configureAuthHeader(tenant),
            });

            accessTokenResponse = await response.json();

            if (response.status >= 400) {
              throw new Error(JSON.stringify(accessTokenResponse));
            }
            if (!accessTokenResponse) {
              throw new Error("No access token received");
            }

            return accessTokenResponse.access_token;
          },
  });
}
