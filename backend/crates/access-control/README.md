# haste-access-control

The policy engine behind Haste Health's attribute-based access control. It evaluates `AccessPolicyV2` resources against an incoming FHIR request and the signed-in user, and answers with a `PermissionLevel`: `Allow`, `Deny` or `Undetermined`. The server's authorization middleware runs every policy assigned to the caller through `evaluate_policies` before a request reaches storage, and the `$evaluate-policy` operation exposes the same engine for testing a policy against a request.

Two engines are supported, chosen by `AccessPolicyV2.engine`:

- `full-access` allows everything.
- `rule-engine` walks the policy's `rule` tree. A rule has an optional `target` expression (should this rule run at all?), a `condition`, an `effect` (`permit`, the default, or `deny`) and nested rules combined with `combineBehavior` `any` or `all-of`. Expressions are FHIRPath or x-fhir-query. `%request` and `%user` are built in, and `attribute` entries fetch more context from the FHIR server (a `read`, `search-type` or `search-system`) and expose the result as `%<attributeId>`.

A policy denies as soon as one of its top-level rules denies. A request is allowed when any assigned policy allows it; otherwise the error is an `OperationOutcome` with issue type `forbidden`.

## Install

```bash
cargo add haste-access-control
```

## Usage

```rust
use std::sync::Arc;

use haste_access_control::context::{PolicyContext, PolicyEnvironment, UserInfo};
use haste_access_control::evaluate_policies;

let environment = PolicyEnvironment::new(
    tenant_id,
    project_id,
    fhir_request, // haste_fhir_client::request::FHIRRequest
    Arc::new(UserInfo { id: user_id }),
);
let context = PolicyContext::new(fhir_client, client_ctx, environment);

// Ok(context) when any policy allows the request, Err(OperationOutcomeError) otherwise.
let context = evaluate_policies(context, &policies).await?;
```

`policies` is a `&Vec<Arc<AccessPolicyV2>>`. `fhir_client` is an `Arc` of any `haste_fhir_client::FHIRClient<CTX, OperationOutcomeError>`; the engine uses it to resolve `attribute`s, so it should carry the caller's credentials. `evaluate_policy(Arc<PolicyContext>, Arc<AccessPolicyV2>)` evaluates a single policy and returns its `PermissionLevel`.

## Links

- [API reference](https://rust.haste.health/haste_access_control/)
- [Source](https://github.com/HasteHealth/HasteHealth/tree/main/backend/crates/access-control)
- [Rust SDK overview](https://haste.health/docs/api/sdks/rust)
- [Access control guide](https://haste.health/docs/auth/authorization/access_control), with the rule, attribute and variable syntax, and the [identity and access control concepts](https://haste.health/docs/core_concepts/identity_access_control)
- [AccessPolicyV2 schema](https://haste.health/docs/reference/fhir/model/resources/AccessPolicyV2); expressions run on [haste-fhirpath](https://crates.io/crates/haste-fhirpath) and [haste-x-fhir-query](https://crates.io/crates/haste-x-fhir-query)

Part of [Haste Health](https://haste.health), licensed under [Apache-2.0](https://github.com/HasteHealth/HasteHealth/blob/main/LICENSE).
