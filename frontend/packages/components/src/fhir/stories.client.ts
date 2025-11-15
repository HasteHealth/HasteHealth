import createHTTPClient from "@haste-health/client/http";

const OPEN_URL = "https://open-api.haste-health.app/w/system";
export function createStorybookClient() {
  const client = createHTTPClient({
    url: OPEN_URL,
  });

  return client;
}
