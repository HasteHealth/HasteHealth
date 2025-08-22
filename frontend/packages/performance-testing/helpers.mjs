function configureAuthHeader(clientId, clientSecret) {
  return {
    headers: {
      Authorization: `Basic ${btoa(`${clientId}:${clientSecret}`)}`,
    },
  };
}

export async function generateAuthToken(context, events, done) {
  const OIDC_WELL_KNOWN_URL = process.env.OIDC_WELL_KNOWN_URL;
  const wellknown = await fetch(OIDC_WELL_KNOWN_URL).then((res) => res.json());

  if (!wellknown.token_endpoint) {
    throw new Error(
      "token_endpoint is not defined in OIDC well-known document",
    );
  }

  const CLIENT_ID = process.env.CLIENT_ID;
  if (!CLIENT_ID) {
    throw new Error("CLIENT_ID is not defined");
  }
  const CLIENT_SECRET = process.env.CLIENT_SECRET;
  if (!CLIENT_SECRET) {
    throw new Error("CLIENT_SECRET is not defined");
  }

  const response = await fetch(wellknown.token_endpoint, {
    method: "POST",
    body: new URLSearchParams({
      grant_type: "client_credentials",
    }),
    ...configureAuthHeader(CLIENT_ID, CLIENT_SECRET),
  });

  const accessTokenResponse = await response.json();

  if (response.status >= 400) {
    throw new Error(JSON.stringify(accessTokenResponse));
  }
  if (!accessTokenResponse) {
    throw new Error("No access token received");
  }

  context.vars.sharedToken = accessTokenResponse.access_token;

  return;
}
