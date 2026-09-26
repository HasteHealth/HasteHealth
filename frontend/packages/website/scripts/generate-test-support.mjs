import { readFile, readdir, writeFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const CATEGORY_URL =
  "https://haste.health/fhir/StructureDefinition/testscript-category";

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

function actionResult(report, testIndex, actionIndex, kind) {
  return report?.test?.[testIndex]?.action?.[actionIndex]?.[kind]?.result ??
    "not-run";
}

function overallResult(results) {
  if (results.some((result) => result === "fail" || result === "error")) {
    return "fail";
  }
  if (
    results.length > 0 &&
    results.every((result) => result === "pass" || result === "warning")
  ) {
    return "pass";
  }
  return "not-run";
}

/** Canonical URL -> search parameter type, from the HL7 R4 definitions. */
async function searchParameterTypes(bundlePath) {
  const bundle = JSON.parse(await readFile(bundlePath, "utf8"));
  return new Map(
    (bundle.entry ?? [])
      .map((entry) => entry.resource)
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

const searchParametersPath =
  argument("search-parameters") ??
  path.resolve(
    path.dirname(fileURLToPath(import.meta.url)),
    "../../../../artifacts/r4/hl7-core/definitions/hl7/search-parameters.json",
  );

const [postgresBundle, elasticsearchBundle, parameterTypes] = await Promise.all([
  readFile(postgresPath, "utf8").then(JSON.parse),
  readFile(elasticsearchPath, "utf8").then(JSON.parse),
  searchParameterTypes(searchParametersPath),
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
        report?.result ?? "not-run",
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
