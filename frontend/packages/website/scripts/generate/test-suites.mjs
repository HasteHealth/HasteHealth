// The test suites the conformance pages show beside the core report: each
// suite's TestScripts, grouped as the suite is organised, with every
// assertion's result per search backend. One compact file per suite, plus a
// summary the overview page and the homepage read.
//
//   static/test-reports/suites/<suite>.json
//   static/test-reports/suites.json
//
// Options, one per suite and search backend, each a TestReport Bundle or a
// directory of them (the access control and g10 runs write one per scenario):
//
//   --us-core-postgres <path>         --us-core-elasticsearch <path>
//   --g10-postgres <path>
//   --access-control-postgres <path>  --access-control-elasticsearch <path>
//   --sql-on-fhir-postgres <path>
//
// Defaults: backend/testreports/<suite>/<backend>(.json). A suite with no
// reports at all keeps its committed file, so a run whose job failed still
// deploys the last published results. The summary is always rewritten from the
// suite files and support.json.

import fs from "node:fs";
import path from "node:path";

import {
  BACKEND,
  REPO,
  SUPPORT,
  combine,
  jsonFiles,
  readJson,
  readResources,
  relative,
  site,
  writeFile,
} from "./lib.mjs";

export const SUITES_DIR = site("static/test-reports/suites");
export const SUITES_SUMMARY = site("static/test-reports/suites.json");

// Set by `testscript run --allow-failure` on a TestReport whose failure is known.
const EXPECTED_FAILURE_URL =
  "https://haste.health/fhir/StructureDefinition/testreport-expected-failure";

const G10_GROUPS = {
  "standalone-patient-app": "1 · Standalone patient app, full access",
  "limited-access-app": "2 · Standalone patient app, limited access",
  "ehr-practitioner-app": "3 · EHR practitioner app",
  "multi-patient-api": "8 · Multi-patient API (Bulk Data)",
  "smart-v1-scopes": "9.22 · SMART v1 scopes",
  "fine-grained-scopes-1": "9.25.1 · Fine-grained scopes, group 1",
  "fine-grained-scopes-2": "9.25.2 · Fine-grained scopes, group 2",
  "single-patient-api": "12 · Single patient API (US Core 7)",
  "profile-validation": "12 · Profile validation",
  setup: "Shared setup: the US Core patient record",
};

// One group per file of the SQL on FHIR shared test suite.
const SQL_ON_FHIR_GROUPS = {
  basic: "Basic views",
  collection: "Collection columns",
  constant: "Constants",
  constant_types: "Constant types",
  fhirpath: "FHIRPath expressions",
  fhirpath_numbers: "FHIRPath numbers",
  fn_boundary: "lowBoundary() and highBoundary()",
  fn_empty: "empty()",
  fn_oftype: "ofType()",
  foreach: "forEach and forEachOrNull",
  repeat: "repeat",
  row_index: "%rowIndex",
  union: "unionAll",
  validate: "Invalid views",
  view_resource: "View resource type",
};

const ACCESS_CONTROL_GROUPS = {
  "patient-scopes": "Patient scopes",
  "user-scopes": "User scopes",
  "system-scopes": "System scopes",
  "smart-launch": "SMART EHR launch",
};

/** How each suite is found, grouped and named. */
const SUITES = [
  {
    id: "us-core",
    title: "US Core",
    page: "/docs/reference/conformance/us-core",
    backends: ["postgres", "elasticsearch"],
    // Scripts live under their US Core version: 9.0.0/examples/ and
    // 9.0.0/profiles/. A generated example script is named after its example
    // file: ---artifacts-r4-test-data-testscripts-us_core-Observation-bmi-json
    // The profile conformance scripts are one group per version, whose
    // lowercase id sorts after the resource type groups.
    group(relativePath, testScript) {
      const version = /^(\d+\.\d+\.\d+)\/profiles\//.exec(relativePath)?.[1];
      if (version) {
        return {
          id: `us-core-${version}`,
          title: `US Core ${version} profiles`,
        };
      }
      const match = /us_core-([A-Z][A-Za-z]+)-/.exec(testScript.id);
      return { id: match?.[1] ?? "Patient", title: match?.[1] ?? "Patient" };
    },
    scriptTitle(testScript) {
      const match = /us_core-([A-Z][A-Za-z]+)-(.+)-json$/.exec(testScript.id);
      return match ? `${match[1]} ${match[2]}` : testScript.title;
    },
  },
  {
    id: "g10",
    title: "ONC (g)(10) Standardized API",
    page: "/docs/reference/conformance/onc-g10",
    backends: ["postgres"],
    knownFailures: "known-failures.txt",
    group(relativePath) {
      const [scenario, phase, folder] = relativePath.split(path.sep);
      // single-patient-api/tests/12.18-observation-lab/... -> "12.18 · observation lab"
      const subgroup =
        scenario === "setup" || phase === "setup"
          ? "Setup"
          : folder?.endsWith(".json") === false
            ? folder.replace(/^([\d.]+)-/, "$1 · ").replaceAll("-", " ")
            : undefined;
      return {
        id: scenario,
        title: G10_GROUPS[scenario] ?? scenario,
        subgroup,
      };
    },
    order: Object.keys(G10_GROUPS),
  },
  {
    id: "access-control",
    title: "SMART scopes and access control",
    page: "/docs/reference/conformance/access-control",
    backends: ["postgres", "elasticsearch"],
    group(relativePath) {
      const [scenario, phase] = relativePath.split(path.sep);
      return {
        id: scenario,
        title: ACCESS_CONTROL_GROUPS[scenario] ?? scenario,
        subgroup: phase === "setup" ? "Setup" : undefined,
      };
    },
    order: Object.keys(ACCESS_CONTROL_GROUPS),
  },
  {
    id: "sql-on-fhir",
    title: "SQL on FHIR v2",
    page: "/docs/reference/conformance/sql-on-fhir",
    backends: ["postgres"],
    knownFailures: "known-failures.txt",
    // CI runs the listed TestScripts with --allow-failure instead of skipping
    // them, so their reports say what passed.
    knownFailuresRun: true,
    // basic/01-basic-attribute.testscript.json -> "basic.json · Basic views"
    group(relativePath) {
      const [file] = relativePath.split(path.sep);
      return {
        id: file,
        title: `${file}.json · ${SQL_ON_FHIR_GROUPS[file] ?? file}`,
      };
    },
    // "SQL on FHIR basic: basic attribute" -> "01 · basic attribute"
    scriptTitle(testScript, relativePath) {
      const number = /^(\d+)-/.exec(path.basename(relativePath))?.[1];
      const title = testScript.title.replace(/^SQL on FHIR \w+: /, "");
      return number ? `${number} · ${title}` : title;
    },
    // The rows assert compares base64; its description already shows the rows.
    expression: (assert) =>
      assert.expression?.startsWith("Binary.data = '")
        ? undefined
        : assert.expression,
    order: Object.keys(SQL_ON_FHIR_GROUPS),
  },
];

/** A TestReport action result as the website's Result. Mirrors test-support.mjs. */
function toResult(result, report) {
  if (result === "fail" || result === "error") {
    const expected = (report?.extension ?? []).some(
      (extension) =>
        extension.url === EXPECTED_FAILURE_URL &&
        extension.valueBoolean === true,
    );
    return expected ? "warn" : "fail";
  }
  if (result === "pass" || result === "warning") return "pass";
  return "not-run";
}

/** TestReports by id from a Bundle file or a directory of them. */
function readReports(location) {
  const files = fs.statSync(location).isDirectory()
    ? jsonFiles(location)
    : [location];
  const reports = new Map();
  for (const file of files) {
    for (const resource of readResources(file)) {
      if (resource?.resourceType === "TestReport") {
        reports.set(resource.id, resource);
      }
    }
  }
  return reports;
}

/**
 * known-failures.txt: TestScript paths or ids, or scenario directories ending
 * in `/`, each under the comment block that explains it. A `#` after an entry
 * starts a comment.
 */
function readKnownFailures(file) {
  if (!fs.existsSync(file)) return [];
  const entries = [];
  let reason = [];
  let inComment = false;
  for (const raw of fs.readFileSync(file, "utf8").split("\n")) {
    const line = raw.trim();
    if (line.startsWith("#")) {
      if (!inComment) reason = [];
      reason.push(line.replace(/^#\s?/, ""));
      inComment = true;
    } else {
      inComment = false;
      const entry = line.replace(/\s+#.*$/, "");
      if (entry) entries.push({ path: entry, reason: reason.join(" ") });
    }
  }
  return entries;
}

function knownFailureOf(entries, relativePath, testScript) {
  const posix = relativePath.split(path.sep).join("/");
  return entries.find((entry) =>
    entry.path.endsWith("/")
      ? posix.startsWith(entry.path)
      : posix === entry.path || testScript.id === entry.path,
  )?.reason;
}

/** What an assert checks, in words, when it carries no label of its own. */
function assertionLabel(assert, index) {
  if (assert.description) return assert.description;
  if (assert.label) return assert.label;
  if (assert.responseCode) return `Response code is ${assert.responseCode}`;
  if (assert.response) return `Response is ${assert.response}`;
  if (assert.headerField) return `Header ${assert.headerField} checked`;
  if (assert.contentType) return `Content type is ${assert.contentType}`;
  if (assert.resource && !assert.expression) {
    return `Response is a ${assert.resource}`;
  }
  return assert.expression ?? `Assertion ${index + 1}`;
}

function testName(test, index) {
  const name = test.name ?? "";
  // The generated US Core scripts name every test after their tag.
  if (!name || name.startsWith("Test for resource with tag")) {
    const operation = test.action?.find((action) => action.operation);
    const type = operation?.operation?.type?.code;
    const resource = operation?.operation?.resource;
    return type && resource ? `${type} ${resource}` : `Test ${index + 1}`;
  }
  return name;
}

/** One TestScript's tests and assertions with each backend's results. */
function scriptResults(testScript, reports, backends, suite) {
  const expressionOf = suite.expression ?? ((assert) => assert.expression);
  const sections = [
    ...(testScript.setup?.action?.some((action) => action.assert)
      ? [
          {
            name: "Setup",
            actions: testScript.setup.action,
            report: (report) => report?.setup?.action,
          },
        ]
      : []),
    ...(testScript.test ?? []).map((test, testIndex) => ({
      name: testName(test, testIndex),
      description: test.description,
      actions: test.action ?? [],
      report: (report) => report?.test?.[testIndex]?.action,
    })),
  ];
  return sections
    .map((section) => ({
      name: section.name,
      ...(section.description ? { description: section.description } : {}),
      assertions: section.actions.flatMap((action, actionIndex) =>
        action.assert
          ? [
              {
                label: assertionLabel(action.assert, actionIndex),
                ...(expressionOf(action.assert)
                  ? { expression: expressionOf(action.assert) }
                  : {}),
                results: Object.fromEntries(
                  backends.map((backend) => [
                    backend,
                    toResult(
                      section.report(reports[backend])?.[actionIndex]?.assert
                        ?.result,
                      reports[backend],
                    ),
                  ]),
                ),
              },
            ]
          : [],
      ),
    }))
    .filter((test) => test.assertions.length > 0);
}

/** Natural order, so 12.9 sorts before 12.10. */
const collator = new Intl.Collator("en", { numeric: true });

function buildSuite(suite, locations) {
  const backends = suite.backends.filter((backend) => locations[backend]);
  const reports = Object.fromEntries(
    backends.map((backend) => [backend, readReports(locations[backend])]),
  );
  const dir = path.join(BACKEND, "testscripts", suite.id);
  const knownFailures = suite.knownFailures
    ? readKnownFailures(path.join(dir, suite.knownFailures))
    : [];

  const groups = new Map();
  for (const file of jsonFiles(dir)) {
    const testScript = readJson(file);
    if (testScript.resourceType !== "TestScript") continue;
    const relativePath = path.relative(dir, file);
    const { id, title, subgroup } = suite.group(relativePath, testScript);
    const scriptReports = Object.fromEntries(
      backends.map((backend) => [backend, reports[backend].get(testScript.id)]),
    );
    const tests = scriptResults(testScript, scriptReports, backends, suite);
    // A skipped known failure has no report, so it shows as a known gap. One
    // that ran shows its report's result: a listed TestScript that passed is
    // no longer a gap.
    const result = Object.fromEntries(
      backends.map((backend) => [
        backend,
        toResult(scriptReports[backend]?.result, scriptReports[backend]),
      ]),
    );
    const listedFailure = knownFailureOf(
      knownFailures,
      relativePath,
      testScript,
    );
    const knownFailure =
      suite.knownFailuresRun &&
      backends.every((backend) => result[backend] === "pass")
        ? undefined
        : listedFailure;
    if (knownFailure && !suite.knownFailuresRun) {
      for (const backend of backends) result[backend] = "warn";
    }
    const group = groups.get(id) ?? { id, title, scripts: [] };
    group.scripts.push({
      id: testScript.id,
      title:
        suite.scriptTitle?.(testScript, relativePath) ??
        testScript.title ??
        testScript.name,
      ...(testScript.description
        ? { description: testScript.description }
        : {}),
      path: path.relative(REPO, file).split(path.sep).join("/"),
      ...(subgroup ? { subgroup } : {}),
      ...(knownFailure ? { knownFailure } : {}),
      result,
      tests,
    });
    groups.set(id, group);
  }

  const order = suite.order ?? [];
  const rank = (id) => (order.includes(id) ? order.indexOf(id) : order.length);
  const sorted = [...groups.values()]
    .sort((a, b) => rank(a.id) - rank(b.id) || collator.compare(a.id, b.id))
    .map((group) => ({
      ...group,
      scripts: group.scripts.sort(
        (a, b) =>
          (a.subgroup === "Setup" ? -1 : 0) -
            (b.subgroup === "Setup" ? -1 : 0) ||
          collator.compare(a.subgroup ?? "", b.subgroup ?? "") ||
          collator.compare(a.title, b.title),
      ),
    }));

  return {
    id: suite.id,
    title: suite.title,
    generatedAt: new Date().toISOString(),
    backends,
    knownFailures: knownFailures.length,
    ...(suite.knownFailuresRun ? { knownFailuresRun: true } : {}),
    groups: sorted,
  };
}

/** Counts for the overview page and the homepage. */
function totals(report) {
  const scripts = report.groups.flatMap((group) => group.scripts);
  const assertions = scripts.flatMap((script) =>
    script.tests.flatMap((test) => test.assertions),
  );
  const everywhere = (results, value) =>
    report.backends.every((backend) => results[backend] === value);
  return {
    testScripts: scripts.length,
    testScriptsPassing: scripts.filter((script) =>
      everywhere(script.result, "pass"),
    ).length,
    knownGaps: scripts.filter((script) => script.knownFailure).length,
    assertions: assertions.length,
    assertionsPassing: assertions.filter((assertion) =>
      everywhere(assertion.results, "pass"),
    ).length,
  };
}

/** The core report's counts, as the homepage has always computed them. */
function coreTotals() {
  const report = readJson(SUPPORT);
  const assertions = report.resources.flatMap((resource) =>
    resource.groups.flatMap((group) => group.assertions),
  );
  const resultOf = (resource) =>
    combine(
      resource.groups.flatMap((group) =>
        report.backends.map((backend) => group.results[backend]),
      ),
    );
  return {
    id: "core",
    title: "FHIR R4 core",
    page: "/docs/reference/conformance/test-coverage",
    generatedAt: report.generatedAt,
    backends: report.backends,
    testScripts: report.resources.length,
    testScriptsPassing: report.resources.filter(
      (resource) => resultOf(resource) === "pass",
    ).length,
    knownGaps: report.resources.filter(
      (resource) => resultOf(resource) === "warn",
    ).length,
    assertions: assertions.length,
    assertionsPassing: assertions.filter((assertion) =>
      report.backends.every((backend) => assertion.results[backend] === "pass"),
    ).length,
  };
}

function defaultLocation(suite, backend) {
  const base = path.join(BACKEND, "testreports", suite, backend);
  if (fs.existsSync(base)) return base;
  return fs.existsSync(`${base}.json`) ? `${base}.json` : undefined;
}

export async function run({ args }) {
  const written = [];
  const kept = [];
  for (const suite of SUITES) {
    const locations = Object.fromEntries(
      suite.backends.map((backend) => {
        const location =
          args[`${suite.id}-${backend}`] ?? defaultLocation(suite.id, backend);
        if (location && !fs.existsSync(location)) {
          throw new Error(`${location} does not exist.`);
        }
        return [backend, location];
      }),
    );
    const file = path.join(SUITES_DIR, `${suite.id}.json`);
    if (Object.values(locations).every((location) => !location)) {
      if (!fs.existsSync(file)) {
        throw new Error(
          `No TestReports for ${suite.id} and no committed ${relative(file)}.`,
        );
      }
      kept.push(suite.id);
      continue;
    }
    // Compact: the suite page downloads this file at runtime.
    writeFile(file, `${JSON.stringify(buildSuite(suite, locations))}\n`);
    written.push(suite.id);
  }

  const suites = [
    coreTotals(),
    ...SUITES.map((suite) => {
      const report = readJson(path.join(SUITES_DIR, `${suite.id}.json`));
      return {
        id: suite.id,
        title: suite.title,
        page: suite.page,
        generatedAt: report.generatedAt,
        backends: report.backends,
        ...totals(report),
      };
    }),
  ];
  writeFile(SUITES_SUMMARY, `${JSON.stringify({ suites }, null, 2)}\n`);

  const checks = suites.reduce((sum, suite) => sum + suite.assertions, 0);
  return [
    written.length ? `wrote ${written.join(", ")}` : undefined,
    kept.length ? `kept the committed ${kept.join(", ")}` : undefined,
    `${checks} checks across ${suites.length} suites in ${relative(SUITES_SUMMARY)}`,
  ]
    .filter(Boolean)
    .join("; ");
}
