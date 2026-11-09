#!/usr/bin/env node
// Signs a private_key_jwt client assertion and requests a token with it, the
// way a SMART Backend Services client or a confidential asymmetric app does.
// Node's crypto module does the key handling, so there is nothing to install.
//
//   node scripts/private_key_jwt.mjs keygen --alg RS384 --kid k1 > private.jwk.json
//   node scripts/private_key_jwt.mjs jwks private.jwk.json
//       The JSON Web Key Set to register as ClientApplication.jwks (or to
//       publish at its jwksUri). Only the public parts of the key.
//   node scripts/private_key_jwt.mjs token <token_endpoint> --key private.jwk.json \
//       --client-id <id> [--scope "system/*.rs"] [--grant-type client_credentials]
//       Signs an assertion and POSTs the token request. Prints the request
//       and the response.
//
// Options of `token` that deliberately break the assertion, for checking
// what the server refuses: --aud <url>, --exp-in <seconds> (negative for an
// expired assertion), --jti <value> (send twice to replay it), --kid <kid>
// (one the key set lacks), --tamper (flip a byte of the signature).
// Any extra --<name> <value> is sent as another form parameter (for example
// --code, --code_verifier, --redirect_uri for an authorization_code grant).

import crypto from "node:crypto";
import fs from "node:fs";

const JWT_BEARER = "urn:ietf:params:oauth:client-assertion-type:jwt-bearer";

function parseArgs(argv) {
  const positional = [];
  const options = {};
  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg.startsWith("--")) {
      const name = arg.slice(2);
      const next = argv[i + 1];
      if (next === undefined || next.startsWith("--")) {
        options[name] = true;
      } else {
        options[name] = next;
        i += 1;
      }
    } else {
      positional.push(arg);
    }
  }
  return { positional, options };
}

function base64url(input) {
  return Buffer.from(input).toString("base64url");
}

function fail(message) {
  console.error(message);
  process.exit(2);
}

function keygen(options) {
  const alg = options.alg ?? "RS384";
  const kid = options.kid ?? crypto.randomUUID();
  let pair;
  if (alg === "RS384") {
    pair = crypto.generateKeyPairSync("rsa", { modulusLength: 2048 });
  } else if (alg === "ES384") {
    pair = crypto.generateKeyPairSync("ec", { namedCurve: "P-384" });
  } else {
    fail("--alg must be RS384 or ES384");
  }
  const jwk = pair.privateKey.export({ format: "jwk" });
  console.log(JSON.stringify({ ...jwk, alg, kid, use: "sig" }, null, 2));
}

function readKey(path) {
  if (!path) fail("--key <private.jwk.json> is required");
  const jwk = JSON.parse(fs.readFileSync(path, "utf8"));
  if (!jwk.kid) fail("the key needs a kid");
  if (!["RS384", "ES384"].includes(jwk.alg)) fail("the key's alg must be RS384 or ES384");
  return jwk;
}

function publicJwks(jwk) {
  const key = crypto.createPublicKey({ key: jwk, format: "jwk" });
  const pub = key.export({ format: "jwk" });
  return { keys: [{ ...pub, alg: jwk.alg, kid: jwk.kid, use: "sig" }] };
}

function sign(jwk, header, claims) {
  const key = crypto.createPrivateKey({ key: jwk, format: "jwk" });
  const signingInput = `${base64url(JSON.stringify(header))}.${base64url(JSON.stringify(claims))}`;
  const signature =
    jwk.alg === "RS384"
      ? crypto.sign("sha384", Buffer.from(signingInput), key)
      : crypto.sign("sha384", Buffer.from(signingInput), { key, dsaEncoding: "ieee-p1363" });
  return `${signingInput}.${base64url(signature)}`;
}

async function token(positional, options) {
  const tokenEndpoint = positional[0];
  if (!tokenEndpoint) fail("token <token_endpoint> is required");
  const jwk = readKey(options.key);
  const clientId = options["client-id"];
  if (!clientId) fail("--client-id <id> is required");

  const now = Math.floor(Date.now() / 1000);
  const expIn = options["exp-in"] === undefined ? 300 : Number(options["exp-in"]);
  const header = { alg: jwk.alg, typ: "JWT", kid: options.kid ?? jwk.kid };
  const claims = {
    iss: clientId,
    sub: clientId,
    aud: options.aud ?? tokenEndpoint,
    exp: now + expIn,
    jti: options.jti ?? crypto.randomUUID(),
  };
  let assertion = sign(jwk, header, claims);
  if (options.tamper) {
    const parts = assertion.split(".");
    const signature = Buffer.from(parts[2], "base64url");
    signature[0] ^= 0xff;
    parts[2] = signature.toString("base64url");
    assertion = parts.join(".");
  }

  const reserved = new Set(["key", "client-id", "scope", "grant-type", "aud", "exp-in", "jti", "kid", "tamper"]);
  const form = new URLSearchParams();
  form.set("grant_type", options["grant-type"] ?? "client_credentials");
  if (options.scope) form.set("scope", options.scope);
  form.set("client_assertion_type", JWT_BEARER);
  form.set("client_assertion", assertion);
  for (const [name, value] of Object.entries(options)) {
    if (!reserved.has(name) && value !== true) form.set(name, value);
  }

  console.log(`POST ${tokenEndpoint}`);
  console.log("Content-Type: application/x-www-form-urlencoded\n");
  for (const [name, value] of form) {
    console.log(`${name}=${name === "client_assertion" ? `${value.slice(0, 40)}...` : value}`);
  }
  console.log(`\nassertion header: ${JSON.stringify(header)}`);
  console.log(`assertion claims: ${JSON.stringify(claims)}\n`);

  const response = await fetch(tokenEndpoint, {
    method: "POST",
    headers: { "Content-Type": "application/x-www-form-urlencoded", Accept: "application/json" },
    body: form,
  });
  const body = await response.text();
  console.log(`HTTP ${response.status}`);
  try {
    console.log(JSON.stringify(JSON.parse(body), null, 2));
  } catch {
    console.log(body);
  }
  process.exitCode = response.ok ? 0 : 1;
}

const { positional, options } = parseArgs(process.argv.slice(2));
const command = positional.shift();
switch (command) {
  case "keygen":
    keygen(options);
    break;
  case "jwks":
    console.log(JSON.stringify(publicJwks(readKey(positional[0] ?? options.key)), null, 2));
    break;
  case "token":
    await token(positional, options);
    break;
  default:
    fail("usage: private_key_jwt.mjs keygen|jwks|token ... (see the comment at the top of the script)");
}
