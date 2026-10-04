import { readFileSync } from "node:fs";
import { join } from "node:path";
import type { LoadContext, Plugin } from "@docusaurus/types";

// This runs in Node.js - Don't use client-side code here (browser APIs, JSX...)

/**
 * The figures the homepage quotes, read at build time from the generated
 * reports the conformance and MCP pages render in full. A number on the
 * homepage therefore cannot drift from the report behind it.
 *
 * Only the totals are published as global data: the conformance report itself
 * is several megabytes and has no business in the homepage bundle.
 */
export type HomepageFacts = {
  conformance: {
    /** When the conformance report was generated (ISO 8601). */
    generatedAt: string;
    /** Search backends every check ran against. */
    backends: string[];
    resourceTypes: number;
    /** Resource types that pass every check on every backend. */
    passing: number;
    /** Resource types whose only failures are known, published ones. */
    knownIssues: string[];
    /** Resource types with a failure that is not a known one, or untested. */
    failing: string[];
    searchParameters: number;
    checks: number;
    /** Checks that pass on every backend. */
    checksPassing: number;
  };
  /** Tools the MCP endpoint lists. */
  mcpTools: number;
};

export const PLUGIN_NAME = "homepage-facts";

const SUPPORT_PATH = "static/test-reports/support.json";
const MCP_TOOLS_PATH = "static/mcp/tools.json";

type Result = "pass" | "warn" | "fail" | "not-run";

type SupportReport = {
  generatedAt: string;
  backends: string[];
  resources: {
    resourceType: string;
    groups: {
      group: string;
      searchParameterUrl?: string;
      assertions: { results: Record<string, Result> }[];
      results: Record<string, Result>;
    }[];
  }[];
};

/**
 * The worst result of a set. Mirrors combine() in
 * src/components/TestCoverage/data.ts, which the conformance page uses.
 */
function combine(results: Result[]): Result {
  if (results.includes("fail")) return "fail";
  if (results.length === 0 || results.includes("not-run")) return "not-run";
  return results.includes("warn") ? "warn" : "pass";
}

function readJson<T>(siteDir: string, path: string): T {
  return JSON.parse(readFileSync(join(siteDir, path), "utf8")) as T;
}

export function loadHomepageFacts(siteDir: string): HomepageFacts {
  const report = readJson<SupportReport>(siteDir, SUPPORT_PATH);
  const tools = readJson<unknown[]>(siteDir, MCP_TOOLS_PATH);
  const { backends } = report;

  // A resource type's result is its worst group on its worst backend.
  const byResource = report.resources.map((resource) => ({
    resourceType: resource.resourceType,
    result: combine(
      resource.groups.flatMap((group) =>
        backends.map((backend) => group.results[backend]),
      ),
    ),
  }));
  const named = (...results: Result[]) =>
    byResource
      .filter((resource) => results.includes(resource.result))
      .map((resource) => resource.resourceType);

  const assertions = report.resources.flatMap((resource) =>
    resource.groups.flatMap((group) => group.assertions),
  );
  const searchParameters = new Set(
    report.resources.flatMap((resource) =>
      resource.groups
        .filter((group) => group.group === "search")
        .map((group) => `${resource.resourceType}|${group.searchParameterUrl}`),
    ),
  );

  return {
    conformance: {
      generatedAt: report.generatedAt,
      backends,
      resourceTypes: byResource.length,
      passing: named("pass").length,
      knownIssues: named("warn"),
      failing: named("fail", "not-run"),
      searchParameters: searchParameters.size,
      checks: assertions.length,
      checksPassing: assertions.filter((assertion) =>
        backends.every((backend) => assertion.results[backend] === "pass"),
      ).length,
    },
    mcpTools: tools.length,
  };
}

export default function homepageFactsPlugin(
  context: LoadContext,
): Plugin<HomepageFacts> {
  return {
    name: PLUGIN_NAME,
    getPathsToWatch() {
      return [SUPPORT_PATH, MCP_TOOLS_PATH].map((path) =>
        join(context.siteDir, path),
      );
    },
    loadContent() {
      return loadHomepageFacts(context.siteDir);
    },
    contentLoaded({ content, actions }) {
      actions.setGlobalData(content);
    },
  };
}
