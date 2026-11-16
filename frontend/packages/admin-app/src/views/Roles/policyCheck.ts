/**
 * Turns the matrix into the requests the policy should permit or deny, and
 * runs them through the server's $evaluate-policy operation.
 *
 * The evaluator is injected so the expected/actual comparison can be tested
 * without a server.
 */
import type {
  BundleEntry,
  Resource,
  code,
  id,
  uri,
} from "@haste-health/fhir-types/r4/types";

import {
  configuredTypes,
  type Permission,
  type PermissionMap,
} from "./policyDraft.ts";

export type CheckInteraction =
  | "create"
  | "read"
  | "search"
  | "update"
  | "delete";

export const CHECK_INTERACTIONS: CheckInteraction[] = [
  "create",
  "read",
  "search",
  "update",
  "delete",
];

export type PolicyCheckCase = {
  resourceType: string;
  interaction: CheckInteraction;
  expected: boolean;
  entry: BundleEntry;
};

export type CheckStatus = "pass" | "fail" | "skipped";

export type CheckResult = PolicyCheckCase & {
  actual: boolean | undefined;
  message: string;
  status: CheckStatus;
};

/**
 * A small, valid resource for create/update probes.
 *
 * $evaluate-policy only decides the request; it does not store the body. The
 * policy sees the resource type from the request URL, so one valid body can
 * probe every type without needing a valid fixture for each one.
 */
const PROBE_RESOURCE = {
  resourceType: "Basic",
  code: { text: "policy-check" },
} as Resource;

/** Read covers search in the engine, so both map to the matrix's Read cell. */
export function expectedFor(
  permission: Permission,
  interaction: CheckInteraction,
): boolean {
  switch (interaction) {
    case "read":
    case "search":
      return permission.read;
    case "create":
      return permission.create;
    case "update":
      return permission.update;
    case "delete":
      return permission.delete;
  }
}

function entryFor(
  resourceType: string,
  interaction: CheckInteraction,
  probeId: string,
): BundleEntry {
  switch (interaction) {
    case "read":
      return {
        request: {
          method: "GET" as code,
          url: `${resourceType}/${probeId}` as uri,
        },
      };
    case "search":
      return {
        request: {
          method: "GET" as code,
          url: `${resourceType}?_count=1` as uri,
        },
      };
    case "delete":
      return {
        request: {
          method: "DELETE" as code,
          url: `${resourceType}/${probeId}` as uri,
        },
      };
    case "create":
      return {
        request: { method: "POST" as code, url: resourceType as uri },
        resource: { ...PROBE_RESOURCE },
      };
    case "update":
      return {
        request: {
          method: "PUT" as code,
          url: `${resourceType}/${probeId}` as uri,
        },
        resource: { ...PROBE_RESOURCE, id: probeId as id },
      };
  }
}

/**
 * Every cell worth checking, plus one read on a type the matrix leaves
 * unconfigured so a denial is exercised even when every selected type grants
 * every permission.
 */
export function buildChecks(args: {
  permissions: PermissionMap;
  allResourceTypes: string[];
  probeId: string;
}): PolicyCheckCase[] {
  const configured = configuredTypes(args.permissions);
  const configuredSet = new Set(configured);
  const checks: PolicyCheckCase[] = [];

  for (const resourceType of configured) {
    const permission = args.permissions[resourceType]!;
    for (const interaction of CHECK_INTERACTIONS) {
      checks.push({
        resourceType,
        interaction,
        expected: expectedFor(permission, interaction),
        entry: entryFor(resourceType, interaction, args.probeId),
      });
    }
  }

  const denied = args.allResourceTypes.find((type) => !configuredSet.has(type));
  if (denied) {
    checks.push({
      resourceType: denied,
      interaction: "read",
      expected: false,
      entry: entryFor(denied, "read", args.probeId),
    });
  }

  return checks;
}

/**
 * Runs cases in order. A case whose request fails before the policy is
 * evaluated is `skipped`, so a fixture problem is not reported as a policy
 * failure.
 */
export async function runPolicyChecks(args: {
  checks: PolicyCheckCase[];
  evaluate: (entry: BundleEntry) => Promise<string>;
  onResult?: (result: CheckResult, index: number) => void;
}): Promise<CheckResult[]> {
  const results: CheckResult[] = [];

  for (let index = 0; index < args.checks.length; index += 1) {
    const check = args.checks[index]!;
    let result: CheckResult;
    try {
      const message = await args.evaluate(check.entry);
      const actual = message.includes("approved");
      result = {
        ...check,
        actual,
        message,
        status: actual === check.expected ? "pass" : "fail",
      };
    } catch (error) {
      const message =
        error instanceof Error ? error.message : `Request failed: ${String(error)}`;
      result = {
        ...check,
        actual: undefined,
        message,
        status: message.includes("Failed to deserialize")
          ? "skipped"
          : "fail",
      };
    }

    results.push(result);
    args.onResult?.(result, index);
  }

  return results;
}
