// The MCP server under test: raw JSON-RPC over HTTP for protocol checks, the
// official MCP SDK client for tool calls.
//
// env: HASTE_API_URI, TENANT, PROJECT, CLIENT_ID, CLIENT_SECRET
import { Client } from "@modelcontextprotocol/sdk/client/index.js";
import { StreamableHTTPClientTransport } from "@modelcontextprotocol/sdk/client/streamableHttp.js";

const {
  HASTE_API_URI = "http://localhost:3000",
  TENANT = "my-health",
  PROJECT = "system",
  CLIENT_ID = "testing",
  CLIENT_SECRET = "testing",
} = process.env;

export { PROJECT, TENANT };

const apiUri = HASTE_API_URI.replace(/\/$/, "");
export const mcpUrl = `${apiUri}/w/${TENANT}/${PROJECT}/api/v1/mcp`;

// The specification version the suite checks against.
export const PROTOCOL_VERSION = "2025-11-25";

let token;

/** A client-credentials access token, fetched once. */
export async function accessToken() {
  if (token) return token;
  const discovery = await (
    await fetch(
      `${apiUri}/.well-known/openid-configuration/w/${TENANT}/${PROJECT}`,
    )
  ).json();
  const response = await fetch(discovery.token_endpoint, {
    method: "POST",
    headers: { "Content-Type": "application/x-www-form-urlencoded" },
    body: new URLSearchParams({
      grant_type: "client_credentials",
      client_id: CLIENT_ID,
      client_secret: CLIENT_SECRET,
    }),
  });
  if (!response.ok) throw new Error(`token: ${await response.text()}`);
  token = (await response.json()).access_token;
  return token;
}

/**
 * POSTs `body` (a string as is, anything else as JSON) to the MCP endpoint.
 * `headers` override the defaults (null leaves one out); `auth: false` sends
 * no token.
 */
export async function post(body, { headers = {}, auth = true } = {}) {
  const all = {
    "Content-Type": "application/json",
    Accept: "application/json, text/event-stream",
    "MCP-Protocol-Version": PROTOCOL_VERSION,
    Authorization: auth ? `Bearer ${await accessToken()}` : null,
    ...headers,
  };
  const response = await fetch(mcpUrl, {
    method: "POST",
    headers: Object.fromEntries(
      Object.entries(all).filter(([, value]) => value !== null),
    ),
    body: typeof body === "string" ? body : JSON.stringify(body),
  });
  const text = await response.text();
  return {
    status: response.status,
    headers: response.headers,
    text,
    json: parseMessage(text),
  };
}

// A JSON body, or the first message of an SSE stream.
function parseMessage(text) {
  const event = /^data:(.*)$/m.exec(text);
  try {
    return JSON.parse(event ? event[1] : text);
  } catch {
    return undefined;
  }
}

let nextId = 1;

/** Sends a JSON-RPC request; the response carries the `id` it was sent with. */
export async function rpc(method, params, { id = nextId++, ...options } = {}) {
  const response = await post({ jsonrpc: "2.0", id, method, params }, options);
  return { ...response, id };
}

export function initializeParams(protocolVersion = PROTOCOL_VERSION) {
  return {
    protocolVersion,
    capabilities: {},
    clientInfo: { name: "haste-health-mcp-conformance", version: "1.0.0" },
  };
}

/** An initialized SDK client; it validates every message it receives. */
export async function connect() {
  const client = new Client({
    name: "haste-health-mcp-conformance",
    version: "1.0.0",
  });
  await client.connect(
    new StreamableHTTPClientTransport(new URL(mcpUrl), {
      requestInit: {
        headers: { Authorization: `Bearer ${await accessToken()}` },
      },
    }),
  );
  return client;
}
