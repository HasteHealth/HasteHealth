#!/usr/bin/env node
// Assembles the npm packages for one release from its binaries.
//
//   node npm/build.mjs <version> <binaries-dir> <out-dir>
//
// <binaries-dir> holds the haste-health-<target> files that release.yml builds.
// <out-dir> gets one package per platform plus haste-health itself. Publish the
// platform packages first, since haste-health depends on them.

import {
  chmodSync,
  copyFileSync,
  cpSync,
  existsSync,
  mkdirSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const HERE = dirname(fileURLToPath(import.meta.url));
const LICENSE = join(HERE, "..", "LICENSE");

// Node's process.platform-process.arch, and the Rust target built for it.
const PLATFORMS = {
  "linux-x64": "x86_64-unknown-linux-musl",
  "linux-arm64": "aarch64-unknown-linux-musl",
  "darwin-arm64": "aarch64-apple-darwin",
};

const [version, binariesDir, outDir] = process.argv.slice(2);
if (!version || !binariesDir || !outDir) {
  console.error("Usage: build.mjs <version> <binaries-dir> <out-dir>");
  process.exit(2);
}

const writeJson = (file, value) =>
  writeFileSync(file, `${JSON.stringify(value, null, 2)}\n`);

const { name, ...main } = JSON.parse(
  readFileSync(join(HERE, "haste-health", "package.json"), "utf8"),
);
const optionalDependencies = {};

rmSync(outDir, { recursive: true, force: true });

for (const [platform, target] of Object.entries(PLATFORMS)) {
  const binary = join(binariesDir, `haste-health-${target}`);
  if (!existsSync(binary)) throw new Error(`Missing binary: ${binary}`);

  const packageName = `@haste-health/cli-${platform}`;
  const dir = join(outDir, `cli-${platform}`);
  const [os, cpu] = platform.split("-");

  mkdirSync(join(dir, "bin"), { recursive: true });
  copyFileSync(binary, join(dir, "bin", "haste-health"));
  // Workflow artifacts lose the executable bit.
  chmodSync(join(dir, "bin", "haste-health"), 0o755);
  copyFileSync(LICENSE, join(dir, "LICENSE"));
  writeJson(join(dir, "package.json"), {
    name: packageName,
    version,
    description: `The haste-health binary for ${platform}. Install haste-health instead.`,
    homepage: main.homepage,
    repository: { ...main.repository, directory: "npm" },
    license: main.license,
    os: [os],
    cpu: [cpu],
    files: ["bin"],
    // Yarn Plug'n'Play must unpack the binary to run it.
    preferUnplugged: true,
    publishConfig: { access: "public" },
  });

  optionalDependencies[packageName] = version;
}

const mainDir = join(outDir, "haste-health");
cpSync(join(HERE, "haste-health"), mainDir, { recursive: true });
copyFileSync(LICENSE, join(mainDir, "LICENSE"));
writeJson(join(mainDir, "package.json"), {
  name,
  version,
  ...main,
  optionalDependencies,
});

console.log(`Built haste-health ${version} for ${Object.keys(PLATFORMS).join(", ")} in ${outDir}`);
