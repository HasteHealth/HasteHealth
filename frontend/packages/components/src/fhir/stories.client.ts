import createHTTPClient from "@oxidized-health/client/http";

const OPEN_URL = "https://open-api.oxidized-health.app/w/system";
export function createStorybookClient() {
  const client = createHTTPClient({
    url: OPEN_URL,
  });

  return client;
}
