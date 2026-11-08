# haste-jwt

The identity types Haste Health puts in its tokens and passes through its APIs. `claims::UserTokenClaims` is the access token payload: the standard `sub`, `exp`, `aud` and `scope`, the SMART on FHIR `fhirUser` and patient claims, and the Haste Health claims under `https://haste.health/` (tenant, project, user role, subscription tier, access policy versions, membership, FHIR version). `claims::IdTokenClaims` is the OpenID Connect id token. `scopes::Scopes` parses and prints a space separated scope string covering OpenID Connect scopes and SMART App Launch scopes (`launch`, `launch/patient`, `fhirUser`, `user/Patient.rs`, `system/*.cruds`). The crate also defines the id newtypes used throughout the stack, `TenantId`, `ProjectId`, `ResourceId`, `VersionId`, `AuthorId` and `AuthorKind`, which reserve the word `system` for the server itself, plus `UserRole` and `SupportedFHIRVersions`.

It does not sign or verify tokens; the server does that. It gives the server, [`haste-fhir-client`](https://crates.io/crates/haste-fhir-client), [`haste-repository`](https://crates.io/crates/haste-repository) and [`haste-access-control`](https://crates.io/crates/haste-access-control) one definition of what a token says.

## Install

```bash
cargo add haste-jwt
```

## Usage

```rust
use haste_jwt::scopes::{OIDCScope, Scope, Scopes, SmartScope};

let scopes = Scopes::try_from("openid fhirUser launch/patient user/*.*")?;
assert!(scopes.contains_scope(&Scope::OIDC(OIDCScope::OpenId)));
assert!(scopes.contains_scope(&Scope::SMART(SmartScope::FHIRUser)));

// Printing normalises permissions to the SMART v2 letters.
assert_eq!(String::from(scopes), "openid fhirUser launch/patient user/*.cruds");
```

A malformed scope (`sytem/Patient.cud`, `user/Patient.duc`) fails with an `OperationOutcomeError`, so the error can be returned to the client as is. Token payloads deserialize with serde:

```rust
use haste_jwt::claims::UserTokenClaims;

let claims: UserTokenClaims = serde_json::from_str(r#"{
  "sub": "user-1",
  "exp": 1760000000,
  "aud": "https://fhir.example.com",
  "scope": "openid user/*.rs",
  "https://haste.health/tenant": "acme",
  "https://haste.health/project": "ehr",
  "https://haste.health/subscription_tier": "team",
  "https://haste.health/user_role": "member",
  "https://haste.health/user_id": "user-1",
  "https://haste.health/resource_type": "Membership",
  "https://haste.health/access_policies": ["policy-v1"],
  "https://haste.health/fhir_version": "r4"
}"#)?;
assert_eq!(claims.tenant.as_ref(), "acme");
```

## Features

| Feature   | What it enables                                                                                                           |
| --------- | ------------------------------------------------------------------------------------------------------------------------- |
| `sqlx`    | PostgreSQL encode and decode for the id newtypes, `Scopes` and `SupportedFHIRVersions`, so they bind directly in queries  |
| `reflect` | `MetaValue` implementations for the id newtypes, so tokens and ids can take part in FHIRPath and access policy evaluation |

## Links

- [API reference](https://rust.haste.health/haste_jwt/)
- [Source](https://github.com/HasteHealth/HasteHealth/tree/main/backend/crates/jwt)
- [Rust SDK overview](https://haste.health/docs/api/sdks/rust), [scopes and permissions](https://haste.health/docs/auth/authorization/scopes) and [SMART on FHIR](https://haste.health/docs/auth/authentication/smart_on_fhir)
- TypeScript equivalent: [@haste-health/jwt](https://www.npmjs.com/package/@haste-health/jwt)

Part of [Haste Health](https://haste.health), licensed under [Apache-2.0](https://github.com/HasteHealth/HasteHealth/blob/main/LICENSE).
