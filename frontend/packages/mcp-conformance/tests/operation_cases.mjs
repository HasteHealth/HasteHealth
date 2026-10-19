// One case per fhir_op_* tool, checked against its OperationDefinition
// (artifacts/r4/hastehealth-core) or the FHIR specification of the operation.
// A single `return` output of a resource type is returned as that resource.
//
// Each case gets { ok, fails, refused, create } (see src/tools.mjs);
// `create` makes a resource that is deleted after the run.
import assert from "node:assert/strict";
import { randomUUID } from "node:crypto";
import { mcpUrl, PROJECT } from "../src/server.mjs";
import { eventually } from "../src/tools.mjs";

const GENDER_VALUE_SET = "http://hl7.org/fhir/ValueSet/administrative-gender";
const GENDER_SYSTEM = "http://hl7.org/fhir/administrative-gender";

/** The named output of a Parameters resource. */
function output(parameters, name) {
  assert.equal(parameters.resourceType, "Parameters");
  const found = parameters.parameter?.find((p) => p.name === name);
  assert.ok(found, `output '${name}'`);
  return found;
}

export const OPERATION_CASES = {
  // out project 1..1 Project: the current project.
  async fhir_op_current_project({ ok }) {
    const { resource } = output(
      await ok("fhir_op_current_project", {}),
      "project",
    );
    assert.equal(resource.resourceType, "Project");
    assert.equal(resource.id, PROJECT);
  },

  // Seven 1..1 uri outputs; mcp-endpoint is this endpoint.
  async fhir_op_endpoints({ ok }) {
    const result = await ok("fhir_op_endpoints", {});
    for (const name of [
      "fhir-r4-base-url",
      "fhir-r4-capabilities-url",
      "oidc-discovery-url",
      "oidc-token-endpoint",
      "oidc-authorize-endpoint",
      "oidc-jwks-endpoint",
    ]) {
      assert.ok(URL.canParse(output(result, name).valueUri), name);
    }
    assert.equal(output(result, "mcp-endpoint").valueUri, mcpUrl);
  },

  async fhir_op_tenant_branding({ ok }) {
    assert.equal(
      (await ok("fhir_op_tenant_branding", {})).resourceType,
      "Parameters",
    );
  },

  // Authorization, Error Handling: 403 for "insufficient permissions"; the
  // suite's client is not the tenant owner.
  async fhir_op_tenant_customization({ refused }) {
    await refused("fhir_op_tenant_customization", { name: "conformance" });
  },

  async fhir_op_indexing_errors({ ok }) {
    assert.equal(
      (await ok("fhir_op_indexing_errors", {})).resourceType,
      "Parameters",
    );
  },

  async fhir_op_refresh_tokens({ ok }) {
    assert.equal(
      (await ok("fhir_op_refresh_tokens", {})).resourceType,
      "Parameters",
    );
  },

  async fhir_op_scopes({ ok }) {
    assert.equal((await ok("fhir_op_scopes", {})).resourceType, "Parameters");
  },

  // in client_id 1..1; out return 1..1 OperationOutcome.
  async fhir_op_delete_scope({ ok, fails }) {
    const result = await ok("fhir_op_delete_scope", {
      client_id: `conformance-${randomUUID()}`,
    });
    assert.equal(result.resourceType, "OperationOutcome");
    await fails("fhir_op_delete_scope", {});
  },

  // in client_id 1..1. Calling without user_agent (0..1) is a known gap in
  // tools.test.mjs.
  async fhir_op_delete_refresh_token({ fails }) {
    await fails("fhir_op_delete_refresh_token", { user_agent: "conformance" });
  },

  // Instance level on IdentityProvider.
  async fhir_op_registration_info({ ok, fails, create }) {
    const provider = await create({
      resourceType: "IdentityProvider",
      name: "conformance",
      status: "active",
      accessType: "oidc",
    });
    const result = await ok("fhir_op_registration_info", { id: provider.id });
    assert.equal(result.resourceType, "Parameters");
    await fails("fhir_op_registration_info", {});
  },

  // in request 1..1 Bundle; out return 1..1 OperationOutcome.
  async fhir_op_evaluate_policy({ ok, fails, create }) {
    const policy = await create({
      resourceType: "AccessPolicyV2",
      name: "conformance",
      engine: "full-access",
    });
    const request = {
      resourceType: "Bundle",
      type: "batch",
      entry: [{ request: { method: "GET", url: "Patient" } }],
    };
    const result = await ok("fhir_op_evaluate_policy", {
      id: policy.id,
      request,
    });
    assert.equal(result.resourceType, "OperationOutcome");
    await fails("fhir_op_evaluate_policy", { id: policy.id });
  },

  // in hl7v2 1..1; MSH-2 holds the encoding characters.
  async fhir_op_hl7v2_parse({ ok, fails }) {
    const message = [
      "MSH|^~\\&|SEND|FAC|RECV|FAC|20240101120000||ADT^A01|MSG0001|P|2.5",
      "PID|1||12345^^^MRN||Doe^John||19800101|M",
    ].join("\r");
    const { resource } = output(
      await ok("fhir_op_hl7v2_parse", { hl7v2: message }),
      "hl7v2",
    );
    const [msh, pid] = resource.segments;
    assert.deepEqual([msh.id, pid.id], ["MSH", "PID"]);
    assert.equal(msh.fields[0].value.value.value, "^~\\&");
    await fails("fhir_op_hl7v2_parse", {});
  },

  // ValueSet $expand returns the expanded ValueSet, also for the shared
  // ValueSets read at the instance level.
  async fhir_op_expand({ ok, fails }) {
    for (const args of [
      { url: GENDER_VALUE_SET },
      { id: "administrative-gender" },
    ]) {
      const valueSet = await ok("fhir_op_expand", args);
      assert.equal(valueSet.resourceType, "ValueSet");
      assert.ok(valueSet.expansion.contains.some((c) => c.code === "female"));
    }
    await fails("fhir_op_expand", { url: "http://example.org/no-such-vs" });
  },

  // ValueSet $validate-code: out result 1..1 boolean.
  async fhir_op_validate_code({ ok }) {
    for (const [code, valid] of [
      ["female", true],
      ["not-a-gender", false],
    ]) {
      const result = await ok("fhir_op_validate_code", {
        url: GENDER_VALUE_SET,
        system: GENDER_SYSTEM,
        code,
      });
      assert.equal(output(result, "result").valueBoolean, valid);
    }
  },

  // Patient $everything returns a searchset Bundle of the patient's
  // compartment.
  async fhir_op_everything({ ok, fails, create }) {
    const patient = await create({ resourceType: "Patient" });
    const observation = await create({
      resourceType: "Observation",
      status: "final",
      code: { text: "conformance" },
      subject: { reference: `Patient/${patient.id}` },
    });
    await eventually(async () => {
      const bundle = await ok("fhir_op_everything", { id: patient.id });
      assert.equal(bundle.type, "searchset");
      const ids = bundle.entry.map(
        (e) => `${e.resource.resourceType}/${e.resource.id}`,
      );
      assert.ok(ids.includes(`Patient/${patient.id}`));
      assert.ok(ids.includes(`Observation/${observation.id}`));
    });
    await fails("fhir_op_everything", { id: randomUUID() });
  },

  // "Only a signed-in user can start a launch."
  async fhir_op_launch({ refused, create }) {
    const patient = await create({ resourceType: "Patient" });
    await refused("fhir_op_launch", {
      id: "testing",
      patient: { reference: `Patient/${patient.id}` },
    });
  },

  // SQL on FHIR $run: the rows as a Binary in the requested _format.
  async fhir_op_viewdefinition_run({ ok, fails }) {
    const result = await ok("fhir_op_viewdefinition_run", {
      viewResource: {
        resourceType: "ViewDefinition",
        resource: "Patient",
        status: "active",
        select: [{ column: [{ name: "family", path: "name.family" }] }],
      },
      _format: "json",
      resource: [
        { resourceType: "Patient", id: "a", name: [{ family: "Alpha" }] },
        { resourceType: "Patient", id: "b", name: [{ family: "Beta" }] },
      ],
    });
    assert.equal(result.resourceType, "Binary");
    assert.deepEqual(
      JSON.parse(Buffer.from(result.data, "base64").toString()),
      [{ family: "Alpha" }, { family: "Beta" }],
    );
    await fails("fhir_op_viewdefinition_run", {
      viewResource: { resourceType: "ViewDefinition" },
    });
  },
};
