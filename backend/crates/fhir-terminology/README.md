# haste-fhir-terminology

FHIR terminology services in Rust: `$expand` and `$validate-code` over `ValueSet` and `CodeSystem` resources, using the typed inputs and outputs of [`haste-fhir-generated-ops`](https://crates.io/crates/haste-fhir-generated-ops). The resources are fetched through a `CanonicalResolver` (from [`haste-fhir-client`](https://crates.io/crates/haste-fhir-client)), so the same code expands a value set stored in a Haste Health project, embedded in a binary or loaded from disk.

The Haste Health server answers its terminology endpoints with `FHIRCanonicalTerminology`, and [`haste-codegen`](https://crates.io/crates/haste-codegen) uses it to expand the value sets behind the generated `BoundCode` types.

## Install

```bash
cargo add haste-fhir-terminology
```

## Usage

```rust
use haste_fhir_generated_ops::generated::{ValueSetExpand, ValueSetValidateCode};
use haste_fhir_terminology::{FHIRTerminology, client::FHIRCanonicalTerminology};

let terminology = FHIRCanonicalTerminology::new();

// `resolver` implements haste_fhir_client::canonical_resolver::CanonicalResolver and
// returns ValueSet and CodeSystem resources by canonical URL. `parameters` is the
// Parameters resource a client sent to [base]/ValueSet/$validate-code.
let input = ValueSetValidateCode::Input::try_from(parameters.parameter.unwrap_or_default())?;
let output = terminology.validate(resolver.clone(), input).await?;
assert_eq!(output.result.value, Some(true));

// $expand returns the ValueSet with an `expansion` added.
let input = ValueSetExpand::Input::try_from(expand_parameters.parameter.unwrap_or_default())?;
let expanded = terminology.expand(resolver, input).await?.return_;
```

`expand` resolves the value set by `url` (or takes it inline as `valueSet`), then walks `compose.include`: inline concepts are copied, included value sets are expanded recursively, and a bare `system` pulls every concept of the resolved `CodeSystem` (content `complete`, `fragment` or `supplement`; `not-present` is an error). `validate` expands the same way and looks for the `code`.

What is not covered yet, so you can plan around it: `expand` ignores `filter`, `offset`, `count` and the other expansion options; `validate` only accepts the `code` input (not `coding` or `codeableConcept`) and matches on the code alone, without the system; and `lookup` is declared on the trait but `FHIRCanonicalTerminology` does not implement it.

## API

- `FHIRTerminology`: `expand`, `validate` and `lookup`, each taking a `CanonicalResolver` and the matching generated `Input`, and returning the generated `Output`.
- `client::FHIRCanonicalTerminology`: the implementation described above.
- `TerminologyError`: `ExpansionError`, `ValidationError`, `LookupError`, convertible into `OperationOutcomeError`.

## Links

- [API reference](https://rust.haste.health/haste_fhir_terminology/)
- [Source](https://github.com/HasteHealth/HasteHealth/tree/main/backend/crates/fhir-terminology)
- [Rust SDK overview](https://haste.health/docs/api/sdks/rust) and the [terminology REST API](https://haste.health/docs/api/rest_api/terminology/intro)
- Related: [haste-fhir-generated-ops](https://crates.io/crates/haste-fhir-generated-ops), [haste-fhir-client](https://crates.io/crates/haste-fhir-client)

Part of [Haste Health](https://haste.health), licensed under [Apache-2.0](https://github.com/HasteHealth/HasteHealth/blob/main/LICENSE).
