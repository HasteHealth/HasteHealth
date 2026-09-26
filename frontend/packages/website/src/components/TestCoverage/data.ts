import { useEffect, useState } from "react";

export type Result = "pass" | "fail" | "not-run";
export type Backend = "postgres" | "elasticsearch";
export type Interaction = "create" | "read" | "update" | "patch" | "delete";
export type ParamType =
  | "string"
  | "token"
  | "reference"
  | "date"
  | "number"
  | "quantity"
  | "uri";

export type Assertion = {
  label: string;
  expression?: string;
  operator?: string;
  expected?: string;
  results: Record<Backend, Result>;
};

export type SupportGroup = {
  group: Interaction | "search";
  searchParameterUrl?: string;
  searchParameterCode?: string;
  searchParameterType?: ParamType;
  variant?: string;
  assertions: Assertion[];
  results: Record<Backend, Result>;
};

export type ResourceSupport = {
  resourceType: string;
  testScriptUrl: string;
  overall: Record<Backend, Result>;
  groups: SupportGroup[];
};

export type SupportData = {
  generatedAt: string;
  resources: ResourceSupport[];
};

export const BACKENDS: readonly Backend[] = ["postgres", "elasticsearch"];
export const BACKEND_NAMES: Record<Backend, string> = {
  postgres: "PostgreSQL",
  elasticsearch: "Elasticsearch",
};
export const INTERACTIONS: readonly Interaction[] = [
  "create",
  "read",
  "update",
  "patch",
  "delete",
];

// ---------------------------------------------------------------------------
// Loading: every section on the page shares one request.
// ---------------------------------------------------------------------------

let request: Promise<SupportData> | undefined;

function load(): Promise<SupportData> {
  request ??= fetch("/test-reports/support.json").then((response) => {
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    return response.json() as Promise<SupportData>;
  });
  return request;
}

export type LoadState =
  | { status: "loading" }
  | { status: "error" }
  | { status: "ready"; data: SupportData };

export function useSupportData(): LoadState {
  const [state, setState] = useState<LoadState>({ status: "loading" });
  useEffect(() => {
    let active = true;
    load().then(
      (data) => active && setState({ status: "ready", data }),
      () => {
        request = undefined;
        if (active) setState({ status: "error" });
      },
    );
    return () => {
      active = false;
    };
  }, []);
  return state;
}

// ---------------------------------------------------------------------------
// Derived views
// ---------------------------------------------------------------------------

/** `fail` if anything failed, `pass` if everything passed, else `not-run`. */
export function combine(results: Result[]): Result {
  if (results.includes("fail")) return "fail";
  if (results.length > 0 && results.every((result) => result === "pass")) return "pass";
  return "not-run";
}

export function groupsResult(groups: SupportGroup[], backend: Backend): Result {
  return combine(groups.map((group) => group.results[backend]));
}

/**
 * The search feature a group tests. `missing-true`/`missing-false` are one
 * feature, `:missing`. Groups without a variant are setup steps (such as
 * creating a reference target), not features.
 */
export function featureKey(variant: string | undefined): string | undefined {
  if (!variant) return undefined;
  return variant.startsWith("missing-") ? "missing" : variant;
}

export type Tally = { pass: number; total: number };

function tally(results: Result[]): Tally {
  return {
    pass: results.filter((result) => result === "pass").length,
    total: results.length,
  };
}

export type BackendSummary = {
  resourcesPassing: number;
  checks: Tally;
};

export type Summary = {
  resourceTypes: number;
  searchParameters: number;
  checks: number;
  /** Checks where both backends returned the same result. */
  agreeing: number;
  backends: Record<Backend, BackendSummary>;
};

export function summarize(data: SupportData): Summary {
  const assertions = data.resources.flatMap((resource) =>
    resource.groups.flatMap((group) => group.assertions),
  );
  const parameters = new Set(
    data.resources.flatMap((resource) =>
      resource.groups
        .filter((group) => group.group === "search")
        .map((group) => `${resource.resourceType}|${group.searchParameterUrl}`),
    ),
  );

  const backend = (name: Backend): BackendSummary => ({
    resourcesPassing: data.resources.filter(
      (resource) => groupsResult(resource.groups, name) === "pass",
    ).length,
    checks: tally(assertions.map((assertion) => assertion.results[name])),
  });

  return {
    resourceTypes: data.resources.length,
    searchParameters: parameters.size,
    checks: assertions.length,
    agreeing: assertions.filter(
      (assertion) => assertion.results.postgres === assertion.results.elasticsearch,
    ).length,
    backends: { postgres: backend("postgres"), elasticsearch: backend("elasticsearch") },
  };
}

/** One search feature (e.g. token `:not`) across every parameter of a type. */
export type FeatureCoverage = {
  type: ParamType;
  feature: string;
  /** Distinct resource type + parameter pairs that exercise it. */
  parameters: number;
  results: Record<Backend, Tally>;
};

export function featureCoverage(data: SupportData): FeatureCoverage[] {
  const features = new Map<string, { type: ParamType; feature: string; groups: SupportGroup[][] }>();

  for (const resource of data.resources) {
    const byFeature = new Map<string, SupportGroup[]>();
    for (const group of resource.groups) {
      const feature = featureKey(group.variant);
      if (group.group !== "search" || !group.searchParameterType || !feature) continue;
      const key = [group.searchParameterType, feature, group.searchParameterUrl].join("|");
      byFeature.set(key, [...(byFeature.get(key) ?? []), group]);
    }
    for (const [key, groups] of byFeature) {
      const [type, feature] = key.split("|") as [ParamType, string];
      const featureId = `${type}|${feature}`;
      const entry = features.get(featureId) ?? { type, feature, groups: [] };
      entry.groups.push(groups);
      features.set(featureId, entry);
    }
  }

  return [...features.values()].map(({ type, feature, groups }) => ({
    type,
    feature,
    parameters: groups.length,
    results: Object.fromEntries(
      BACKENDS.map((backend) => [
        backend,
        tally(groups.map((parameterGroups) => groupsResult(parameterGroups, backend))),
      ]),
    ) as Record<Backend, Tally>,
  }));
}

/** Resource types supporting each interaction, per backend. */
export function interactionCoverage(
  data: SupportData,
): { interaction: Interaction; results: Record<Backend, Tally> }[] {
  return INTERACTIONS.map((interaction) => {
    const groups = data.resources
      .map((resource) => resource.groups.filter((group) => group.group === interaction))
      .filter((groups) => groups.length > 0);
    return {
      interaction,
      results: Object.fromEntries(
        BACKENDS.map((backend) => [
          backend,
          tally(groups.map((resourceGroups) => groupsResult(resourceGroups, backend))),
        ]),
      ) as Record<Backend, Tally>,
    };
  });
}

/** A resource's search groups, one entry per parameter. */
export type ParameterCoverage = {
  code: string;
  url?: string;
  type?: ParamType;
  features: { feature: string; groups: SupportGroup[] }[];
  assertions: Assertion[];
  results: Record<Backend, Result>;
};

export function parameterCoverage(resource: ResourceSupport): ParameterCoverage[] {
  const parameters = new Map<string, SupportGroup[]>();
  for (const group of resource.groups) {
    if (group.group !== "search") continue;
    const code = group.searchParameterCode ?? "unknown";
    parameters.set(code, [...(parameters.get(code) ?? []), group]);
  }

  return [...parameters.entries()]
    .sort(([a], [b]) => a.localeCompare(b))
    .map(([code, groups]) => {
      const features = new Map<string, SupportGroup[]>();
      for (const group of groups) {
        const key = featureKey(group.variant);
        if (key) features.set(key, [...(features.get(key) ?? []), group]);
      }
      return {
        code,
        url: groups[0].searchParameterUrl,
        type: groups[0].searchParameterType,
        features: [...features.entries()].map(([feature, featureGroups]) => ({
          feature,
          groups: featureGroups,
        })),
        assertions: groups.flatMap((group) => group.assertions),
        results: Object.fromEntries(
          BACKENDS.map((backend) => [backend, groupsResult(groups, backend)]),
        ) as Record<Backend, Result>,
      };
    });
}

/** Everything that failed on at least one backend, grouped for reading. */
export type Gap = {
  resourceType: string;
  kind: "search" | "interaction";
  /** The search parameter code, or the interaction. */
  subject: string;
  type?: ParamType;
  features: string[];
  backends: Backend[];
};

export function knownGaps(data: SupportData): Gap[] {
  const gaps = new Map<string, Gap>();
  for (const resource of data.resources) {
    for (const group of resource.groups) {
      const failing = BACKENDS.filter((backend) => group.results[backend] === "fail");
      if (failing.length === 0) continue;
      const subject = group.group === "search" ? (group.searchParameterCode ?? "search") : group.group;
      const key = `${resource.resourceType}|${group.group}|${subject}|${failing.join(",")}`;
      const gap = gaps.get(key) ?? {
        resourceType: resource.resourceType,
        kind: group.group === "search" ? "search" : "interaction",
        subject,
        type: group.searchParameterType,
        features: [],
        backends: failing,
      };
      const feature = featureKey(group.variant);
      if (feature && !gap.features.includes(feature)) gap.features.push(feature);
      gaps.set(key, gap);
    }
  }
  return [...gaps.values()];
}
