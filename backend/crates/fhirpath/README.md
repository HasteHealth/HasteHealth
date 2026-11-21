# haste-fhirpath

An async [FHIRPath](https://hl7.org/fhirpath/) engine for Rust. `FPEngine` parses an expression once (the AST is cached by expression text) and evaluates it over any values that implement `MetaValue` from [`haste-reflect`](https://crates.io/crates/haste-reflect), which includes every type in [`haste-fhir-model`](https://crates.io/crates/haste-fhir-model). The result is a `Context` you iterate as `&dyn MetaValue` items and downcast to the FHIR types you expect.

This is the engine behind Haste Health's search indexing, profile validation (constraints and slicing discriminators), SQL on FHIR views, subscription filters, access policy rules and the TestScript runner.

## Install

```bash
cargo add haste-fhirpath
```

## Usage

```rust
use haste_fhir_model::r4::generated::{
    resources::Patient,
    types::{FHIRString, HumanName},
};
use haste_fhirpath::FPEngine;

let patient = Patient {
    name: Some(vec![HumanName {
        family: Some(Box::new(FHIRString {
            value: Some("Doe".to_string()),
            ..Default::default()
        })),
        ..Default::default()
    }]),
    ..Default::default()
};

let engine = FPEngine::new();
let result = engine.evaluate("Patient.name.family", vec![&patient]).await?;

let families: Vec<&FHIRString> = result
    .iter()
    .filter_map(|v| v.as_any().downcast_ref::<FHIRString>())
    .collect();
assert_eq!(families[0].value.as_deref(), Some("Doe"));
```

`evaluate_with_config` takes a `Config` for everything outside the expression itself:

```rust
use std::{collections::HashMap, sync::Arc};

use haste_fhirpath::{Config, ExternalConstantResolver};
use haste_reflect::MetaValue;

let wanted = FHIRString { value: Some("Doe".to_string()), ..Default::default() };
let mut variables: HashMap<String, &dyn MetaValue> = HashMap::new();
variables.insert("family".to_string(), &wanted);

let config = Config::builder()
    .with_variable_resolver(ExternalConstantResolver::Variable(Arc::new(variables)));

let found = engine
    .evaluate_with_config("Patient.name.family = %family", vec![&patient], Arc::new(config))
    .await?;
assert!(!found.is_empty());
```

- `with_variable_resolver` resolves `%name` constants, either from a fixed map (an unknown name is an error) or from an async callback (`ExternalConstantResolver::Function`) that can look values up on demand.
- `with_services` plugs in a `FHIRServices` implementation for the functions that need a server: `memberOf`, `subsumes`, `subsumedBy`, `conformsTo` and `resolve`. Without it, `resolve` passes references through unchanged and the others fail.
- `with_resource_id` supplies the id that `getResourceKey()` returns for SQL on FHIR views.

## What is supported

- Navigation and filtering: paths with a leading type filter (`Patient.name`), indexers, `$this` and `$index`, `where`, `select`, `repeat` (bounded by an item limit), `ofType`, `as`, `is`, `children`, `descendants`, `iif`, `trace`.
- Collections: `empty`, `exists`, `all`, `allTrue`, `anyTrue`, `allFalse`, `anyFalse`, `count`, `distinct`, `isDistinct`, `subsetOf`, `supersetOf`, `first`, `last`, `tail`, `single`, `skip`, `take`, `union`, `combine`, `intersect`, `exclude`.
- Strings and math: `startsWith`, `endsWith`, `contains`, `upper`, `lower`, `length`, `toChars`, `indexOf`, `substring`, `replace`, `matches`, `replaceMatches`, `join`, `log`, `power`, `round`, plus the `toX` and `convertsToX` conversions (`toInteger`, `toDecimal`, `toString`, `toBoolean`, `toDate`, `toDateTime`, `toTime`, `toQuantity`).
- Operators: arithmetic (`+ - * / div mod`), string concatenation (`&`), comparison, equality and equivalence (`= != ~ !~`), `and or xor implies`, `in`, `contains`, union (`|`), `is` and `as`.
- Dates and times: `@` literals, `now()`, `today()`, `timeOfDay()`, comparison and arithmetic that respect FHIR's partial precision (a year, a year and month, or a full date), including calendar quantities such as `+ 1 year`.
- FHIR additions: `extension(url)`, `hasValue`, `getValue`, `htmlChecks`, `resolve`, `memberOf`, `subsumes`, `subsumedBy`, `conformsTo`, and the SQL on FHIR `getResourceKey` and `getReferenceKey`.

The HL7 FHIRPath test suite runs as an integration test (`tests/hl7_suite.rs`), with the cases that still fail listed in `tests/hl7/known_failures.txt`. Parse and evaluation problems surface as `FHIRPathError`.

## Links

- [API reference](https://rust.haste.health/haste_fhirpath/)
- [Source](https://github.com/HasteHealth/HasteHealth/tree/main/backend/crates/fhirpath)
- [Rust SDK overview](https://haste.health/docs/api/sdks/rust) and the [SQL on FHIR guide](https://haste.health/docs/guides/sql_on_fhir)
- Related crates: [haste-fhir-model](https://crates.io/crates/haste-fhir-model), [haste-reflect](https://crates.io/crates/haste-reflect), [haste-x-fhir-query](https://crates.io/crates/haste-x-fhir-query)
- TypeScript equivalent: [@haste-health/fhirpath](https://www.npmjs.com/package/@haste-health/fhirpath)

Part of [Haste Health](https://haste.health), licensed under [Apache-2.0](https://github.com/HasteHealth/HasteHealth/blob/main/LICENSE).
