// Access control TestScripts: what a principal limited by scopes or access
// policies can reach. Each directory in backend/testscripts/access-control is a
// scenario:
//   scenario.json  who signs in: a user through a client (grantType
//                  authorization_code) or a client on its own
//                  (client_credentials), and the scopes requested
//   setup/         TestScripts run as the admin: the data, the principal's
//                  access policies, the client and any user
//   tests/         TestScripts run as the principal
//
// Run the phases in order against a running server (test_e2e_access_control.yml):
//   node .github/scripts/access_control.mjs setup
//   node .github/scripts/access_control.mjs sign-in
//   node .github/scripts/access_control.mjs test
//
// env: ADMIN_PROFILE (testing), TENANT (my-health), PROJECT (system),
//      HASTE_API_URI (http://localhost:3000)
import { spawnSync } from "node:child_process";
import { randomBytes } from "node:crypto";
import { mkdirSync, readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { headlessLogin } from "./headless_login.mjs";

const backendDir = join(import.meta.dirname, "../../backend");
const cliPath = join(backendDir, "target/debug/haste-health");
const scenariosDir = join(backendDir, "testscripts/access-control");
const reportsDir = join(backendDir, "access-control-reports");

const {
  ADMIN_PROFILE = "testing",
  TENANT = "my-health",
  PROJECT = "system",
  HASTE_API_URI = "http://localhost:3000",
} = process.env;

const phases = {
  // Writes the scenario's data, user and client as the admin.
  setup(scenario) {
    cli("config set-active-profile", { name: ADMIN_PROFILE });
    runTestScripts(scenario, "setup", `${scenario.name}-setup.json`);
  },

  // Signs the principal in with a fresh credential, into the scenario's own CLI
  // profile.
  async "sign-in"(scenario) {
    const signIn = signInBy[scenario.grantType];
    if (!signIn) throw new Error(`Unknown grantType '${scenario.grantType}'`);
    deleteProfile(scenario.profile);
    await signIn(scenario);
  },

  // Runs the scenario's tests as the principal.
  test(scenario) {
    cli("config set-active-profile", { name: scenario.profile });
    runTestScripts(scenario, "tests", `${scenario.name}.json`);
  },
};

// Credentials are generated each run, so none is kept in the repo.
const signInBy = {
  // A user, through a public client.
  async authorization_code(scenario) {
    const password = randomBytes(24).toString("base64");
    cli("admin user set-password", {
      tenant: TENANT,
      email: scenario.userEmail,
      password,
    });
    cli("config create-profile", {
      ...profileFlags(scenario, "authorization-code"),
      "redirect-uri": scenario.redirectUri,
    });
    await headlessLogin(cliPath, scenario.userEmail, password);
  },

  // The client itself, with a secret.
  client_credentials(scenario) {
    const secret = randomBytes(24).toString("base64");
    cli("config set-active-profile", { name: ADMIN_PROFILE });
    // Quiet: it prints the client, secret included.
    cli(
      `api patch ClientApplication ${scenario.clientId}`,
      { data: JSON.stringify([{ op: "add", path: "/secret", value: secret }]) },
      { quiet: true },
    );
    cli("config create-profile", {
      ...profileFlags(scenario, "client-credentials"),
      secret,
    });
  },
};

function profileFlags(scenario, authMode) {
  return {
    name: scenario.profile,
    "auth-mode": authMode,
    "r4-url": `${HASTE_API_URI}/w/${TENANT}/${PROJECT}/api/v1/fhir/r4`,
    "discovery-uri": `${HASTE_API_URI}/.well-known/openid-configuration/w/${TENANT}/${PROJECT}`,
    id: scenario.clientId,
    scope: scenario.scope,
  };
}

// `cli("config set-active-profile", { name: "testing" })` runs
// `haste-health config set-active-profile --name testing` from backend/, where
// `admin` commands read haste.toml and .env.
function cli(command, flags = {}, { quiet = false } = {}) {
  const args = [
    ...command.split(" "),
    ...Object.entries(flags).flatMap(([flag, value]) => [`--${flag}`, value]),
  ];
  const { status } = spawnSync(cliPath, args, {
    cwd: backendDir,
    stdio: ["ignore", quiet ? "ignore" : "inherit", "inherit"],
  });
  if (status !== 0) throw new Error(`haste-health ${command} failed`);
}

function runTestScripts(scenario, directory, report) {
  cli("testscript run", {
    input: join(scenario.dir, directory),
    "index-wait-ms": "30000",
    output: join(reportsDir, report),
  });
}

// Drops a previous run's profile and its tokens. Quiet when there is none.
function deleteProfile(name) {
  spawnSync(
    cliPath,
    ["config", "delete-profile", "--name", name, "--confirm", "true"],
    { cwd: backendDir },
  );
}

function loadScenarios() {
  return readdirSync(scenariosDir, { withFileTypes: true })
    .filter((entry) => entry.isDirectory())
    .map(({ name }) => {
      const dir = join(scenariosDir, name);
      const scenario = JSON.parse(
        readFileSync(join(dir, "scenario.json"), "utf-8"),
      );
      return { ...scenario, name, dir, profile: `access-control-${name}` };
    });
}

const phaseName = process.argv[2];
const phase = phases[phaseName];
if (!phase) {
  console.error(
    `usage: node .github/scripts/access_control.mjs <${Object.keys(phases).join("|")}>`,
  );
  process.exit(2);
}

mkdirSync(reportsDir, { recursive: true });
const failed = [];
// One at a time: scenarios share the CLI's active profile and the login's
// loopback port.
for (const scenario of loadScenarios()) {
  console.log(`=== ${phaseName}: ${scenario.name}`);
  try {
    await phase(scenario);
  } catch (error) {
    console.error(error.message);
    failed.push(scenario.name);
  }
}

// Whatever runs next uses the admin profile.
cli("config set-active-profile", { name: ADMIN_PROFILE });

if (failed.length) {
  console.error(`${phaseName} failed for: ${failed.join(", ")}`);
  process.exit(1);
}
