import { readFile, readdir, writeFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const CATEGORY_URL =
  "https://haste.health/fhir/StructureDefinition/testscript-category";
// Set by `testscript run --allow-failure` on a TestReport whose failure is known.
const EXPECTED_FAILURE_URL =
  "https://haste.health/fhir/StructureDefinition/testreport-expected-failure";

function argument(name) {
  const index = process.argv.indexOf(`--${name}`);
  return index === -1 ? undefined : process.argv[index + 1];
}

function extensionValue(extension, url) {
  const part = extension?.extension?.find((item) => item.url === url);
  if (!part) return undefined;
  return Object.entries(part).find(([key]) => key.startsWith("value"))?.[1];
}

function category(action) {
  const source = action.operation ?? action.assert;
  const extension = source?.extension?.find((item) => item.url === CATEGORY_URL);
  if (!extension) return undefined;
  return {
    group: extensionValue(extension, "group"),
    searchParameterUrl: extensionValue(extension, "searchParameterUrl"),
    searchParameterCode: extensionValue(extension, "searchParameterCode"),
    variant: extensionValue(extension, "variant"),
  };
}

function reportsById(bundle) {
  return new Map(
    (bundle.entry ?? [])
      .map((entry) => entry.resource)
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

function overallResult(results) {
  if (results.includes("fail")) return "fail";
  if (results.length === 0 || results.includes("not-run")) return "not-run";
  return results.includes("warn") ? "warn" : "pass";
}

/** The resources in a JSON file: a Bundle's entries, a list, or one. */
async function resourcesIn(file) {
  const json = JSON.parse(await readFile(file, "utf8"));
  if (json.resourceType === "Bundle") {
    return (json.entry ?? []).map((entry) => entry.resource);
  }
  return Array.isArray(json) ? json : [json];
}

/**
 * Canonical URL -> search parameter type, from the HL7 R4 bundle and Haste
 * Health's own parameters (User, AccessPolicyV2...).
 */
async function searchParameterTypes(bundlePath, hasteDirectory) {
  const hasteFiles = (await readdir(hasteDirectory, { recursive: true }))
    .filter((file) => file.endsWith(".json"))
    .map((file) => path.join(hasteDirectory, file));
  const resources = (
    await Promise.all([bundlePath, ...hasteFiles].map(resourcesIn))
  ).flat();
  return new Map(
    resources
      .filter((resource) => resource?.resourceType === "SearchParameter")
      .map((parameter) => [parameter.url, parameter.type]),
  );
}

const testscriptDirectory = argument("testscripts");
const postgresPath = argument("postgres");
const elasticsearchPath = argument("elasticsearch");
const outputPath = argument("output");

if (!testscriptDirectory || !postgresPath || !elasticsearchPath || !outputPath) {
  throw new Error(
    "Usage: node generate-test-support.mjs --testscripts <dir> --postgres <bundle> --elasticsearch <bundle> --output <file>",
  );
}

const artifacts = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  "../../../../artifacts/r4",
);
const searchParametersPath =
  argument("search-parameters") ??
  path.join(artifacts, "hl7-core/definitions/hl7/search-parameters.json");
const hasteSearchParametersPath = path.join(
  artifacts,
  "hastehealth-core/definitions/haste-health/search_parameter",
);

const [postgresBundle, elasticsearchBundle, parameterTypes] = await Promise.all([
  readFile(postgresPath, "utf8").then(JSON.parse),
  readFile(elasticsearchPath, "utf8").then(JSON.parse),
  searchParameterTypes(searchParametersPath, hasteSearchParametersPath),
]);
const backendReports = {
  postgres: reportsById(postgresBundle),
  elasticsearch: reportsById(elasticsearchBundle),
};

const files = (await readdir(testscriptDirectory))
  .filter((file) => file.endsWith(".testscript.json"))
  .sort();
const resources = [];

for (const file of files) {
  const testScript = JSON.parse(
    await readFile(path.join(testscriptDirectory, file), "utf8"),
  );
  const reports = Object.fromEntries(
    Object.entries(backendReports).map(([backend, reports]) => [
      backend,
      reports.get(testScript.id),
    ]),
  );
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
        results: { postgres: [], elasticsearch: [] },
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
      for (const backend of Object.keys(backendReports)) {
        entry.results[backend].push(
          actionResult(reports[backend], testIndex, actionIndex, kind),
        );
      }
      groups.set(key, entry);
    }
  }

  resources.push({
    resourceType: file.replace(".testscript.json", ""),
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
          overallResult(results),
        ]),
      ),
    })),
  });
}

// Compact: the docs page downloads this file at runtime.
await writeFile(
  outputPath,
  `${JSON.stringify({
    generatedAt: new Date().toISOString(),
    backends: ["postgres", "elasticsearch"],
    resources,
  })}\n`,
);
