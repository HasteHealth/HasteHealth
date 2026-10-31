# @haste-health/jwt

Types and helpers for the JSON Web Tokens Haste Health issues. The access and ID token payload types spell out the standard OpenID Connect claims, the SMART on FHIR launch context (`fhirUser`, `patient`, `encounter`, `fhirContext`) and Haste Health's own namespaced claims for the tenant, project, user role and access policies. A `JWT<Payload>` is a branded string, so parsing a token returns a payload that is already typed.

The Haste Health admin app and [`@haste-health/components`](https://www.npmjs.com/package/@haste-health/components) use it to read the signed-in user's claims. The Node-side helpers create signing keys, sign tokens and expose the matching JWKS.

## Install

```bash
npm install @haste-health/jwt
```

## Usage

Reading claims in the browser (decodes only, no signature check):

```ts
import { CUSTOM_CLAIMS, parseJwt, type AccessToken } from "@haste-health/jwt";

const claims = parseJwt(accessToken as AccessToken<string>);

claims.sub; // subject
claims.scope; // granted scopes, space separated
claims[CUSTOM_CLAIMS.TENANT]; // "https://haste.health/tenant"
claims[CUSTOM_CLAIMS.PROJECT]; // "https://haste.health/project"
claims.patient; // SMART launch context, when present
```

Issuing tokens on a server with [jose](https://github.com/panva/jose):

```ts
import {
  createCertsIfNoneExists,
  getJWKS,
  getSigningKey,
} from "@haste-health/jwt/certifications";
import { createToken } from "@haste-health/jwt/token";

const keys = {
  type: "file",
  directory: "./certs",
  kid: "dev",
  alg: "RS384",
} as const;

await createCertsIfNoneExists(keys); // development only; use type: "environment" in production
const signingKey = await getSigningKey(keys);
const token = await createToken({ signingKey, payload, expiresIn: "1h" });
const jwks = await getJWKS(keys); // public keys for the issuer's JWKS endpoint
```

## Claims

| Constant                                     | Claim                                             | Meaning                                                                                    |
| -------------------------------------------- | ------------------------------------------------- | ------------------------------------------------------------------------------------------ |
| `CUSTOM_CLAIMS.TENANT`                       | `https://haste.health/tenant`                     | Tenant the token is scoped to                                                              |
| `CUSTOM_CLAIMS.PROJECT`                      | `https://haste.health/project`                    | Project the token is scoped to                                                             |
| `CUSTOM_CLAIMS.ROLE`                         | `https://haste.health/user_role`                  | The user's role in that tenant                                                             |
| `CUSTOM_CLAIMS.RESOURCE_TYPE`, `RESOURCE_ID` | `https://haste.health/resourceType`, `resourceId` | What the token was issued to: a `Membership`, `ClientApplication` or `OperationDefinition` |
| `CUSTOM_CLAIMS.ACCESS_POLICY_VERSION_IDS`    | `https://haste.health/accessPolicyVersionIds`     | Versions of the `AccessPolicyV2` resources in force                                        |

`AccessTokenPayload` and `IDTokenPayload` (with the OIDC profile claims such as `name`, `email` and `address`) are in `@haste-health/jwt/types`; `ALGORITHMS` lists the supported signing algorithms, `RS256` and `RS384`.

## Links

- [API reference](https://typescript.haste.health/modules/_haste-health_jwt.html)
- [Source](https://github.com/HasteHealth/HasteHealth/tree/main/frontend/packages/jwt)
- [TypeScript SDK overview](https://haste.health/docs/api/sdks/typescript)
- [Scopes and permissions](https://haste.health/docs/auth/authorization/scopes) and [SMART on FHIR](https://haste.health/docs/auth/authentication/smart_on_fhir) on haste.health
- Rust equivalent: [haste-jwt](https://crates.io/crates/haste-jwt)

Part of [Haste Health](https://haste.health), licensed under [Apache-2.0](https://github.com/HasteHealth/HasteHealth/blob/main/LICENSE).
