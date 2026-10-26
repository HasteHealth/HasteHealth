import { useEffect, useState } from "react";

import type { Backend, Result } from "../TestCoverage/data";

export type { Backend, Result };

/** A suite page's report, written by scripts/generate/test-suites.mjs. */
export type SuiteAssertion = {
  label: string;
  expression?: string;
  results: Partial<Record<Backend, Result>>;
};

export type SuiteTest = {
  name: string;
  description?: string;
  assertions: SuiteAssertion[];
};

export type SuiteScript = {
  id: string;
  title: string;
  description?: string;
  /** Repository-relative path of the TestScript. */
  path: string;
  subgroup?: string;
  /**
   * Why the script fails today. CI skips it, or, when the report's
   * `knownFailuresRun` is set, runs it and expects it to fail.
   */
  knownFailure?: string;
  result: Partial<Record<Backend, Result>>;
  tests: SuiteTest[];
};

export type SuiteGroup = {
  id: string;
  title: string;
  scripts: SuiteScript[];
};

export type SuiteReport = {
  id: string;
  title: string;
  generatedAt: string;
  backends: Backend[];
  /** CI runs the known failures and expects them to fail, instead of skipping them. */
  knownFailuresRun?: boolean;
  groups: SuiteGroup[];
};

export type SuiteId = "us-core" | "g10" | "access-control" | "sql-on-fhir";

export const REPOSITORY =
  "https://github.com/HasteHealth/HasteHealth/blob/main";

export const ISSUES = "https://github.com/HasteHealth/HasteHealth/issues";

export const BACKEND_SHORT: Record<Backend, string> = {
  postgres: "PG",
  elasticsearch: "ES",
};

// ---------------------------------------------------------------------------
// Loading: one request per suite, shared by every section on its page.
// ---------------------------------------------------------------------------

const requests = new Map<SuiteId, Promise<SuiteReport>>();

function load(suite: SuiteId): Promise<SuiteReport> {
  let request = requests.get(suite);
  if (!request) {
    request = fetch(`/test-reports/suites/${suite}.json`).then((response) => {
      if (!response.ok) throw new Error(`HTTP ${response.status}`);
      return response.json() as Promise<SuiteReport>;
    });
    requests.set(suite, request);
  }
  return request;
}

export type LoadState =
  | { status: "loading" }
  | { status: "error" }
  | { status: "ready"; data: SuiteReport };

export function useSuite(suite: SuiteId): LoadState {
  const [state, setState] = useState<LoadState>({ status: "loading" });
  useEffect(() => {
    let cancelled = false;
    load(suite).then(
      (data) => !cancelled && setState({ status: "ready", data }),
      () => !cancelled && setState({ status: "error" }),
    );
    return () => {
      cancelled = true;
    };
  }, [suite]);
  return state;
}

// ---------------------------------------------------------------------------
// Derived values
// ---------------------------------------------------------------------------

export function scriptsOf(report: SuiteReport): SuiteScript[] {
  return report.groups.flatMap((group) => group.scripts);
}

/** A script passes when it passed on every backend the suite ran on. */
export function passesEverywhere(
  results: Partial<Record<Backend, Result>>,
  backends: Backend[],
): boolean {
  return backends.every((backend) => results[backend] === "pass");
}

/** The worst result across the suite's backends. */
export function worst(
  results: Partial<Record<Backend, Result>>,
  backends: Backend[],
): Result {
  const values = backends.map((backend) => results[backend] ?? "not-run");
  if (values.includes("fail")) return "fail";
  if (values.includes("warn")) return "warn";
  if (values.length === 0 || values.includes("not-run")) return "not-run";
  return "pass";
}

export type SuiteTotals = {
  scripts: number;
  scriptsPassing: number;
  knownGaps: number;
  failing: number;
  checks: number;
  checksPassing: number;
};

export function totalsOf(report: SuiteReport): SuiteTotals {
  const scripts = scriptsOf(report);
  const assertions = scripts.flatMap((script) =>
    script.tests.flatMap((test) => test.assertions),
  );
  return {
    scripts: scripts.length,
    scriptsPassing: scripts.filter((script) =>
      passesEverywhere(script.result, report.backends),
    ).length,
    knownGaps: scripts.filter((script) => script.knownFailure).length,
    failing: scripts.filter(
      (script) =>
        !script.knownFailure &&
        worst(script.result, report.backends) === "fail",
    ).length,
    checks: assertions.length,
    checksPassing: assertions.filter((assertion) =>
      passesEverywhere(assertion.results, report.backends),
    ).length,
  };
}

/** Known gaps by the reason given for them, in the order they first appear. */
export function gapsByReason(
  report: SuiteReport,
): { reason: string; scripts: SuiteScript[] }[] {
  const byReason = new Map<string, SuiteScript[]>();
  for (const script of scriptsOf(report)) {
    if (!script.knownFailure) continue;
    const list = byReason.get(script.knownFailure) ?? [];
    list.push(script);
    byReason.set(script.knownFailure, list);
  }
  return [...byReason.entries()].map(([reason, scripts]) => ({
    reason,
    scripts,
  }));
}
