#!/usr/bin/env node
// Sets every crate and npm package to one version, the GitHub release's.
//
//   node .github/scripts/set-version.mjs <version> [--forward-only] [--crates-only]
//
//   --forward-only  Do nothing if <version> is older than the current version.
//   --crates-only   Leave the npm packages alone (for builds that only compile Rust).
//
// Needs cargo, and pnpm 11 unless --crates-only.

import { execFileSync } from "node:child_process";
import { readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { parseArgs } from "node:util";

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "..", "..");
const BACKEND = join(ROOT, "backend");
const FRONTEND = join(ROOT, "frontend");
const CARGO_MANIFEST = join(BACKEND, "Cargo.toml");

// npm packages that keep their own version. US Core's is the IG's (9.0.0).
const OWN_VERSION_PACKAGES = ["@haste-health/hl7.fhir.us.core"];

const SEMVER = /^(\d+)\.(\d+)\.(\d+)(-[0-9A-Za-z.-]+)?(\+[0-9A-Za-z.-]+)?$/;

/** Whether `a` is older than `b`, by major.minor.patch. */
function isOlder(a, b) {
  const [x, y] = [a, b].map((v) => SEMVER.exec(v).slice(1, 4).map(Number));
  const i = x.findIndex((part, j) => part !== y[j]);
  return i !== -1 && x[i] < y[i];
}

const { values, positionals } = parseArgs({
  allowPositionals: true,
  options: {
    "forward-only": { type: "boolean" },
    "crates-only": { type: "boolean" },
  },
});

const version = positionals[0]?.replace(/^v/, "");
if (positionals.length !== 1 || !SEMVER.test(version)) {
  console.error(
    "Usage: set-version.mjs <version> [--forward-only] [--crates-only]",
  );
  process.exit(2);
}

// Every crate inherits [workspace.package]'s version, so that is the one line
// to change.
const lines = readFileSync(CARGO_MANIFEST, "utf8").split("\n");
let section;
const versionLine = lines.findIndex((line) => {
  section = /^\[([^\]]+)\]/.exec(line)?.[1] ?? section;
  return section === "workspace.package" && /^version\s*=/.test(line);
});
if (versionLine === -1) {
  throw new Error("backend/Cargo.toml has no [workspace.package] version.");
}

const current = /"(.*)"/.exec(lines[versionLine])[1];
if (values["forward-only"] && isOlder(version, current)) {
  console.log(`${version} is older than ${current}, nothing to do.`);
  process.exit(0);
}

lines[versionLine] = `version = "${version}"`;
writeFileSync(CARGO_MANIFEST, lines.join("\n"));

// Catch a crate that sets its own version instead of inheriting it.
const { packages } = JSON.parse(
  execFileSync("cargo", ["metadata", "--no-deps", "--format-version", "1"], {
    cwd: BACKEND,
    encoding: "utf8",
  }),
);
const stale = packages.filter((crate) => crate.version !== version);
if (stale.length > 0) {
  throw new Error(
    `Set version.workspace = true in: ${stale.map((crate) => crate.name).join(", ")}`,
  );
}

// Builds use --locked, which won't update Cargo.lock, so update it here.
// --workspace changes only the workspace crates' entries.
execFileSync("cargo", ["update", "--workspace"], {
  cwd: BACKEND,
  stdio: "inherit",
});

if (!values["crates-only"]) {
  // pnpm-lock.yaml doesn't record workspace package versions, so it stays as
  // is. --no-git-checks because the Cargo edits above dirtied the tree.
  execFileSync(
    "pnpm",
    [
      "version",
      version,
      "--recursive",
      "--allow-same-version",
      "--no-git-checks",
      ...OWN_VERSION_PACKAGES.flatMap((name) => ["--filter", `!${name}`]),
    ],
    { cwd: FRONTEND, stdio: "inherit" },
  );
}
