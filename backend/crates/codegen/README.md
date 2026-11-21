# haste-codegen

The code generators behind Haste Health's FHIR model. From FHIR `StructureDefinition`, `OperationDefinition` and `SearchParameter` resources it produces the Rust types in [`haste-fhir-model`](https://crates.io/crates/haste-fhir-model), the typed operation bindings in [`haste-fhir-generated-ops`](https://crates.io/crates/haste-fhir-generated-ops), the single-valued search parameter table the server's search index relies on, and FHIR `TestScript` resources from example data. Rust output is a `proc_macro2::TokenStream`; generated doc comments are rewritten so they pass clippy's `doc_markdown` lint under `-Dclippy::pedantic`.

It is a library. The `haste-health generate` commands drive it, and the scripts in `backend/scripts` (`types_build.sh`, `operation_build.sh`, `search_param_cardinality_build.sh`, `testscript_build.sh`) run those commands with the repository's artifact inputs. [`haste-fhir-profiling`](https://crates.io/crates/haste-fhir-profiling) reuses its `StructureDefinition` traversal.

## Install

```bash
cargo add haste-codegen
```

## Usage

Generate the Rust types for a set of StructureDefinition files or directories:

```rust
use haste_codegen::type_gen::rust_types;

let inputs = vec![
    "../artifacts/r4/hl7-core/definitions/hl7/profiles-types.min.json".to_string(),
    "../artifacts/r4/hl7-core/definitions/hl7/profiles-resources.min.json".to_string(),
    "../artifacts/r4/hl7-core/definitions/hl7/valuesets.min.json".to_string(),
];

// `None` generates every tier; "primitive-type", "complex-type" or "resource" restricts it.
let generated = rust_types::generate(&inputs, None).await?;

std::fs::write("resources.rs", generated.resources.to_string())?;
std::fs::write("types.rs", generated.types.to_string())?;
std::fs::write("terminology.rs", generated.terminology.to_string())?;
```

Walk a StructureDefinition snapshot bottom-up, children before their parent:

```rust
use haste_codegen::traversal::traversal;

let element_count = traversal(&structure_definition, &mut |_element, child_counts: Vec<usize>, _index| {
    child_counts.iter().sum::<usize>() + 1
})?;
```

## API

| Module                            | What it does                                                                                                                                                                 |
| --------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `type_gen::rust_types`            | `generate(files, level)`: structs and enums for resources, data types and terminology, as `GeneratedTypes`.                                                                  |
| `type_gen::operation_definitions` | `generate_operation_definitions_from_files(files)`: typed `Input` and `Output` bindings for each `OperationDefinition`.                                                      |
| `search_param_cardinality`        | Resolves each `SearchParameter` expression against the snapshots to decide whether it can yield more than one value; `generate_lookup` emits the `is_single_valued` table.   |
| `testscript_gen`                  | `generate_testscripts(files)`: for each file of example resources, a TestScript that creates them as fixtures, asserts the created type and deletes them by tag in teardown. |
| `traversal`                       | `traversal` and `ele_index_to_child_indices` for walking snapshot elements.                                                                                                  |
| `utilities`                       | FHIR to Rust name and type conversion, element extraction, and loading of definition files (`utilities::load`).                                                              |

## Links

- [API reference](https://rust.haste.health/haste_codegen/)
- [Source](https://github.com/HasteHealth/HasteHealth/tree/main/backend/crates/codegen)
- [Rust SDK overview](https://haste.health/docs/api/sdks/rust)
- [CLI reference](https://github.com/HasteHealth/HasteHealth/blob/main/backend/documentation/cli_commands.md#haste-health-generate) for the `generate` commands
- TypeScript equivalent: [@haste-health/codegen](https://www.npmjs.com/package/@haste-health/codegen)

Part of [Haste Health](https://haste.health), licensed under [Apache-2.0](https://github.com/HasteHealth/HasteHealth/blob/main/LICENSE).
