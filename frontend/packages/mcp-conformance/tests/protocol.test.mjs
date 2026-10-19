// The base protocol, lifecycle, Streamable HTTP transport and authorization,
// checked on raw HTTP. Comments quote the section of
// https://modelcontextprotocol.io/specification/2025-11-25 a test checks.
//
// A test titled "known gap: ..." checks a requirement the server does not
// meet yet; CI reports those in a separate step that may fail.
import {
  EmptyResultSchema,
  ErrorCode,
  InitializeResultSchema,
  JSONRPCErrorResponseSchema,
  JSONRPCResultResponseSchema,
  ListToolsResultSchema,
  SUPPORTED_PROTOCOL_VERSIONS,
} from "@modelcontextprotocol/sdk/types.js";
import assert from "node:assert/strict";
import { describe, test } from "node:test";
import {
  accessToken,
  initializeParams,
  mcpUrl,
  post,
  PROTOCOL_VERSION,
  rpc,
} from "../src/server.mjs";

// Transports: a request "MUST either return Content-Type: text/event-stream
// ... or Content-Type: application/json".
function assertMessageContentType(response) {
  assert.match(
    response.headers.get("content-type") ?? "",
    /^(application\/json|text\/event-stream)\b/,
    response.text,
  );
}

// Basic, Result Responses: "MUST include the same ID as the request" and "a
// result field".
function assertResult(response) {
  assert.equal(response.status, 200, response.text);
  assertMessageContentType(response);
  const message = JSONRPCResultResponseSchema.parse(response.json);
  assert.deepEqual(message.id, response.id);
  return message.result;
}

// Basic, Error Responses: the same ID, and "an error field with a code and
// message". "Error codes MUST be integers."
function assertError(response) {
  assertMessageContentType(response);
  const { id, error } = JSONRPCErrorResponseSchema.parse(response.json);
  assert.deepEqual(id, response.id);
  assert.ok(Number.isInteger(error.code));
  return error;
}

// Transports: a notification or response the server accepts gets "HTTP
// status code 202 Accepted with no body".
function assertAccepted(response) {
  assert.equal(response.status, 202, response.text);
  assert.equal(response.text, "");
}

// JSON-RPC 2.0, which MCP messages "MUST follow": input that is not a valid
// request is answered with `code` and a null id. Transports allows an HTTP
// error status with "a JSON-RPC error response that has no id".
function assertRejected(response, code) {
  if (response.json === undefined) {
    assert.ok(response.status >= 400 && response.status < 500, response.text);
    return;
  }
  const { error } = JSONRPCErrorResponseSchema.parse({
    id: null,
    ...response.json,
  });
  assert.equal(error.code, code);
}

describe("lifecycle", () => {
  // Initialization: "The server MUST respond with its own capabilities and
  // information". Tools: "Servers that support tools MUST declare the tools
  // capability".
  test("initialize returns an InitializeResult with the tools capability", async () => {
    const result = InitializeResultSchema.parse(
      assertResult(await rpc("initialize", initializeParams())),
    );
    assert.ok(result.capabilities.tools);
  });

  // Version Negotiation: "If the server supports the requested protocol
  // version, it MUST respond with the same version." The suite checks
  // 2025-11-25, so the server has to support it.
  test(`known gap: initialize negotiates ${PROTOCOL_VERSION}`, async () => {
    const result = assertResult(await rpc("initialize", initializeParams()));
    assert.equal(result.protocolVersion, PROTOCOL_VERSION);
  });

  // "Otherwise, the server MUST respond with another protocol version it
  // supports."
  test("initialize answers an unknown version with a supported one", async () => {
    const result = assertResult(
      await rpc("initialize", initializeParams("1900-01-01")),
    );
    assert.ok(SUPPORTED_PROTOCOL_VERSIONS.includes(result.protocolVersion));
  });

  test("known gap: notifications/initialized gets 202", async () => {
    assertAccepted(
      await post({ jsonrpc: "2.0", method: "notifications/initialized" }),
    );
  });

  // Cancellation: "Invalid cancellation notifications SHOULD be ignored:
  // Unknown request IDs".
  test("known gap: notifications/cancelled for an unknown request gets 202", async () => {
    assertAccepted(
      await post({
        jsonrpc: "2.0",
        method: "notifications/cancelled",
        params: { requestId: "conformance-unknown" },
      }),
    );
  });

  // Transports: a response the server cannot accept gets "an HTTP error
  // status code".
  test("a JSON-RPC response is accepted or refused with an HTTP error", async () => {
    const response = await post({ jsonrpc: "2.0", id: "unsent", result: {} });
    if (response.status !== 202) {
      assert.ok(response.status >= 400, `status ${response.status}`);
    }
  });
});

describe("base protocol", () => {
  // Ping: "The receiver MUST respond promptly with an empty response".
  test("known gap: ping returns an empty result", async () => {
    EmptyResultSchema.strict().parse(assertResult(await rpc("ping")));
  });

  // Requests: "Requests MUST include a string or integer ID."
  test("a string id is echoed", async () => {
    assertResult(await rpc("tools/list", {}, { id: "conformance-id" }));
  });

  test("an integer id is echoed", async () => {
    assertResult(await rpc("tools/list", {}, { id: 4242 }));
  });

  test("tools/list returns a ListToolsResult", async () => {
    const result = ListToolsResultSchema.parse(
      assertResult(await rpc("tools/list", {})),
    );
    assert.ok(result.tools.length > 0);
  });

  // JSON-RPC 2.0: -32601 "The method does not exist / is not available."
  test("known gap: an unknown method is -32601 Method not found", async () => {
    const error = assertError(await rpc("conformance/unknown", {}));
    assert.equal(error.code, ErrorCode.MethodNotFound);
  });

  // Tools, Error Handling: unknown tools are protocol errors.
  test("an unknown tool is a JSON-RPC error", async () => {
    assertError(
      await rpc("tools/call", { name: "conformance_unknown", arguments: {} }),
    );
  });

  test("a body that is not JSON is a -32700 Parse error", async () => {
    assertRejected(await post("{not json"), ErrorCode.ParseError);
  });

  test("a message without a method is a -32600 Invalid Request", async () => {
    assertRejected(
      await post({ jsonrpc: "2.0", id: 1 }),
      ErrorCode.InvalidRequest,
    );
  });
});

describe("Streamable HTTP transport", () => {
  // Listening for Messages: "MUST either return Content-Type:
  // text/event-stream in response to this HTTP GET, or else return HTTP 405".
  test("GET opens an SSE stream or is 405", async () => {
    const controller = new AbortController();
    const response = await fetch(mcpUrl, {
      headers: {
        Accept: "text/event-stream",
        Authorization: `Bearer ${await accessToken()}`,
      },
      signal: controller.signal,
    });
    controller.abort();
    if (response.status !== 405) {
      assert.match(
        response.headers.get("content-type") ?? "",
        /^text\/event-stream/,
      );
    }
  });

  // Protocol Version Header: "If the server receives a request with an
  // invalid or unsupported MCP-Protocol-Version, it MUST respond with 400 Bad
  // Request."
  test("known gap: an unsupported MCP-Protocol-Version is 400", async () => {
    const response = await rpc(
      "tools/list",
      {},
      {
        headers: { "MCP-Protocol-Version": "1900-01-01" },
      },
    );
    assert.equal(response.status, 400, response.text);
  });

  // "If the server does not receive an MCP-Protocol-Version header ... the
  // server SHOULD assume protocol version 2025-03-26."
  test("a request without MCP-Protocol-Version is served", async () => {
    assertResult(
      await rpc(
        "tools/list",
        {},
        { headers: { "MCP-Protocol-Version": null } },
      ),
    );
  });
});

describe("authorization", () => {
  // Token Handling: "Invalid or expired tokens MUST receive a HTTP 401
  // response." Discovery: a 401 names the metadata in WWW-Authenticate
  // `resource_metadata`, or the metadata is at the well-known URI.
  for (const [label, headers] of [
    ["no token", {}],
    ["an invalid token", { Authorization: "Bearer not-a-token" }],
  ]) {
    test(`a request with ${label} is 401 and leads to the metadata`, async () => {
      const response = await post(
        { jsonrpc: "2.0", id: 1, method: "tools/list", params: {} },
        { auth: false, headers },
      );
      assert.equal(response.status, 401, response.text);
      await assertProtectedResourceMetadata(
        response.headers.get("www-authenticate"),
      );
    });
  }
});

// Authorization Server Location: the metadata "MUST include the
// authorization_servers field containing at least one authorization server".
// RFC 9728 requires `resource`.
async function assertProtectedResourceMetadata(challenge) {
  const { origin, pathname } = new URL(mcpUrl);
  const candidates = [
    /resource_metadata="([^"]+)"/.exec(challenge ?? "")?.[1],
    `${origin}/.well-known/oauth-protected-resource${pathname}`,
    `${origin}/.well-known/oauth-protected-resource`,
  ].filter(Boolean);

  for (const url of candidates) {
    const response = await fetch(url);
    if (!response.ok) continue;
    const metadata = await response.json();
    assert.equal(typeof metadata.resource, "string");
    assert.ok(metadata.authorization_servers?.length > 0);
    return;
  }
  assert.fail(`no protected resource metadata at ${candidates.join(", ")}`);
}
