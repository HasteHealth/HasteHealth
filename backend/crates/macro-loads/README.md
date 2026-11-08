# haste-macro-loads

A small procedural macro, `load_artifacts!`, that embeds every `.json` file under a directory into the binary at compile time. The directory is resolved relative to the source file that invokes the macro and walked recursively, and the macro expands to a `&[&str]` with one `include_str!` per file. It was written to ship FHIR artifact packages (StructureDefinitions, SearchParameters, ValueSets) with a binary without listing the files by hand, and it works for any directory of JSON.

The Haste Health server now embeds its artifact packages with `rust-embed` in its unpublished `haste-artifacts` crate, so nothing in the workspace calls this macro today. It is published for projects that want the simpler `include_str!` approach.

## Install

```bash
cargo add haste-macro-loads
```

## Usage

```rust
// In src/lib.rs, with the JSON files under <crate>/definitions/.
static ARTIFACTS: &[&str] = haste_macro_loads::load_artifacts!("../definitions");

for json in ARTIFACTS {
    let resource: serde_json::Value = serde_json::from_str(json)?;
    // ...
}
```

The argument is one string literal naming a directory relative to the calling file. Files whose extension is not `json` are skipped. Each embedded file goes through `include_str!`, so editing one triggers a rebuild as usual; the file list itself is fixed when the macro expands, so after adding or removing a JSON file, touch the invoking source file or run a clean build.

## Links

- [API reference](https://rust.haste.health/haste_macro_loads/)
- [Source](https://github.com/HasteHealth/HasteHealth/tree/main/backend/crates/macro-loads)
- [Rust SDK overview](https://haste.health/docs/api/sdks/rust)
- The artifact packages themselves: [artifacts/](https://github.com/HasteHealth/HasteHealth/tree/main/artifacts)

Part of [Haste Health](https://haste.health), licensed under [Apache-2.0](https://github.com/HasteHealth/HasteHealth/blob/main/LICENSE).
