#!/usr/bin/env node
// Runs the haste-health binary for this platform. npm installs it from the
// matching package in optionalDependencies and skips the others.
"use strict";

const { spawn } = require("node:child_process");
const { optionalDependencies = {} } = require("../package.json");

const PREFIX = "@haste-health/cli-";
const platform = `${process.platform}-${process.arch}`;
const binaryPackage = PREFIX + platform;

function fail(message) {
  console.error(`haste-health: ${message}`);
  process.exit(1);
}

if (!(binaryPackage in optionalDependencies)) {
  if (process.platform === "win32") {
    fail(
      "Windows isn't supported directly. Run it inside WSL:\n" +
        "  1. In PowerShell: wsl --install\n" +
        "  2. In the WSL terminal, install Node.js and run: npx haste-health",
    );
  }
  const available = Object.keys(optionalDependencies).map((name) =>
    name.slice(PREFIX.length),
  );
  fail(`no build for ${platform}. Builds exist for: ${available.join(", ")}.`);
}

let binary;
try {
  binary = require.resolve(`${binaryPackage}/bin/haste-health`);
} catch {
  fail(
    `${binaryPackage} is not installed. It's an optional dependency, so ` +
      "reinstall without --omit=optional (or --no-optional).",
  );
}

const child = spawn(binary, process.argv.slice(2), { stdio: "inherit" });

// Ctrl+C already reaches the binary (same process group), so just wait for it
// to exit. Stop signals sent only to this process are passed on.
process.on("SIGINT", () => {});
for (const signal of ["SIGTERM", "SIGHUP"]) {
  process.on(signal, () => child.kill(signal));
}

child.on("error", (error) => fail(`could not run ${binary}: ${error.message}`));
child.on("exit", (code, signal) => {
  if (signal) {
    // Exit the way the binary did.
    process.removeAllListeners(signal);
    process.kill(process.pid, signal);
  } else {
    process.exit(code ?? 1);
  }
});
