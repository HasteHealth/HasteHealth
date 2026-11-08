# haste-pointer

Pointers into FHIR values, built on `MetaValue` from [`haste-reflect`](https://crates.io/crates/haste-reflect). A `Path` is a JSON Pointer style string such as `/name/0/family/value`: extend it with `descend`, split it with `ascend` into the parent path and the last `Key`, and resolve it against any `&dyn MetaValue` with `get` or `get_typed::<T>`. Field names containing `/` or `~` are escaped the JSON Pointer way (`~1` and `~0`). A `TypedPointer<Root, T>` pairs a shared root (`Arc<Root>`) with a position inside it: it knows its path and its current value, and `descend::<Child>(&Key)` only moves when the child really has that type.

[`haste-fhir-profiling`](https://crates.io/crates/haste-fhir-profiling) reports validation failures at a `Path`, [`haste-testscript-runner`](https://crates.io/crates/haste-testscript-runner) walks fixtures with a `TypedPointer`, and the [`haste-access-control`](https://crates.io/crates/haste-access-control) rule engine uses one to address the rules inside an `AccessPolicyV2`.

## Install

```bash
cargo add haste-pointer
```

## Usage

```rust
use std::sync::Arc;

use haste_fhir_model::r4::generated::{
    resources::Patient,
    types::{FHIRString, HumanName},
};
use haste_pointer::{Key, Path, TypedPointer};

let patient = Arc::new(Patient {
    name: Some(vec![HumanName {
        family: Some(Box::new(FHIRString {
            value: Some("Doe".to_string()),
            ..Default::default()
        })),
        ..Default::default()
    }]),
    ..Default::default()
});

// Untyped: build a path and resolve it against any MetaValue.
let path = Path::new().descend("name").descend("0").descend("family").descend("value");
assert_eq!(path.to_string(), "/name/0/family/value");
assert_eq!(path.get_typed::<String>(patient.as_ref()), Some(&"Doe".to_string()));

// Typed: each step names the type it expects and answers None otherwise.
let family = TypedPointer::<Patient, Patient>::new(patient)
    .descend::<Vec<HumanName>>(&Key::Field("name".to_string()))
    .and_then(|p| p.descend::<HumanName>(&Key::Index(0)))
    .and_then(|p| p.descend::<Box<FHIRString>>(&Key::Field("family".to_string())))
    .unwrap();
assert_eq!(family.path(), "/name/0/family");
assert_eq!(family.value().and_then(|f| f.value.as_deref()), Some("Doe"));
```

`Key::parse` reads a segment as `Key::Index` when it is a number and `Key::Field` otherwise, and `TypedPointer::root()` returns to the top of the resource.

## Links

- [API reference](https://rust.haste.health/haste_pointer/)
- [Source](https://github.com/HasteHealth/HasteHealth/tree/main/backend/crates/pointer)
- [Rust SDK overview](https://haste.health/docs/api/sdks/rust)
- Related crates: [haste-reflect](https://crates.io/crates/haste-reflect), [haste-fhir-model](https://crates.io/crates/haste-fhir-model)
- TypeScript equivalent: [@haste-health/fhir-pointer](https://www.npmjs.com/package/@haste-health/fhir-pointer)

Part of [Haste Health](https://haste.health), licensed under [Apache-2.0](https://github.com/HasteHealth/HasteHealth/blob/main/LICENSE).
