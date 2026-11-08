# haste-rate-limit

The rate limiting interface the Haste Health server enforces request budgets through: one trait, `RateLimit`, and one error type, `RateLimitError`, with no dependencies, so a store can implement it without depending on the server.

[`haste-repository`](https://crates.io/crates/haste-repository) implements it for PostgreSQL (a `check_rate_limit` SQL function behind a short-lived in-memory cache), and the server's rate limit middleware calls it with the budget of the tenant's subscription tier from [`haste-subscription`](https://crates.io/crates/haste-subscription).

## Install

```bash
cargo add haste-rate-limit
```

## Usage

```rust
use haste_rate_limit::{RateLimit, RateLimitError};

async fn spend(limiter: &dyn RateLimit, tenant: &str) -> Result<(), RateLimitError> {
    // Spend 25 points against this tenant's budget of 250,000 per 24 hours.
    let used = limiter
        .check(&format!("tenant:{tenant}"), 250_000, 25, 24 * 60 * 60)
        .await?;

    println!("{used} points used in the current window");
    Ok(())
}
```

`check(rate_key, max, points, window_in_seconds)` spends `points` against `rate_key` and returns the running total, or `RateLimitError::Exceeded` once it would pass `max`. An implementation may answer from a short-lived cache, so concurrent callers can overshoot the cap by a small margin before the store catches up.

`check_uncached` has the same signature but must consult the backing store on every call, so parallel requests are counted exactly. Use it for security-sensitive counters such as verification code attempts. Its default implementation delegates to `check`; real implementations back it with a serialized store operation.

## Links

- [API reference](https://rust.haste.health/haste_rate_limit/)
- [Source](https://github.com/HasteHealth/HasteHealth/tree/main/backend/crates/rate-limit)
- [Rust SDK overview](https://haste.health/docs/api/sdks/rust)
- Implementation: [haste-repository](https://crates.io/crates/haste-repository); budgets: [haste-subscription](https://crates.io/crates/haste-subscription)

Part of [Haste Health](https://haste.health), licensed under [Apache-2.0](https://github.com/HasteHealth/HasteHealth/blob/main/LICENSE).
