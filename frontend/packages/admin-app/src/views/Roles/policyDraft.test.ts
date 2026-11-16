import assert from "node:assert/strict";
import { test } from "node:test";

import {
  applyPreset,
  buildAssignment,
  buildPolicy,
  buildRules,
  configuredTypes,
  describePermission,
  emptyPermission,
  interactionsFor,
  targetExpression,
  type PermissionMap,
} from "./policyDraft.ts";

test("read covers read, vread and search", () => {
  assert.deepEqual(interactionsFor({ ...emptyPermission(), read: true }), [
    "read",
    "vread",
    "search",
  ]);
});

test("update covers update and patch", () => {
  assert.deepEqual(interactionsFor({ ...emptyPermission(), update: true }), [
    "update",
    "patch",
  ]);
});

test("all permissions use the engine's interaction order", () => {
  assert.deepEqual(
    interactionsFor({
      create: true,
      read: true,
      update: true,
      delete: true,
    }),
    [
      "create",
      "read",
      "vread",
      "search",
      "update",
      "patch",
      "delete",
    ],
  );
});

test("target expression uses the engine's request fields", () => {
  assert.equal(
    targetExpression("Observation", {
      create: true,
      read: true,
      update: true,
      delete: true,
    }),
    "%request.resource_type = 'Observation' and %request.type in ('create' | 'read' | 'vread' | 'search' | 'update' | 'patch' | 'delete')",
  );
});

test("a read-only target omits write interactions", () => {
  assert.equal(
    targetExpression("Patient", { ...emptyPermission(), read: true }),
    "%request.resource_type = 'Patient' and %request.type in ('read' | 'vread' | 'search')",
  );
});

test("an empty permission has no target expression", () => {
  assert.equal(targetExpression("Patient", emptyPermission()), undefined);
});

test("rules are sorted, skip empty rows and always carry a condition", () => {
  const rules = buildRules({
    Patient: { ...emptyPermission(), read: true },
    Observation: { create: true, read: true, update: true, delete: true },
    Encounter: emptyPermission(),
  });

  assert.equal(rules.length, 2);
  assert.equal(rules[0]!.name, "Observation permissions");
  assert.equal(rules[1]!.name, "Patient permissions");
  assert.equal(rules[0]!.effect, "permit");
  assert.equal(rules[0]!.condition!.expression.expression, "true");
  assert.match(
    rules[0]!.target!.expression.expression as string,
    /%request\.resource_type = 'Observation'/,
  );
});

test("the policy uses the rule-engine code and lets the server assign the id", () => {
  const policy = buildPolicy({
    name: "  Clinician access  ",
    description: "Clinical staff",
    permissions: { Observation: { ...emptyPermission(), read: true } },
  });

  assert.equal(policy.resourceType, "AccessPolicyV2");
  assert.equal(policy.name, "Clinician access");
  assert.equal(policy.description, "Clinical staff");
  assert.equal(policy.engine, "rule-engine");
  assert.equal(policy.id, undefined);
  assert.equal(policy.rule?.length, 1);
});

test("an empty description is omitted", () => {
  const policy = buildPolicy({
    name: "Access",
    description: "   ",
    permissions: {},
  });
  assert.equal("description" in policy, false);
});

test("an assignment points at the policy and the target", () => {
  const assignment = buildAssignment("urn:uuid:policy", {
    reference: "Membership/member-1",
  });
  assert.equal(assignment.accessPolicy.reference, "urn:uuid:policy");
  assert.equal(assignment.link.reference, "Membership/member-1");
});

test("an assignment without a target reference is rejected", () => {
  assert.throws(() => buildAssignment("urn:uuid:policy", {}));
});

test("auditor grants read on every type", () => {
  const permissions = applyPreset("auditor", ["Patient", "Observation"]);
  assert.deepEqual(permissions.Patient, { ...emptyPermission(), read: true });
  assert.deepEqual(permissions.Observation, { ...emptyPermission(), read: true });
});

test("clinician writes clinical types and only reads patient-facing ones", () => {
  const permissions = applyPreset("clinician", [
    "Patient",
    "Observation",
    "Basic",
  ]);
  assert.deepEqual(permissions.Observation, {
    create: true,
    read: true,
    update: true,
    delete: false,
  });
  assert.deepEqual(permissions.Patient, { ...emptyPermission(), read: true });
  assert.deepEqual(permissions.Basic, emptyPermission());
});

test("intake clerk manages registration data and reads the directory", () => {
  const permissions = applyPreset("intake", [
    "Patient",
    "Encounter",
    "Organization",
    "Observation",
  ]);
  assert.deepEqual(permissions.Patient, {
    create: true,
    read: true,
    update: true,
    delete: false,
  });
  assert.deepEqual(permissions.Encounter, {
    create: true,
    read: true,
    update: false,
    delete: false,
  });
  assert.deepEqual(permissions.Organization, {
    ...emptyPermission(),
    read: true,
  });
  assert.deepEqual(permissions.Observation, emptyPermission());
});


test("front desk manages registration and scheduling without clinical writes", () => {
  const permissions = applyPreset("frontDesk", [
    "Patient",
    "Appointment",
    "Practitioner",
    "Observation",
  ]);
  assert.deepEqual(permissions.Patient, {
    create: true,
    read: true,
    update: true,
    delete: false,
  });
  assert.deepEqual(permissions.Appointment, {
    create: true,
    read: true,
    update: true,
    delete: false,
  });
  assert.deepEqual(permissions.Practitioner, {
    ...emptyPermission(),
    read: true,
  });
  assert.deepEqual(permissions.Observation, emptyPermission());
});

test("nurse charts observations and administers medication", () => {
  const permissions = applyPreset("nurse", [
    "Observation",
    "MedicationAdministration",
    "Patient",
    "Basic",
  ]);
  assert.deepEqual(permissions.Observation, {
    create: true,
    read: true,
    update: true,
    delete: false,
  });
  assert.deepEqual(permissions.MedicationAdministration, {
    create: true,
    read: true,
    update: true,
    delete: false,
  });
  assert.deepEqual(permissions.Patient, {
    ...emptyPermission(),
    read: true,
  });
  assert.deepEqual(permissions.Basic, emptyPermission());
});

test("laboratory works specimens and results", () => {
  const permissions = applyPreset("laboratory", [
    "Specimen",
    "Observation",
    "DiagnosticReport",
    "Patient",
    "ServiceRequest",
    "Basic",
  ]);
  assert.deepEqual(permissions.Specimen, {
    create: true,
    read: true,
    update: true,
    delete: false,
  });
  assert.deepEqual(permissions.Observation, {
    create: true,
    read: true,
    update: true,
    delete: false,
  });
  assert.deepEqual(permissions.DiagnosticReport, {
    create: true,
    read: true,
    update: true,
    delete: false,
  });
  assert.deepEqual(permissions.Patient, {
    ...emptyPermission(),
    read: true,
  });
  assert.deepEqual(permissions.ServiceRequest, {
    ...emptyPermission(),
    read: true,
  });
  assert.deepEqual(permissions.Basic, emptyPermission());
});

test("oncology manages treatment and research context", () => {
  const permissions = applyPreset("oncology", [
    "Condition",
    "MedicationRequest",
    "Patient",
    "ResearchStudy",
    "Basic",
  ]);
  assert.deepEqual(permissions.Condition, {
    create: true,
    read: true,
    update: true,
    delete: false,
  });
  assert.deepEqual(permissions.MedicationRequest, {
    create: true,
    read: true,
    update: true,
    delete: false,
  });
  assert.deepEqual(permissions.Patient, {
    ...emptyPermission(),
    read: true,
  });
  assert.deepEqual(permissions.ResearchStudy, {
    ...emptyPermission(),
    read: true,
  });
  assert.deepEqual(permissions.Basic, emptyPermission());
});

test("configured types are sorted and omit empty rows", () => {
  const permissions: PermissionMap = {
    Patient: { ...emptyPermission(), read: true },
    Observation: { ...emptyPermission(), create: true },
    Basic: emptyPermission(),
  };
  assert.deepEqual(configuredTypes(permissions), ["Observation", "Patient"]);
});

test("descriptions use the matrix's words", () => {
  assert.equal(
    describePermission({
      create: true,
      read: true,
      update: false,
      delete: true,
    }),
    "create, read and delete",
  );
  assert.equal(describePermission(emptyPermission()), "no access");
});
