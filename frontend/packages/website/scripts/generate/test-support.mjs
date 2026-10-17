// The conformance report: what each core TestScript checked and what the two
// search backends' TestReports say about it, merged into the compact file the
// test coverage page downloads and the FHIR model pages embed.
//
//   static/test-reports/support.json
//
// Options (all optional):
//   --testscripts <dir>         default backend/testscripts/core
//   --postgres <bundle>         a TestReport Bundle, default backend/testreports/postgres.json
//   --elasticsearch <bundle>    default backend/testreports/elasticsearch.json
//   --output <file>             default static/test-reports/support.json

import fs from "node:fs";
import path from "node:path";

import {
  ARTIFACTS,
  BACKEND,
  SUPPORT,
  combine,
  jsonFiles,
  readJson,
  readResources,
  relative,
  writeFile,
} from "./lib.mjs";

const CATEGORY_URL =
  "https://haste.health/fhir/StructureDefinition/testscript-category";
// Set by `testscript run --allow-failure` on a TestReport whose failure is known.
const EXPECTED_FAILURE_URL =
  "https://haste.health/fhir/StructureDefinition/testreport-expected-failure";

const HL7_SEARCH_PARAMETERS = path.join(
  ARTIFACTS,
  "hl7-core/definitions/hl7/search-parameters.json",
);
const HASTE_SEARCH_PARAMETERS = path.join(
  ARTIFACTS,
  "hastehealth-core/definitions/haste-health/search_parameter",
);

function extensionValue(extension, url) {
  const part = extension?.extension?.find((item) => item.url === url);
  if (!part) return undefined;
  return Object.entries(part).find(([key]) => key.startsWith("value"))?.[1];
}

function category(action) {
  const source = action.operation ?? action.assert;
  const extension = source?.extension?.find(
    (item) => item.url === CATEGORY_URL,
  );
  if (!extension) return undefined;
  return {
    group: extensionValue(extension, "group"),
    searchParameterUrl: extensionValue(extension, "searchParameterUrl"),
    searchParameterCode: extensionValue(extension, "searchParameterCode"),
    variant: extensionValue(extension, "variant"),
  };
}

function reportsById(file) {
  return new Map(
    readResources(file)
      .filter((resource) => resource?.resourceType === "TestReport")
      .map((report) => [report.id, report]),
  );
}

function isExpectedFailure(report) {
  return (report?.extension ?? []).some(
    (extension) =>
      extension.url === EXPECTED_FAILURE_URL && extension.valueBoolean === true,
  );
}

/**
 * A TestReport action result as the website's Result: pass | warn | fail |
 * not-run. Failures in a report marked as an expected failure become `warn`.
 * (The runner's own `warning`, a failed operation whose assertions decide the
 * outcome, counts as a pass.)
 */
function toResult(result, report) {
  if (result === "fail" || result === "error") {
    return isExpectedFailure(report) ? "warn" : "fail";
  }
  if (result === "pass" || result === "warning") return "pass";
  return "not-run";
}

function actionResult(report, testIndex, actionIndex, kind) {
  return toResult(
    report?.test?.[testIndex]?.action?.[actionIndex]?.[kind]?.result,
    report,
  );
}

/**
 * Canonical URL -> search parameter type, from the HL7 R4 bundle and Haste
 * Health's own parameters (User, AccessPolicyV2...).
 */
function searchParameterTypes() {
  const resources = [
    HL7_SEARCH_PARAMETERS,
    ...jsonFiles(HASTE_SEARCH_PARAMETERS),
  ].flatMap(readResources);
  return new Map(
    resources
      .filter((resource) => resource?.resourceType === "SearchParameter")
      .map((parameter) => [parameter.url, parameter.type]),
  );
}

/** The groups of one TestScript: its actions by category, with each backend's results. */
function resourceSupport(testScript, reports, parameterTypes) {
  const groups = new Map();

  for (const [testIndex, test] of (testScript.test ?? []).entries()) {
    for (const [actionIndex, action] of (test.action ?? []).entries()) {
      const metadata = category(action);
      if (!metadata?.group) continue;
      const kind = action.operation ? "operation" : "assert";
      const key = [
        metadata.group,
        metadata.searchParameterUrl,
        metadata.variant,
      ].join("|");
      const entry = groups.get(key) ?? {
        group: metadata.group,
        searchParameterUrl: metadata.searchParameterUrl,
        searchParameterCode: metadata.searchParameterCode,
        searchParameterType: parameterTypes.get(metadata.searchParameterUrl),
        variant: metadata.variant,
        assertions: [],
        results: Object.fromEntries(
          Object.keys(reports).map((backend) => [backend, []]),
        ),
      };

      if (kind === "assert") {
        entry.assertions.push({
          label: action.assert.label ?? test.name,
          expression: action.assert.expression,
          operator: action.assert.operator,
          expected: action.assert.value,
          results: Object.fromEntries(
            Object.entries(reports).map(([backend, report]) => [
              backend,
              actionResult(report, testIndex, actionIndex, kind),
            ]),
          ),
        });
      }
      for (const [backend, report] of Object.entries(reports)) {
        entry.results[backend].push(
          actionResult(report, testIndex, actionIndex, kind),
        );
      }
      groups.set(key, entry);
    }
  }

  return {
    testScriptUrl: testScript.url,
    overall: Object.fromEntries(
      Object.entries(reports).map(([backend, report]) => [
        backend,
        toResult(report?.result, report),
      ]),
    ),
    groups: [...groups.values()].map((group) => ({
      ...group,
      results: Object.fromEntries(
        Object.entries(group.results).map(([backend, results]) => [
          backend,
          combine(results),
        ]),
      ),
    })),
  };
}

export async function run({ args }) {
  const testscripts =
    args.testscripts ?? path.join(BACKEND, "testscripts/core");
  const output = args.output ?? SUPPORT;
  const bundles = {
    postgres: args.postgres ?? path.join(BACKEND, "testreports/postgres.json"),
    elasticsearch:
      args.elasticsearch ??
      path.join(BACKEND, "testreports/elasticsearch.json"),
  };
  for (const file of [testscripts, ...Object.values(bundles)]) {
    if (!fs.existsSync(file)) throw new Error(`${file} does not exist.`);
  }

  const backendReports = Object.fromEntries(
    Object.entries(bundles).map(([backend, file]) => [
      backend,
      reportsById(file),
    ]),
  );
  const parameterTypes = searchParameterTypes();

  const files = fs
    .readdirSync(testscripts)
    .filter((file) => file.endsWith(".testscript.json"))
    .sort();
  const resources = files.map((file) => {
    const testScript = readJson(path.join(testscripts, file));
    const reports = Object.fromEntries(
      Object.entries(backendReports).map(([backend, reports]) => [
        backend,
        reports.get(testScript.id),
      ]),
    );
    return {
      resourceType: file.replace(".testscript.json", ""),
      ...resourceSupport(testScript, reports, parameterTypes),
    };
  });

  // Compact: the coverage page downloads this file at runtime.
  const report = {
    generatedAt: new Date().toISOString(),
    backends: Object.keys(bundles),
    resources,
  };
  writeFile(output, `${JSON.stringify(report)}\n`);
  return `Wrote ${relative(output)}: ${resources.length} resource types from ${files.length} TestScripts`;
}
