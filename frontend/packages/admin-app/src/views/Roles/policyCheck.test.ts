import assert from "node:assert/strict";
import { test } from "node:test";

import {
  buildChecks,
  expectedFor,
  runPolicyChecks,
  type PolicyCheckCase,
} from "./policyCheck.ts";
import { emptyPermission, type PermissionMap } from "./policyDraft.ts";

test("read and search share the matrix's read cell", () => {
  const permission = { ...emptyPermission(), read: true };
  assert.equal(expectedFor(permission, "read"), true);
  assert.equal(expectedFor(permission, "search"), true);
  assert.equal(expectedFor(permission, "create"), false);
});

test("buildChecks covers every permission and adds a denied type", () => {
  const permissions: PermissionMap = {
    Patient: { create: false, read: true, update: false, delete: false },
    Observation: { create: true, read: true, update: true, delete: true },
    Basic: emptyPermission(),
  };

  const checks = buildChecks({
    permissions,
    allResourceTypes: ["Patient", "Observation", "Basic"],
    probeId: "role-check",
  });

  const observation = checks.filter((c) => c.resourceType === "Observation");
  assert.deepEqual(
    observation.map((c) => c.interaction),
    ["create", "read", "search", "update", "delete"],
  );
  assert.deepEqual(
    observation.map((c) => c.expected),
    [true, true, true, true, true],
  );

  const patient = checks.filter((c) => c.resourceType === "Patient");
  assert.deepEqual(
    patient.map((c) => c.interaction),
    ["create", "read", "search", "update", "delete"],
  );
  assert.deepEqual(
    patient.map((c) => c.expected),
    [false, true, true, false, false],
  );

  const denied = checks.find((c) => c.resourceType === "Basic");
  assert.equal(denied?.interaction, "read");
  assert.equal(denied?.expected, false);
});

test("create and update use a valid probe body", () => {
  const checks = buildChecks({
    permissions: {
      Observation: { create: true, read: true, update: true, delete: true },
    },
    allResourceTypes: ["Observation"],
    probeId: "role-check",
  });

  const create = checks.find((c) => c.interaction === "create");
  assert.equal(create?.entry.request?.method, "POST");
  assert.equal(create?.entry.request?.url, "Observation");
  assert.equal(create?.entry.resource?.resourceType, "Basic");

  const update = checks.find((c) => c.interaction === "update");
  assert.equal(update?.entry.request?.method, "PUT");
  assert.equal(update?.entry.request?.url, "Observation/role-check");
  assert.equal(update?.entry.resource?.id, "role-check");
});

test("read, search and delete use the target type's URL", () => {
  const checks = buildChecks({
    permissions: {
      Observation: { create: true, read: true, update: true, delete: true },
    },
    allResourceTypes: ["Observation"],
    probeId: "role-check",
  });

  assert.equal(
    checks.find((c) => c.interaction === "read")?.entry.request?.url,
    "Observation/role-check",
  );
  assert.equal(
    checks.find((c) => c.interaction === "search")?.entry.request?.url,
    "Observation?_count=1",
  );
  assert.equal(
    checks.find((c) => c.interaction === "delete")?.entry.request?.method,
    "DELETE",
  );
});

function check(
  expected: boolean,
  interaction: PolicyCheckCase["interaction"] = "read",
): PolicyCheckCase {
  const method =
    interaction === "create"
      ? "POST"
      : interaction === "update"
        ? "PUT"
        : interaction === "delete"
          ? "DELETE"
          : "GET";
  return {
    resourceType: "Observation",
    interaction,
    expected,
    entry: {
      request: { method, url: "Observation/role-check" },
    } as PolicyCheckCase["entry"],
  };
}

test("a decision matching the matrix passes", async () => {
  const results = await runPolicyChecks({
    checks: [check(true), check(false, "create")],
    evaluate: async (entry) =>
      entry.request?.method === "GET"
        ? "Policy approved user access."
        : "Policy denied user access.",
  });

  assert.deepEqual(
    results.map((r) => r.status),
    ["pass", "pass"],
  );
  assert.deepEqual(
    results.map((r) => r.actual),
    [true, false],
  );
});

test("a decision contradicting the matrix fails", async () => {
  const results = await runPolicyChecks({
    checks: [check(true)],
    evaluate: async () => "Policy denied user access.",
  });

  assert.equal(results[0]!.status, "fail");
  assert.equal(results[0]!.actual, false);
});

test("an incomplete fixture is skipped, not failed", async () => {
  const results = await runPolicyChecks({
    checks: [check(true, "create")],
    evaluate: async () => {
      throw new Error("Failed to deserialize resource: missing field `status`");
    },
  });

  assert.equal(results[0]!.status, "skipped");
  assert.equal(results[0]!.actual, undefined);
});

test("other request errors fail", async () => {
  const results = await runPolicyChecks({
    checks: [check(true)],
    evaluate: async () => {
      throw new Error("Network unavailable");
    },
  });

  assert.equal(results[0]!.status, "fail");
});

test("onResult is called for each case", async () => {
  const seen: number[] = [];
  await runPolicyChecks({
    checks: [check(true), check(false, "delete")],
    evaluate: async () => "Policy approved user access.",
    onResult: (_result, index) => seen.push(index),
  });

  assert.deepEqual(seen, [0, 1]);
});
