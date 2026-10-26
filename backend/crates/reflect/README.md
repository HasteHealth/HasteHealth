# haste-reflect

Runtime reflection for FHIR values. The `MetaValue` trait lets code walk a value without knowing its Rust type: list its fields, fetch a field or an index as another `&dyn MetaValue`, flatten collections, ask for its FHIR type name, tell single values from repeating ones, and downcast to the concrete type through `as_any`. The primitives (`i64`, `u64`, `f64`, `bool`, `String`, `&'static str`), `Vec<T>`, `Option<T>` and `Box<T>` implement it out of the box, and the `Reflect` derive (feature `derive`) implements it for your own structs and enums.

This is the trait the rest of Haste Health evaluates against. Every type in [`haste-fhir-model`](https://crates.io/crates/haste-fhir-model) derives `Reflect`, and [`haste-fhirpath`](https://crates.io/crates/haste-fhirpath), [`haste-pointer`](https://crates.io/crates/haste-pointer), [`haste-fhir-profiling`](https://crates.io/crates/haste-fhir-profiling), [`haste-access-control`](https://crates.io/crates/haste-access-control) and the search indexer all take `&dyn MetaValue` rather than concrete resources.

## Install

```bash
cargo add haste-reflect --features derive
```

## Usage

```rust
use haste_reflect::{MetaValue, derive::Reflect};

#[derive(Reflect, Debug)]
#[fhir_type = "HumanName"]
struct Name {
    family: Option<String>,
    given: Vec<String>,
}

let name = Name {
    family: Some("Doe".to_string()),
    given: vec!["Eve".to_string(), "A.".to_string()],
};

assert_eq!(name.fhir_type(), "HumanName");
assert_eq!(name.fields(), vec!["family", "given"]);

// Optional fields come back unwrapped, or None when unset.
let family = name.get_field("family").unwrap();
assert_eq!(family.as_any().downcast_ref::<String>(), Some(&"Doe".to_string()));

// Repeating fields are indexed.
let given = name.get_field("given").unwrap();
let second = given.get_index(1).and_then(|v| v.as_any().downcast_ref::<String>());
assert_eq!(second, Some(&"A.".to_string()));
```

`#[fhir_type = "..."]` sets what `fhir_type()` returns and `#[rename_field = "..."]` sets the name `get_field` answers to when the Rust field cannot carry it (`use_` for `use`). On an enum whose variants each wrap one value, such as a `Resource` or a `value[x]` choice, the derive delegates every method to the active variant.

## Features

| Feature  | What it enables                                                         |
| -------- | ----------------------------------------------------------------------- |
| `derive` | The `derive` module, re-exporting `Reflect` from `haste-reflect-derive` |

## Links

- [API reference](https://rust.haste.health/haste_reflect/)
- [Source](https://github.com/HasteHealth/HasteHealth/tree/main/backend/crates/reflect)
- [Rust SDK overview](https://haste.health/docs/api/sdks/rust)
- Related crates: [haste-reflect-derive](https://crates.io/crates/haste-reflect-derive), [haste-fhir-model](https://crates.io/crates/haste-fhir-model), [haste-pointer](https://crates.io/crates/haste-pointer)
- TypeScript equivalent: [@haste-health/meta-value](https://www.npmjs.com/package/@haste-health/meta-value)

Part of [Haste Health](https://haste.health), licensed under [Apache-2.0](https://github.com/HasteHealth/HasteHealth/blob/main/LICENSE).
