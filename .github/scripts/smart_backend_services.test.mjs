// SMART Backend Services authorization with a signed JWT (private_key_jwt):
// the token requests the ONC (g)(10) Inferno test kit sends in 8.1.02 to
// 8.1.06 (smart-app-launch-test-kit, backend_services_authorization_group),
// with the status codes and response fields it asserts, then the server's own
// checks on the assertion. A TestScript cannot send these: the token request
// is a form POST, and each assertion needs a fresh signature, exp and jti.
//
// Run against a running server (test_e2e_g10.yml). Through the admin's CLI
// profile it registers a client of its own, CLIENT_ID, with a fresh RS384 and
// ES384 key as its jwks, and removes it afterwards: no other test's client is
// changed.
//   node --test .github/scripts/smart_backend_services.test.mjs
//
// env: ADMIN_PROFILE (testing), TENANT (my-health), PROJECT (system),
//      HASTE_API_URI (http://localhost:3000),
//      CLIENT_ID (smart-backend-services-test), SCOPE (system/*.rs)
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { generateKeyPairSync, randomBytes, sign } from "node:crypto";
import { join } from "node:path";
import { after, before, describe, test } from "node:test";
import { setTimeout as sleep } from "node:timers/promises";

const {
  ADMIN_PROFILE = "testing",
  TENANT = "my-health",
  PROJECT = "system",
  HASTE_API_URI = "http://localhost:3000",
  CLIENT_ID = "smart-backend-services-test",
  SCOPE = "system/*.rs",
} = process.env;

const JWT_BEARER = "urn:ietf:params:oauth:client-assertion-type:jwt-bearer";
const backendDir = join(import.meta.dirname, "../../backend");
const cliPath = join(backendDir, "target/debug/haste-health");
const fhirBase = `${HASTE_API_URI}/w/${TENANT}/${PROJECT}/api/v1/fhir/r4`;

const keys = { RS384: newKey("RS384"), ES384: newKey("ES384") };
let tokenEndpoint;

// The client the suite signs in as, and the policy that lets it read. Full
// access, as the (g)(10) bulk data client has, leaves SCOPE as its only limit.
const resources = [
  {
    resourceType: "ClientApplication",
    id: CLIENT_ID,
    name: "SMART Backend Services test client",
    grantType: ["client_credentials"],
    responseTypes: "token",
    scope: SCOPE,
    jwks: JSON.stringify({ keys: Object.values(keys).map((key) => key.publicJwk) }),
  },
  {
    resourceType: "AccessPolicyV2",
    id: `${CLIENT_ID}-policy`,
    name: "SMART Backend Services test client",
    engine: "full-access",
  },
  {
    resourceType: "AccessPolicyV2Assignment",
    id: `${CLIENT_ID}-policy-assignment`,
    accessPolicy: { reference: `AccessPolicyV2/${CLIENT_ID}-policy` },
    link: { reference: `ClientApplication/${CLIENT_ID}` },
  },
];

before(async () => {
  const response = await fetch(`${fhirBase}/.well-known/smart-configuration`);
  assert.equal(response.status, 200);
  const configuration = await response.json();
  assert.ok(configuration.capabilities.includes("client-confidential-asymmetric"));
  assert.ok(configuration.token_endpoint_auth_methods_supported.includes("private_key_jwt"));
  assert.deepEqual(configuration.token_endpoint_auth_signing_alg_values_supported, [
    "RS384",
    "ES384",
  ]);
  tokenEndpoint = configuration.token_endpoint;

  cli(`profile use ${ADMIN_PROFILE}`);
  let written;
  for (const resource of resources) {
    written = JSON.parse(
      cli(`api update ${resource.resourceType} ${resource.id}`, { data: JSON.stringify(resource) }),
    );
  }
  await indexed(written);
});

after(() => {
  cli(`profile use ${ADMIN_PROFILE}`);
  for (const { resourceType, id } of resources.toReversed()) {
    cli(`api delete-instance ${resourceType} ${id}`);
  }
});

// Inferno's tests, in its order. 8.1.01 (TLS) does not apply to a local http server.
describe("Inferno 8.1: SMART Backend Services Authorization", () => {
  test("8.1.02 fails when the client supplies an invalid grant_type", async () => {
    const { status } = await requestToken({ grant_type: "not_a_grant_type" });
    assert.equal(status, 400);
  });

  test("8.1.03 fails when the client supplies an invalid client_assertion_type", async () => {
    const { status } = await requestToken({ client_assertion_type: "not_an_assertion_type" });
    assert.ok([400, 401].includes(status), `status ${status}`);
  });

  // Inferno's own version of this test sends a bad client_assertion_type
  // again; this sends what its title describes, a JWT that does not verify.
  test("8.1.04 fails when the client supplies an invalid JWT", async () => {
    const { status } = await requestToken({ client_assertion: tamper(assertion()) });
    assert.ok([400, 401].includes(status), `status ${status}`);
  });

  for (const alg of Object.keys(keys)) {
    test(`8.1.05 and 8.1.06 succeed with an ${alg} assertion and return the required fields`, async () => {
      const { status, body } = await requestToken({ client_assertion: assertion({ alg }) });
      assert.ok([200, 201].includes(status), `status ${status}: ${JSON.stringify(body)}`);
      assert.ok(body.access_token, "access_token");
      assert.equal(body.token_type.toLowerCase(), "bearer");
      assert.equal(typeof body.expires_in, "number");
      assert.ok(body.scope, "scope");

      // The bulk data tests that follow in Inferno use this token.
      const read = await fetch(`${fhirBase}/Patient?_count=1`, {
        headers: { Authorization: `Bearer ${body.access_token}` },
      });
      assert.equal(read.status, 200);
    });
  }
});

// What SMART App Launch's client-confidential-asymmetric requires of the
// assertion: each failure is invalid_client.
describe("The client assertion", () => {
  const refused = [
    ["a reused jti", () => assertion(), { replay: true }],
    ["an aud other than the token endpoint", () => assertion({ aud: `${tokenEndpoint}/other` })],
    ["an exp more than five minutes ahead", () => assertion({ expIn: 10 * 60 })],
    ["an expired exp", () => assertion({ expIn: -5 * 60 })],
    ["no jti", () => assertion({ jti: undefined })],
    ["an iss other than its sub", () => assertion({ iss: "someone-else" })],
    ["a kid the client did not register", () => assertion({ kid: "unregistered" })],
    ["an algorithm other than RS384 or ES384", () => assertion({ alg: "RS256" })],
  ];

  for (const [name, build, { replay = false } = {}] of refused) {
    test(`is refused with ${name}`, async () => {
      const client_assertion = build();
      if (replay) {
        const first = await requestToken({ client_assertion });
        assert.equal(first.status, 200, JSON.stringify(first.body));
      }
      const { status, body } = await requestToken({ client_assertion });
      assert.equal(status, 401, JSON.stringify(body));
      assert.equal(body.error, "invalid_client");
    });
  }

  test("is refused when client_id names another client", async () => {
    const { status, body } = await requestToken({ client_id: "someone-else" });
    assert.equal(status, 401, JSON.stringify(body));
    assert.equal(body.error, "invalid_client");
  });

  test("is refused alongside a client_secret", async () => {
    const { status, body } = await requestToken({ client_secret: "secret" });
    assert.equal(status, 400, JSON.stringify(body));
    assert.equal(body.error, "invalid_request");
  });
});

// Waits until search finds `resource` as it was written. A write reaches the
// search index after it returns, and the token endpoint finds a client's
// policy through search: a token issued before then carries no policy, and
// its requests are answered 403. A project's writes are indexed in order, so
// once the last one is found they all are.
async function indexed({ resourceType, id, meta }) {
  for (const deadline = Date.now() + 60_000; Date.now() < deadline; await sleep(200)) {
    const found = JSON.parse(cli(`api search-type ${resourceType} _id=${id}`));
    if (found.entry?.some((entry) => entry.resource.meta.versionId === meta.versionId)) return;
  }
  throw new Error(`${resourceType}/${id} was not indexed within a minute`);
}

// A token request as Inferno's BackendServicesAuthorizationRequestBuilder
// sends it, with `overrides` in place of its fields.
async function requestToken(overrides) {
  const form = new URLSearchParams({
    scope: SCOPE,
    grant_type: "client_credentials",
    client_assertion_type: JWT_BEARER,
    client_assertion: overrides.client_assertion ?? assertion(),
    ...overrides,
  });
  const response = await fetch(tokenEndpoint, {
    method: "POST",
    headers: {
      "Content-Type": "application/x-www-form-urlencoded",
      Accept: "application/json",
    },
    body: form,
  });
  const text = await response.text();
  let body;
  try {
    body = JSON.parse(text);
  } catch {
    body = text;
  }
  return { status: response.status, body };
}

// An assertion as Inferno's ClientAssertionBuilder signs it: exp five minutes
// ahead and a 64-character hex jti. `jti: undefined` leaves the claim out.
function assertion({
  alg = "RS384",
  kid,
  iss = CLIENT_ID,
  aud = tokenEndpoint,
  expIn = 5 * 60,
  ...rest
} = {}) {
  const key = alg === "RS256" ? keys.RS384 : keys[alg];
  const jti = "jti" in rest ? rest.jti : randomBytes(32).toString("hex");
  const header = { alg, kid: kid ?? key.kid, typ: "JWT" };
  const claims = {
    iss,
    sub: CLIENT_ID,
    aud,
    exp: Math.floor(Date.now() / 1000) + expIn,
    ...(jti === undefined ? {} : { jti }),
  };
  const signingInput = `${base64url(JSON.stringify(header))}.${base64url(JSON.stringify(claims))}`;
  const digest = alg === "RS256" ? "sha256" : "sha384";
  const signature = alg.startsWith("ES")
    ? sign(digest, Buffer.from(signingInput), { key: key.privateKey, dsaEncoding: "ieee-p1363" })
    : sign(digest, Buffer.from(signingInput), key.privateKey);
  return `${signingInput}.${base64url(signature)}`;
}

// Flips a byte of the signature.
function tamper(jwt) {
  const [header, claims, signature] = jwt.split(".");
  const bytes = Buffer.from(signature, "base64url");
  bytes[0] ^= 0xff;
  return `${header}.${claims}.${bytes.toString("base64url")}`;
}

function newKey(alg) {
  const { privateKey, publicKey } =
    alg === "RS384"
      ? generateKeyPairSync("rsa", { modulusLength: 2048 })
      : generateKeyPairSync("ec", { namedCurve: "P-384" });
  const kid = `test-${alg.toLowerCase()}`;
  return {
    kid,
    privateKey,
    publicJwk: { ...publicKey.export({ format: "jwk" }), alg, kid, use: "sig" },
  };
}

function base64url(value) {
  return Buffer.from(value).toString("base64url");
}

// `cli("api update ClientApplication x", { data })` runs
// `haste-health api update ClientApplication x --data ...` from backend/ and
// returns what it prints. Logging is off, as it would print there too.
function cli(command, flags = {}) {
  const args = [
    ...command.split(" "),
    ...Object.entries(flags).flatMap(([flag, value]) => [`--${flag}`, value]),
  ];
  const { status, stdout } = spawnSync(cliPath, args, {
    cwd: backendDir,
    env: { ...process.env, RUST_LOG: "none" },
    stdio: ["ignore", "pipe", "inherit"],
    encoding: "utf8",
  });
  if (status !== 0) throw new Error(`haste-health ${command} failed`);
  return stdout;
}
