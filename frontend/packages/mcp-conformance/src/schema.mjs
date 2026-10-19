// JSON Schema checks. Without `$schema`, MCP schemas are JSON Schema 2020-12
// (Basic, JSON Schema Usage).
import { Ajv2020 } from "ajv/dist/2020.js";
import assert from "node:assert/strict";

// Not strict: tool schemas may carry their own annotation keywords.
const ajv = new Ajv2020({ allErrors: true, strict: false });

/**
 * Fails unless `schema` is valid 2020-12. With `resolveRefs` it must also
 * compile, so every `$ref` resolves without a network fetch.
 */
export function assertValidSchema(schema, label, { resolveRefs = true } = {}) {
  assert.ok(
    ajv.validateSchema(schema),
    `${label}: ${ajv.errorsText(ajv.errors)}`,
  );
  if (resolveRefs) ajv.compile(schema);
}

/** Fails unless `value` matches `schema`. */
export function assertMatches(schema, value, label) {
  const validate = ajv.compile(schema);
  assert.ok(validate(value), `${label}: ${ajv.errorsText(validate.errors)}`);
}
