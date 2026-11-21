// Tool calls, each result checked against the Tools page of the spec.
import { CallToolResultSchema } from "@modelcontextprotocol/sdk/types.js";
import assert from "node:assert/strict";
import { isDeepStrictEqual } from "node:util";
import { assertMatches } from "./schema.mjs";
import { rpc } from "./server.mjs";

/** Tool-call helpers bound to an SDK client and its tools/list result. */
export function toolCaller(client, tools) {
  async function call(name, args) {
    const result = CallToolResultSchema.parse(
      await client.callTool({ name, arguments: args }),
    );
    if (result.isError) return result;

    const { outputSchema } = tools.find((t) => t.name === name) ?? {};
    // Output Schema: "Servers MUST provide structured results that conform
    // to this schema."
    if (outputSchema) {
      assert.ok(result.structuredContent, "structuredContent");
      assertMatches(outputSchema, result.structuredContent, name);
    }
    // Structured Content: it "SHOULD also return the serialized JSON in a
    // TextContent block."
    if (result.structuredContent) {
      assert.ok(
        result.content.some(
          (block) =>
            block.type === "text" &&
            isDeepStrictEqual(parseJson(block.text), result.structuredContent),
        ),
        "structuredContent is repeated as JSON text",
      );
    }
    return result;
  }

  return {
    /** The call succeeds; returns structuredContent. */
    async ok(name, args) {
      const result = await call(name, args);
      assert.notEqual(result.isError, true, `${name}: ${text(result)}`);
      return result.structuredContent;
    },

    /** The call is a tool execution error (isError); returns its text. */
    async fails(name, args) {
      const result = await call(name, args);
      assert.equal(result.isError, true, `${name} succeeded`);
      return text(result);
    },

    /**
     * The call is refused: HTTP 403 (Authorization, Error Handling) or a
     * tool execution error.
     */
    async refused(name, args) {
      const response = await rpc("tools/call", { name, arguments: args });
      if (response.status === 403) return;
      const result = CallToolResultSchema.parse(response.json?.result);
      assert.equal(result.isError, true, `${name} succeeded`);
    },
  };
}

const text = (result) =>
  result.content.map((block) => block.text ?? "").join("\n");

function parseJson(value) {
  try {
    return JSON.parse(value);
  } catch {
    return undefined;
  }
}

/** Retries `check` until it passes, for reads of the search index. */
export async function eventually(check, timeoutMs = 30000) {
  const deadline = Date.now() + timeoutMs;
  for (;;) {
    try {
      return await check();
    } catch (error) {
      if (Date.now() > deadline) throw error;
      await new Promise((resolve) => setTimeout(resolve, 500));
    }
  }
}
