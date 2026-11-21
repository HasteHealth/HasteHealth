# haste-subscription

The single source of truth for what each Haste Health subscription tier allows. One table, `TIERS`, holds everything a tier permits: the request budget the rate limiter enforces, the resource caps the tenant limit middleware enforces, whether a tenant may set its own name and logo, and the commercial facts (price, SLA, support response) the pricing page publishes. Nothing else in the workspace hardcodes a per-tier number.

The server reads it through `limits_for` and `resource_limit_for`, [`haste-repository`](https://crates.io/crates/haste-repository) depends on it, and `haste-health subscription export` writes it to the website as JSON so the published prices cannot drift from the enforced limits.

## Install

```bash
cargo add haste-subscription
```

## Usage

```rust
use haste_fhir_model::r4::generated::resources::ResourceType;
use haste_jwt::claims::SubscriptionTier;
use haste_subscription::{OPERATION_POINTS, ResourceLimit, limits_for, resource_limit_for};

let limits = limits_for(&SubscriptionTier::Free);
println!("{} allows {:?} projects", limits.display_name, limits.projects);

// Per-type caps exist for a few meta resources; every other type is Unlimited.
let cap = resource_limit_for(&SubscriptionTier::Free, &ResourceType::Subscription);
if cap.is_exceeded_at(current_subscription_count) {
    // refuse the create
}

// What a request costs against the tier's budget, in points.
let cost = OPERATION_POINTS.write;
```

## API

| Item                                        | Purpose                                                                                                                                    |
| ------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------ |
| `TIERS`, `DEFAULT_TIER`                     | Every tier in pricing-page order, and the tier a hosted signup starts on.                                                                  |
| `limits_for(&tier)`                         | The `TierLimits` for a tier. Infallible: a unit test keeps `TIERS` exhaustive over `SubscriptionTier`.                                     |
| `resource_limit_for(&tier, &resource_type)` | The cap on one type (`Project`, `SearchParameter`, `OperationDefinition`, `Subscription`, `IdentityProvider`); `Unlimited` for all others. |
| `allows_tenant_customization(&tier)`        | Whether the tenant may set a display name and logo.                                                                                        |
| `OPERATION_POINTS`                          | What a read, write, search, history and invocation cost, in budget points. Uniform across tiers.                                           |
| `ResourceLimit`, `RequestBudget`            | `Count(n)` or `Unlimited`; `Points(n)` or `Unmetered`, with `as_points()` for the rate limiter.                                            |
| `export::build()`                           | The pricing table as the website consumes it: the limits plus display labels and per-day budget examples.                                  |

## Publishing the pricing table

```bash
cd backend
cargo run subscription show             # print every tier as a table
cargo run subscription export           # write ../frontend/packages/website/static/pricing/tiers.json
cargo run subscription export --check   # fail if the committed file is out of date (run in CI)
```

## Links

- [API reference](https://rust.haste.health/haste_subscription/)
- [Source](https://github.com/HasteHealth/HasteHealth/tree/main/backend/crates/subscription)
- [Rust SDK overview](https://haste.health/docs/api/sdks/rust)
- [Pricing](https://haste.health/pricing), the page rendered from the export
- [CLI reference](https://github.com/HasteHealth/HasteHealth/blob/main/backend/documentation/cli_commands.md#haste-health-subscription); the `SubscriptionTier` claim comes from [haste-jwt](https://crates.io/crates/haste-jwt)

Part of [Haste Health](https://haste.health), licensed under [Apache-2.0](https://github.com/HasteHealth/HasteHealth/blob/main/LICENSE).
