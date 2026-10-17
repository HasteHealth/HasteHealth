// Shared by the generate steps: where things are, small file helpers, the
// result combinator the conformance data uses everywhere, and .env loading.

import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

/** frontend/packages/website, whatever the current directory is. */
export const SITE = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  "../..",
);
export const REPO = path.resolve(SITE, "../../..");
export const BACKEND = path.join(REPO, "backend");
export const ARTIFACTS = path.join(REPO, "artifacts/r4");

export const site = (...parts) => path.join(SITE, ...parts);

/** A path as printed in summaries: relative to the website package. */
export const relative = (file) => path.relative(SITE, file) || ".";

// Outputs one step writes and a later one reads.
export const SUPPORT = site("static/test-reports/support.json");
export const METADATA = site("static/fhir/R4/metadata.json");
export const OPERATION_DEFINITIONS = site("static/fhir/R4/OperationDefinition");
export const MCP_TOOLS = site("static/mcp/tools.json");

export function readJson(file) {
  return JSON.parse(fs.readFileSync(file, "utf8"));
}

export function writeFile(file, content) {
  fs.mkdirSync(path.dirname(file), { recursive: true });
  fs.writeFileSync(file, content);
}

/** Compact by default; `pretty` for files people read, such as the raw FHIR JSON. */
export function writeJson(file, value, { pretty = false } = {}) {
  writeFile(
    file,
    pretty ? JSON.stringify(value, null, 2) : JSON.stringify(value),
  );
}

/** The resources in a JSON file: a Bundle's entries, a list, or one. */
export function readResources(file) {
  const json = readJson(file);
  if (json.resourceType === "Bundle") {
    return (json.entry ?? []).map((entry) => entry.resource);
  }
  return Array.isArray(json) ? json : [json];
}

/** Every .json file under `dir`, in a stable order; `skip` names directories to leave out. */
export function jsonFiles(dir, { skip = [] } = {}) {
  if (!fs.existsSync(dir)) return [];
  return fs
    .readdirSync(dir, { withFileTypes: true })
    .sort((a, b) => a.name.localeCompare(b.name))
    .flatMap((entry) => {
      const file = path.join(dir, entry.name);
      if (entry.isDirectory()) {
        return skip.includes(entry.name) ? [] : jsonFiles(file, { skip });
      }
      return entry.name.endsWith(".json") ? [file] : [];
    });
}

/**
 * Removes the files in `dir` with `extension` that this run did not produce:
 * those whose name (without the extension) is not in `keep`.
 */
export function removeStale(dir, extension, keep) {
  if (!fs.existsSync(dir)) return 0;
  let removed = 0;
  for (const file of fs.readdirSync(dir)) {
    if (!file.endsWith(extension)) continue;
    if (keep.has(file.slice(0, -extension.length))) continue;
    fs.rmSync(path.join(dir, file));
    removed += 1;
  }
  return removed;
}

/**
 * The worst of several results: `fail`, then `warn` (a known failure), then
 * `pass` when everything ran; `not-run` if anything didn't. Mirrors combine()
 * in src/components/TestCoverage/data.ts.
 */
export function combine(results) {
  if (results.includes("fail")) return "fail";
  if (results.length === 0 || results.includes("not-run")) return "not-run";
  return results.includes("warn") ? "warn" : "pass";
}

/**
 * Loads `KEY=VALUE` lines from `.env` into process.env, for keys the
 * environment does not already set. Enough of the dotenv format for
 * .env.example: comments, blank lines and optional quotes.
 */
export function loadEnv(file = site(".env")) {
  if (!fs.existsSync(file)) return;
  for (const line of fs.readFileSync(file, "utf8").split(/\r?\n/)) {
    const match =
      /^\s*(?:export\s+)?([A-Za-z_][A-Za-z0-9_.]*)\s*=\s*(.*?)\s*$/.exec(line);
    if (!match || line.trimStart().startsWith("#")) continue;
    const [, key, raw] = match;
    const quoted =
      raw.length >= 2 &&
      (raw[0] === '"' || raw[0] === "'") &&
      raw.at(-1) === raw[0];
    process.env[key] ??= quoted ? raw.slice(1, -1) : raw;
  }
}
