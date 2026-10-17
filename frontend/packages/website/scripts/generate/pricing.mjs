// The subscription tiers the pricing page renders, exported by the backend so
// the page cannot say something the server does not enforce.
//
//   static/pricing/tiers.json

import { spawnSync } from "node:child_process";

import { BACKEND, relative, site } from "./lib.mjs";

const OUTPUT = site("static/pricing/tiers.json");

function cargo(extra) {
  const args = ["run", "--quiet", "subscription", "export", ...extra, "--output", OUTPUT];
  const result = spawnSync("cargo", args, { cwd: BACKEND, stdio: "inherit" });
  if (result.error) {
    throw new Error(`cargo could not be started: ${result.error.message}`);
  }
  if (result.status !== 0) {
    throw new Error(`cargo ${args.join(" ")} exited with ${result.status}`);
  }
}

export async function run() {
  cargo([]);
  return `Wrote ${relative(OUTPUT)}`;
}

export async function check() {
  cargo(["--check"]);
  return `${relative(OUTPUT)} is up to date`;
}
