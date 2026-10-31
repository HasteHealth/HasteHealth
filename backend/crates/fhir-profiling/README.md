# haste-fhir-profiling

Validates a FHIR resource against a profile: a `StructureDefinition` with `derivation = constraint`, such as the US Core profiles. The result is an `OperationOutcome` whose issues name the profile, the element and what failed; an empty issue list means the resource conforms.

The Haste Health server runs it on every create and update for each canonical URL in `meta.profile`, and answers `422 Unprocessable Entity` with the outcome when there are issues.

## Install

```bash
cargo add haste-fhir-profiling
```

## Usage

Profiles are fetched through a `CanonicalResolver` from [`haste-fhir-client`](https://crates.io/crates/haste-fhir-client), so the crate works over any store that can hand back a `StructureDefinition` by URL. This one serves them from a map:

```rust
use std::collections::HashMap;
use std::sync::Arc;

use haste_fhir_client::canonical_resolver::CanonicalResolver;
use haste_fhir_model::r4::generated::resources::{Resource, ResourceType};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_profiling::{FHIRProfileArguments, validate_profile_by_url};

struct Profiles(HashMap<String, Arc<Resource>>);

impl CanonicalResolver for Profiles {
    fn resolve(
        &self,
        _resource_type: ResourceType,
        canonical_url: &str,
    ) -> impl Future<Output = Result<Option<Arc<Resource>>, OperationOutcomeError>> + Send {
        std::future::ready(Ok(self.0.get(canonical_url).cloned()))
    }
}

let outcome = validate_profile_by_url(
    FHIRProfileArguments::new(Arc::new(profiles)),
    "http://hl7.org/fhir/us/core/StructureDefinition/us-core-patient",
    &patient,
)
.await?;

for issue in &outcome.issue {
    println!("{}", issue.diagnostics.as_ref().and_then(|d| d.value.as_deref()).unwrap_or(""));
}
```

If the resolver does not know the URL, the outcome is empty and a warning is logged. When you already hold the `StructureDefinition`, build a `FHIRProfileCTX` and call `validate_profile` instead.

## What is checked

- Cardinality (`min` and `max`) of every element in the snapshot.
- `fixed[x]` and `pattern[x]` values.
- The element's allowed types, and any profiles those types name, which are resolved and validated recursively (extensions such as `us-core-race` are validated this way).
- Slicing with `value`, `pattern`, `type` and `exists` discriminators, including the slice's own cardinality. `profile` and `null` discriminators return a `not-supported` error.

Terminology bindings, FHIRPath invariants and the `maxLength`, `minValue` and `maxValue` constraints are not evaluated yet. The tests in `tests/slicing.rs` run the US Core profiles from `artifacts/r4/hl7.fhir.us.core`.

## Links

- [API reference](https://rust.haste.health/haste_fhir_profiling/)
- [Source](https://github.com/HasteHealth/HasteHealth/tree/main/backend/crates/fhir-profiling)
- [Rust SDK overview](https://haste.health/docs/api/sdks/rust) and the [profiling and validation guide](https://haste.health/docs/guides/profiling)
- Related: [haste-fhir-client](https://crates.io/crates/haste-fhir-client) for `CanonicalResolver`, [haste-fhirpath](https://crates.io/crates/haste-fhirpath) for discriminator paths
- TypeScript equivalent: [@haste-health/fhir-validation](https://www.npmjs.com/package/@haste-health/fhir-validation)

Part of [Haste Health](https://haste.health), licensed under [Apache-2.0](https://github.com/HasteHealth/HasteHealth/blob/main/LICENSE).
