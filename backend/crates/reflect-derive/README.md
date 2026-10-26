# haste-reflect-derive

The procedural macro behind [`haste-reflect`](https://crates.io/crates/haste-reflect): `#[derive(Reflect)]` implements the `MetaValue` trait for a struct or an enum. For a struct it generates `fields()`, `get_field` and `get_field_mut` over the named fields, unwrapping `Option` fields and returning `None` when they are unset. For an enum whose variants each wrap one value, such as a FHIR `Resource` or a `value[x]` choice type, every method delegates to the active variant.

You do not normally depend on this crate directly. Enable the `derive` feature of `haste-reflect` and import `Reflect` from its `derive` module, which keeps the macro and the trait at matching versions.

## Install

```bash
cargo add haste-reflect --features derive
```

## Usage

```rust
use haste_reflect::derive::Reflect;

#[derive(Reflect, Debug)]
#[fhir_type = "Coding"]
struct Coding {
    system: Option<String>,
    code: Option<String>,
    #[rename_field = "userSelected"]
    user_selected: Option<bool>,
}
```

Two attributes are recognised: `fhir_type` on the type sets what `fhir_type()` returns, and `rename_field` on a field sets the name `get_field` answers to when the Rust identifier cannot carry it.

## Links

- [API reference](https://rust.haste.health/haste_reflect_derive/)
- [Source](https://github.com/HasteHealth/HasteHealth/tree/main/backend/crates/reflect-derive)
- [Rust SDK overview](https://haste.health/docs/api/sdks/rust)
- Parent crate: [haste-reflect](https://crates.io/crates/haste-reflect)

Part of [Haste Health](https://haste.health), licensed under [Apache-2.0](https://github.com/HasteHealth/HasteHealth/blob/main/LICENSE).
