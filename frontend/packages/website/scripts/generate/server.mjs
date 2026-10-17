// A snapshot of what the local Haste Health server serves, so the site
// documents the server rather than the artifacts it was built from:
//
//   GET [base]/metadata             -> static/fhir/R4/metadata.json
//   GET [base]/OperationDefinition  -> static/fhir/R4/OperationDefinition/<id>.json
//   MCP tools/list                  -> static/mcp/tools.json
//
// The server, tenant, project and client come from HASTE_HEALTH_* variables,
// read from .env when the environment does not set them (see .env.example).
// The client signs in with the OAuth client-credentials grant; one that only
// allows HTTP Basic (the `basic_auth` grant type) is used that way instead.
//
// Two things are normalized so the committed snapshot is the same whichever
// local server produced it: the MCP tool schemas' `$ref`s point at the public
// API (HASTE_HEALTH_PUBLIC_URL, https://api.haste.health by default) rather
// than at localhost, and their string enums are sorted, since the server
// emits them in hash order.

import path from "node:path";

import {
  MCP_TOOLS,
  METADATA,
  OPERATION_DEFINITIONS,
  loadEnv,
  relative,
  removeStale,
  writeJson,
} from "./lib.mjs";

function settings() {
  loadEnv();
  const env = process.env;
  const clientId = env.HASTE_HEALTH_CLIENT_ID ?? env.HASTE_HEALTH_USERNAME;
  const clientSecret =
    env.HASTE_HEALTH_CLIENT_SECRET ?? env.HASTE_HEALTH_PASSWORD;
  if (!clientId || !clientSecret) {
    throw new Error(
      "HASTE_HEALTH_CLIENT_ID and HASTE_HEALTH_CLIENT_SECRET must be set, in the environment or in .env (see .env.example).",
    );
  }
  const url = (env.HASTE_HEALTH_URL ?? "http://localhost:3000").replace(
    /\/+$/,
    "",
  );
  const tenant = env.HASTE_HEALTH_TENANT ?? "my-health";
  const project = env.HASTE_HEALTH_PROJECT ?? "system";
  const base = `${url}/w/${tenant}/${project}`;
  return {
    clientId,
    clientSecret,
    publicUrl: (
      env.HASTE_HEALTH_PUBLIC_URL ?? "https://api.haste.health"
    ).replace(/\/+$/, ""),
    fhir: `${base}/api/v1/fhir`,
    mcp: `${base}/api/v1/mcp`,
    discovery: `${url}/.well-known/openid-configuration/w/${tenant}/${project}`,
  };
}

/** The useful part of an error body: OperationOutcome issues, OAuth or JSON-RPC errors. */
function reason(text) {
  try {
    const json = JSON.parse(text);
    return (
      json.issue?.map((issue) => issue.diagnostics ?? issue.code).join("; ") ??
      json.error_description ??
      json.error?.message ??
      text.slice(0, 200)
    );
  } catch {
    return text.slice(0, 200);
  }
}

async function request(url, init = {}) {
  let response;
  try {
    response = await fetch(url, init);
  } catch (error) {
    const cause = error.cause?.message ?? error.message;
    throw new Error(`Cannot reach ${url} (${cause}). Is the server running?`);
  }
  const text = await response.text();
  if (!response.ok) {
    throw new Error(
      `${init.method ?? "GET"} ${url} returned ${response.status}: ${reason(text)}`,
    );
  }
  return text ? JSON.parse(text) : undefined;
}

/** An Authorization header: a client-credentials token, or Basic for a client that only allows that. */
async function authorization(server) {
  const basic = `Basic ${Buffer.from(`${server.clientId}:${server.clientSecret}`).toString("base64")}`;
  const discovery = await request(server.discovery);
  const response = await fetch(discovery.token_endpoint, {
    method: "POST",
    headers: { "Content-Type": "application/x-www-form-urlencoded" },
    body: new URLSearchParams({
      grant_type: "client_credentials",
      client_id: server.clientId,
      client_secret: server.clientSecret,
    }),
  });
  if (!response.ok) {
    console.log(
      `  client-credentials grant refused (${reason(await response.text())}); using HTTP Basic`,
    );
    return basic;
  }
  const token = await response.json();
  return `${token.token_type ?? "Bearer"} ${token.access_token}`;
}

/** Every resource of `type` the server lists, following the bundle's next links. */
async function searchAll(server, headers, type) {
  const resources = [];
  let url = `${server.fhir}/${type}?_count=1000`;
  while (url) {
    const bundle = await request(url, { headers });
    resources.push(
      ...(bundle.entry ?? [])
        .map((entry) => entry.resource)
        .filter((resource) => resource?.resourceType === type),
    );
    url = bundle.link?.find((link) => link.relation === "next")?.url;
  }
  return resources;
}

/** Sorts every string enum in a JSON Schema, in place; their order carries no meaning. */
function sortEnums(schema) {
  if (Array.isArray(schema)) {
    schema.forEach(sortEnums);
  } else if (schema && typeof schema === "object") {
    for (const [key, value] of Object.entries(schema)) {
      if (
        key === "enum" &&
        Array.isArray(value) &&
        value.every((v) => typeof v === "string")
      ) {
        value.sort((a, b) => a.localeCompare(b));
      } else {
        sortEnums(value);
      }
    }
  }
  return schema;
}

async function mcpTools(server, headers) {
  const body = await request(server.mcp, {
    method: "POST",
    headers: { ...headers, "Content-Type": "application/json" },
    body: JSON.stringify({ jsonrpc: "2.0", id: 1, method: "tools/list" }),
  });
  if (body.error) throw new Error(`MCP tools/list: ${body.error.message}`);
  // The schemas reference the server's own /schemas/fhir/<Type> endpoint.
  const text = JSON.stringify(body.result.tools).replaceAll(
    /https?:\/\/[^"/]+\/schemas\//g,
    `${server.publicUrl}/schemas/`,
  );
  return sortEnums(JSON.parse(text));
}

export async function run() {
  const server = settings();
  const headers = { Authorization: await authorization(server) };

  const capabilities = await request(`${server.fhir}/metadata`, { headers });
  if (capabilities.resourceType !== "CapabilityStatement") {
    throw new Error(
      `${server.fhir}/metadata did not return a CapabilityStatement.`,
    );
  }
  const definitions = (await searchAll(server, headers, "OperationDefinition"))
    .filter((definition) => definition.id)
    .sort((a, b) => a.id.localeCompare(b.id));
  const tools = await mcpTools(server, headers);

  writeJson(METADATA, capabilities, { pretty: true });
  for (const definition of definitions) {
    writeJson(
      path.join(OPERATION_DEFINITIONS, `${definition.id}.json`),
      definition,
      {
        pretty: true,
      },
    );
  }
  removeStale(
    OPERATION_DEFINITIONS,
    ".json",
    new Set(definitions.map((d) => d.id)),
  );
  writeJson(MCP_TOOLS, tools, { pretty: true });

  const resources = capabilities.rest?.[0]?.resource?.length ?? 0;
  return `${server.fhir}: ${resources} resource types, ${definitions.length} operations and ${tools.length} MCP tools, in ${relative(path.dirname(METADATA))} and ${relative(MCP_TOOLS)}`;
}
