// Every listed tool: its definition against the MCP Tool schema, then calls
// whose results are checked against MCP (src/tools.mjs) and against FHIR R4
// (https://hl7.org/fhir/R4/http.html, search.html), quoted per test.
//
// A tool with no case fails "every listed tool is exercised". A test titled
// "known gap: ..." checks a requirement the server does not meet yet.
import { ToolSchema } from "@modelcontextprotocol/sdk/types.js";
import assert from "node:assert/strict";
import { randomUUID } from "node:crypto";
import { after, describe, test } from "node:test";
import { assertValidSchema } from "../src/schema.mjs";
import { connect } from "../src/server.mjs";
import { eventually, toolCaller } from "../src/tools.mjs";
import { OPERATION_CASES } from "./operation_cases.mjs";

const client = await connect();
const { tools } = await client.listTools();
const { ok, fails, refused } = toolCaller(client, tools);

// Resources the suite creates, deleted after the run.
const created = [];
async function create(resource) {
  const result = await ok("fhir_r4_create", {
    resourceType: resource.resourceType,
    resource,
  });
  created.push(result);
  return result;
}
after(async () => {
  for (const { resourceType, id } of created.reverse()) {
    await client
      .callTool({ name: "fhir_r4_delete", arguments: { resourceType, id } })
      .catch(() => {});
  }
  await client.close();
});

// A tag shared by the Patients this run creates, so searches find only them.
const SYSTEM = "urn:haste-health:mcp-conformance";
const tagged = (value = randomUUID()) => ({ system: SYSTEM, value });
const token = ({ system, value }) => `${system}|${value}`;

describe("tool definitions", () => {
  // Tool Names: "SHOULD be unique within a server".
  test("tool names are unique", () => {
    const names = tools.map((tool) => tool.name);
    assert.deepEqual(
      names.filter((name, i) => names.indexOf(name) !== i),
      [],
    );
  });

  for (const tool of tools) {
    test(tool.name, () => {
      ToolSchema.parse(tool);
      // Tool Names: "between 1 and 128 characters" of "A-Z, a-z", "0-9",
      // "_", "-" and ".".
      assert.match(tool.name, /^[A-Za-z0-9_.-]{1,128}$/);
      // inputSchema "MUST be a valid JSON Schema object"; both schemas have
      // root type "object" (schema.ts).
      assert.equal(tool.inputSchema.type, "object");
      assertValidSchema(tool.inputSchema, "inputSchema");
      if (tool.outputSchema) {
        assert.equal(tool.outputSchema.type, "object");
        assertValidSchema(tool.outputSchema, "outputSchema");
      }
    });
  }
});

// One Patient through its life cycle; later tests use what earlier ones wrote.
describe("FHIR REST tools", () => {
  const identifier = tagged();
  let patient;

  test("fhir_r4_capabilities", async () => {
    const capabilities = await ok("fhir_r4_capabilities", {});
    assert.equal(capabilities.resourceType, "CapabilityStatement");
  });

  test("fhir_r4_get_search_parameters", async () => {
    const parameters = await ok("fhir_r4_get_search_parameters", {
      resourceType: "Patient",
    });
    assert.match(JSON.stringify(parameters), /birthdate/);
    await fails("fhir_r4_get_search_parameters", { resourceType: "NotAType" });
  });

  test("fhir_r4_get_resource_schema", async () => {
    const schema = await ok("fhir_r4_get_resource_schema", {
      resourceType: "Patient",
    });
    assertValidSchema(schema, "Patient schema", { resolveRefs: false });
    await fails("fhir_r4_get_resource_schema", { resourceType: "NotAType" });
  });

  test("fhir_r4_create", async () => {
    patient = await create({
      resourceType: "Patient",
      identifier: [identifier],
      birthDate: "1990-04-01",
    });
    assert.equal(patient.resourceType, "Patient");
    assert.ok(patient.id);

    // Tools, Error Handling: input validation errors are tool execution
    // errors.
    await fails("fhir_r4_create", {
      resourceType: "Patient",
      resource: { resourceType: "Patient", birthDate: "not a date" },
    });
    await fails("fhir_r4_create", {
      resourceType: "Observation",
      resource: { resourceType: "Patient" },
    });
  });

  // read: "a GET for an unknown resource returns 404".
  test("fhir_r4_read", async () => {
    const read = await ok("fhir_r4_read", {
      resourceType: "Patient",
      id: patient.id,
    });
    assert.deepEqual(read.identifier, [identifier]);
    await fails("fhir_r4_read", { resourceType: "Patient", id: randomUUID() });
  });

  // update: "If no id element is provided, or the id disagrees with the id
  // in the URL, the server SHALL respond with an HTTP 400".
  test("fhir_r4_update", async () => {
    const updated = await ok("fhir_r4_update", {
      resourceType: "Patient",
      id: patient.id,
      resource: { ...patient, active: true },
    });
    assert.equal(updated.active, true);
    assert.notEqual(updated.meta.versionId, patient.meta.versionId);

    await fails("fhir_r4_update", {
      resourceType: "Patient",
      id: patient.id,
      resource: { ...patient, id: randomUUID() },
    });
  });

  // RFC 6902: an unknown op is an error.
  test("fhir_r4_patch", async () => {
    const patched = await ok("fhir_r4_patch", {
      resourceType: "Patient",
      id: patient.id,
      patches: [{ op: "replace", path: "/active", value: false }],
    });
    assert.equal(patched.active, false);

    await fails("fhir_r4_patch", {
      resourceType: "Patient",
      id: patient.id,
      patches: [{ op: "frobnicate", path: "/active" }],
    });
  });

  test("fhir_r4_vread", async () => {
    const first = await ok("fhir_r4_vread", {
      resourceType: "Patient",
      id: patient.id,
      versionId: patient.meta.versionId,
    });
    assert.equal(first.active, undefined);

    await fails("fhir_r4_vread", {
      resourceType: "Patient",
      id: patient.id,
      versionId: "no-such-version",
    });
  });

  test("fhir_r4_history_instance", async () => {
    const history = await ok("fhir_r4_history_instance", {
      resourceType: "Patient",
      id: patient.id,
    });
    assert.equal(history.type, "history");
    assert.equal(history.entry.length, 3, "create, update and patch");
  });

  // history: "_count: The server is not bound to return the number
  // requested, but cannot return more".
  test("fhir_r4_history_type", async () => {
    const history = await ok("fhir_r4_history_type", {
      resourceType: "Patient",
      _count: "2",
    });
    assert.equal(history.type, "history");
    assert.ok(history.entry.length <= 2);

    await fails("fhir_r4_history_type", {
      resourceType: "Patient",
      _since: "yesterday",
    });
  });

  // Search: total "is the number of resources that match"; a repeated
  // parameter is "an AND search parameter"; "Where the content of the
  // parameter is syntactically incorrect, servers SHOULD return an error."
  test("fhir_r4_search", async () => {
    const bundle = await eventually(async () => {
      const result = await ok("fhir_r4_search", {
        resourceType: "Patient",
        search_parameters: {
          identifier: token(identifier),
          _total: "accurate",
        },
      });
      assert.equal(result.entry?.length, 1);
      return result;
    });
    assert.equal(bundle.type, "searchset");
    assert.equal(bundle.total, 1);

    const between = await ok("fhir_r4_search", {
      resourceType: "Patient",
      search_parameters: {
        identifier: token(identifier),
        birthdate: ["ge1990-01-01", "lt1991-01-01"],
      },
    });
    assert.equal(between.entry?.length, 1);

    await fails("fhir_r4_search", {
      resourceType: "Patient",
      search_parameters: { birthdate: "not-a-date" },
    });
  });

  // Batch/Transaction: the response "contains one entry for each entry in
  // the request, in the same order".
  test("fhir_r4_transaction", async () => {
    const patientUrl = `urn:uuid:${randomUUID()}`;
    const response = await ok("fhir_r4_transaction", {
      bundle: {
        resourceType: "Bundle",
        type: "transaction",
        entry: [
          {
            fullUrl: patientUrl,
            resource: { resourceType: "Patient" },
            request: { method: "POST", url: "Patient" },
          },
          {
            resource: {
              resourceType: "Observation",
              status: "final",
              code: { text: "conformance" },
              subject: { reference: patientUrl },
            },
            request: { method: "POST", url: "Observation" },
          },
        ],
      },
    });
    assert.equal(response.type, "transaction-response");
    const [txPatient, txObservation] = response.entry.map((e) => e.resource);
    created.push(txPatient, txObservation);
    assert.equal(txObservation.subject.reference, `Patient/${txPatient.id}`);

    await fails("fhir_r4_transaction", { bundle: { resourceType: "Patient" } });
  });

  test("fhir_r4_batch", async () => {
    const response = await ok("fhir_r4_batch", {
      bundle: {
        resourceType: "Bundle",
        type: "batch",
        entry: [
          { request: { method: "GET", url: `Patient/${patient.id}` } },
          { request: { method: "GET", url: `Patient/${randomUUID()}` } },
        ],
      },
    });
    assert.equal(response.type, "batch-response");
    assert.equal(response.entry.length, 2);
    assert.equal(response.entry[0].resource?.id, patient.id);
  });

  test("fhir_r4_delete", async () => {
    await ok("fhir_r4_delete", { resourceType: "Patient", id: patient.id });
    created.splice(created.indexOf(patient), 1);
    await fails("fhir_r4_read", { resourceType: "Patient", id: patient.id });
  });
});

// FHIR requirements that each set up their own data.
describe("FHIR REST semantics", () => {
  // read: "a GET for a deleted resource returns a 410 status code".
  test("known gap: a read of a deleted resource is 410 Gone", async () => {
    const { id } = await ok("fhir_r4_create", {
      resourceType: "Patient",
      resource: { resourceType: "Patient" },
    });
    await ok("fhir_r4_delete", { resourceType: "Patient", id });
    const error = await fails("fhir_r4_read", { resourceType: "Patient", id });
    assert.match(error, /deleted|gone|410/i);
  });

  // Bundle.entry.response.status is 1..1 in a batch-response.
  test("known gap: batch-response entries carry response.status", async () => {
    const response = await ok("fhir_r4_batch", {
      bundle: {
        resourceType: "Bundle",
        type: "batch",
        entry: [{ request: { method: "GET", url: `Patient/${randomUUID()}` } }],
      },
    });
    assert.match(response.entry[0].response?.status ?? "", /^404/);
  });

  // Paging: a server that pages "SHALL conform to this method", links
  // including next.
  test("known gap: a partial search page links to the next page", async () => {
    const identifier = tagged();
    await create({ resourceType: "Patient", identifier: [identifier] });
    await create({ resourceType: "Patient", identifier: [identifier] });

    const page = await eventually(async () => {
      const result = await ok("fhir_r4_search", {
        resourceType: "Patient",
        search_parameters: {
          identifier: token(identifier),
          _count: "1",
          _total: "accurate",
        },
      });
      assert.equal(result.total, 2);
      return result;
    });
    assert.equal(page.entry.length, 1);
    assert.ok(page.link?.some((link) => link.relation === "next"));
  });
});

describe("FHIR operation tools", () => {
  for (const tool of tools.filter((t) => t.name.startsWith("fhir_op_"))) {
    const run = OPERATION_CASES[tool.name];
    if (run) test(tool.name, () => run({ ok, fails, refused, create }));
  }

  // The OperationDefinition makes user_agent 0..1; failures are tool
  // execution errors.
  test("known gap: fhir_op_delete_refresh_token without user_agent", async () => {
    const client_id = `conformance-${randomUUID()}`;
    assert.equal(
      await fails("fhir_op_delete_refresh_token", { client_id }),
      await fails("fhir_op_delete_refresh_token", {
        client_id,
        user_agent: "conformance",
      }),
    );
  });
});

describe("coverage", () => {
  const exercised = new Set([
    "fhir_r4_capabilities",
    "fhir_r4_get_search_parameters",
    "fhir_r4_get_resource_schema",
    "fhir_r4_create",
    "fhir_r4_read",
    "fhir_r4_update",
    "fhir_r4_patch",
    "fhir_r4_vread",
    "fhir_r4_history_instance",
    "fhir_r4_history_type",
    "fhir_r4_search",
    "fhir_r4_transaction",
    "fhir_r4_batch",
    "fhir_r4_delete",
    ...Object.keys(OPERATION_CASES),
  ]);

  test("every listed tool is exercised", () => {
    assert.deepEqual(
      tools.map((t) => t.name).filter((name) => !exercised.has(name)),
      [],
    );
  });

  test("every operation case is for a listed tool", () => {
    const listed = new Set(tools.map((t) => t.name));
    assert.deepEqual(
      Object.keys(OPERATION_CASES).filter((name) => !listed.has(name)),
      [],
    );
  });
});
