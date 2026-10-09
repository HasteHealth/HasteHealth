// The audits Audit Autofix fixes. Each has:
//   workflow  the audit workflow's name
//   paths     what its fixes may change
//   checks    workflows to run on its pull request
//   claude    commands Claude may run, how it verifies, where ignores go
//   images    (docker only) the images, one fix job each
//   audit()   the findings the audit workflow reports
//   fix()     applies the tooling's fixes
//   check()   (docker only) the findings after fixing, and notes for the
//             summary. The others run audit() again.
// A finding is { id, url, package, installed, fixed, title }.
import { execFileSync } from "node:child_process";
import { mkdirSync, mkdtempSync, readFileSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";

export const root = join(import.meta.dirname, "../../..");
const backendDir = join(root, "backend");
const frontendDir = join(root, "frontend");

/** A command as [command, ...args]: split on spaces, each ${value} one argument. */
export function cmd(strings, ...values) {
  const text = strings.reduce(
    (text, string, i) => `${text}\0${i - 1}\0${string}`,
  );
  return text
    .trim()
    .split(/\s+/)
    .map((word) => word.replace(/\0(\d+)\0/g, (_, i) => values[i]));
}

/** Runs a command and returns its stdout. */
export function output(
  [command, ...args],
  { allowFailure = false, ...options } = {},
) {
  try {
    return execFileSync(command, args, {
      cwd: root,
      encoding: "utf8",
      maxBuffer: 1 << 30,
      stdio: ["pipe", "pipe", "inherit"],
      ...options,
    });
  } catch (error) {
    // Audits exit non-zero when they find something, with the report on stdout.
    if (allowFailure && typeof error.stdout === "string") return error.stdout;
    throw error;
  }
}

/** Runs a command with its output in the log. */
export function run(
  [command, ...args],
  { allowFailure = false, ...options } = {},
) {
  try {
    execFileSync(command, args, { cwd: root, stdio: "inherit", ...options });
  } catch (error) {
    if (!allowFailure) throw error;
    console.log(`::warning::${command} ${args.join(" ")} failed`);
  }
}

const cargo = {
  workflow: "<Audit> Cargo",
  paths: ["backend"],
  checks: [
    "test_units.yml",
    "test_e2e.yml",
    "test_e2e_access_control.yml",
    "test_e2e_core_testscripts.yml",
    "test_e2e_hl7v2.yml",
  ],
  claude: {
    commands: [
      "cargo audit",
      "cargo update",
      "cargo tree",
      "cargo info",
      "cargo check",
      "cargo clippy",
    ],
    verify:
      "In backend/, run `cargo audit` and `cargo check --workspace --all-targets`, and `cargo clippy` with the flags in .github/workflows/test_units.yml on the crates whose code you changed.",
    ignores: "backend/.cargo/audit.toml",
  },

  audit() {
    const report = JSON.parse(
      output(cmd`cargo audit --json`, { cwd: backendDir, allowFailure: true }),
    );
    return report.vulnerabilities.list.map((v) => ({
      id: v.advisory.id,
      url: `https://rustsec.org/advisories/${v.advisory.id}`,
      package: v.package.name,
      installed: v.package.version,
      fixed: v.versions.patched.join(", ") || "none",
      title: v.advisory.title,
    }));
  },

  // Updates each vulnerable crate within its semver range, then the crates
  // that depend on what is left, since a crate can't move past what they allow.
  fix(findings) {
    const update = (crate) =>
      run(cmd`cargo update --package ${crate}`, {
        cwd: backendDir,
        allowFailure: true,
      });
    crates(findings).forEach(update);
    for (const crate of crates(cargo.audit()))
      dependents(crate).forEach(update);
  },
};

const crates = (findings) => [
  ...new Set(findings.map((f) => `${f.package}@${f.installed}`)),
];

// Registry crates that depend on `crate`, as name@version. Workspace and git
// crates are left out: `cargo update --package` can't move them.
function dependents(crate) {
  const tree = output(
    cmd`cargo tree --invert ${crate} --target all --prefix none --format {p}`,
    { cwd: backendDir },
  );
  const specs = tree
    .split("\n")
    .map((line) => line.match(/^(\S+) v(\S+)( \(\*\))?$/))
    .filter(Boolean)
    .map(([, name, version]) => `${name}@${version}`);
  return [...new Set(specs)];
}

const frontend = {
  workflow: "<Audit> Frontend Packages",
  paths: ["frontend", "artifacts"],
  checks: ["test_build_frontend.yml"],
  claude: {
    commands: [
      "pnpm audit",
      "pnpm why",
      "pnpm view",
      "pnpm install",
      "pnpm update",
      "pnpm --recursive run build",
    ],
    verify:
      "In frontend/, run `pnpm audit --audit-level=high`, `pnpm install` and `pnpm --recursive run build`.",
    ignores: "auditConfig.ignoreGhsas in frontend/pnpm-workspace.yaml",
  },

  audit() {
    const report = JSON.parse(
      output(cmd`pnpm audit --audit-level=high --json`, {
        cwd: frontendDir,
        allowFailure: true,
      }),
    );
    return Object.values(report.advisories).map((a) => ({
      id: a.github_advisory_id,
      url: a.url,
      package: a.module_name,
      installed: [...new Set(a.findings.map((f) => f.version))].join(", "),
      fixed: a.patched_versions,
      title: a.title,
    }));
  },

  // Overrides each vulnerable package to its patched version in
  // pnpm-workspace.yaml (and exempts that version from minimumReleaseAge).
  // This changes far less of the lockfile than `--fix update`.
  fix() {
    run(cmd`pnpm audit --audit-level=high --fix override`, {
      cwd: frontendDir,
      allowFailure: true,
    });
    run(cmd`pnpm install --lockfile-only`, { cwd: frontendDir });
  },
};

// For each image: its Dockerfile, the stage that builds the app, where that
// stage leaves it, where the published image has it, and a smoke test.
const images = {
  hastehealth: {
    dockerfile: "docker/haste-health.dockerfile",
    appStage: "builder",
    appPath: "build/backend/target/release/haste-health",
    publishedPath: "/haste-health",
    smokeTest: (image) =>
      output(cmd`docker run --rm ${image} --version`).trim(),
  },
  "admin-app": {
    dockerfile: "docker/admin-app/dockerfile",
    appStage: "node",
    appPath: "src/frontend/packages/admin-app/dist",
    publishedPath: "/usr/share/nginx/html",
    // nginx serves the app with the API URL filled in.
    smokeTest(image) {
      run(cmd`docker run --detach --rm --name autofix-smoke --publish 8080:80
        --env VITE_FHIR_BASE_URL=http://localhost:3000 ${image}`);
      try {
        const page = output(
          cmd`curl -fsS --retry 30 --retry-delay 1 --retry-all-errors http://localhost:8080/`,
        );
        const script = page.match(/window\.VITE_FHIR_BASE_URL = '[^']*'/);
        if (!script)
          throw new Error("The admin app page doesn't set VITE_FHIR_BASE_URL");
        return script[0];
      } finally {
        run(cmd`docker rm --force autofix-smoke`);
      }
    },
  },
};

const published = (image) => `ghcr.io/hastehealth/hastehealth/${image}:latest`;

const docker = {
  workflow: "<Audit> Docker Image",
  paths: ["docker"],
  checks: [], // The fix job already rebuilt, smoke tested and scanned the image.
  images: Object.keys(images),
  claude: {
    commands: [
      "node .github/scripts/autofix/autofix.mjs report docker",
      "trivy image",
    ],
    verify:
      "Run `node .github/scripts/autofix/autofix.mjs report docker <image>`: it rebuilds, smoke tests and scans the image.",
    ignores: null,
  },

  audit(image) {
    return trivy(published(image));
  },

  // Moves a pinned nginx base to its newest 1.x alpine tag. The server's
  // debian:bookworm-slim is a moving tag, which the rebuild pulls fresh.
  async fix(findings, image) {
    const dockerfile = join(root, images[image].dockerfile);
    const text = readFileSync(dockerfile, "utf8");
    const current = text.match(/^FROM nginx:(1\.\d+\.\d+)-alpine/m)?.[1];
    if (!current) return;
    const response = await fetch(
      "https://hub.docker.com/v2/namespaces/library/repositories/nginx/tags?page_size=100&name=-alpine&ordering=last_updated",
    );
    const tags = (await response.json()).results.map(
      (tag) => tag.name.match(/^(1\.\d+\.\d+)-alpine$/)?.[1],
    );
    const newest = [current, ...tags.filter(Boolean)]
      .sort(compareVersions)
      .at(-1);
    writeFileSync(
      dockerfile,
      text.replace(
        `FROM nginx:${current}-alpine`,
        `FROM nginx:${newest}-alpine`,
      ),
    );
  },

  // Rebuilds only the runtime stage, which is what Trivy reports on: the app
  // is copied from the published image instead of built.
  check(image) {
    const { dockerfile, appStage, appPath, publishedPath, smokeTest } =
      images[image];
    const app = mkdtempSync(join(tmpdir(), "app-"));
    mkdirSync(join(app, dirname(appPath)), { recursive: true });
    run(cmd`docker pull --quiet ${published(image)}`);
    const container = output(cmd`docker create ${published(image)}`).trim();
    run(cmd`docker cp ${container}:${publishedPath} ${join(app, appPath)}`);
    run(cmd`docker rm ${container}`);

    const rebuilt = `autofix/${image}`;
    run(
      cmd`docker build --pull --build-context ${appStage}=${app} --file ${dockerfile} --tag ${rebuilt} .`,
    );
    const smoke = smokeTest(rebuilt);
    const findings = trivy(rebuilt);

    const notes = [
      `Rebuilt from main with fresh base images and OS packages. Smoke test: \`${smoke}\`.`,
    ];
    if (output(cmd`git status --porcelain docker`).trim())
      notes.push("The image changes with the next release.");
    else if (!findings.length)
      notes.push(
        "No Dockerfile change needed: publish a release to replace the image.",
      );
    return { findings, notes };
  },
};

// Scans an image the way audit_docker.yml does.
function trivy(image) {
  const report = JSON.parse(
    output(
      cmd`trivy image --quiet --severity CRITICAL,HIGH --ignore-unfixed --format json ${image}`,
    ),
  );
  return (report.Results ?? [])
    .flatMap((result) => result.Vulnerabilities ?? [])
    .map((v) => ({
      id: v.VulnerabilityID,
      url: v.PrimaryURL,
      package: v.PkgName,
      installed: v.InstalledVersion,
      fixed: v.FixedVersion,
      title: v.Title ?? "",
    }));
}

function compareVersions(a, b) {
  const [x, y] = [a, b].map((version) => version.split(".").map(Number));
  return x[0] - y[0] || x[1] - y[1] || x[2] - y[2];
}

export const audits = { cargo, frontend, docker };
