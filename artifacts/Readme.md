# Artifacts

FHIR artifacts (profiles, search parameters, terminology, operations, test data)
shared by the backend and the frontend. Each directory under `r4/` and `r4b/` is
one FHIR package: an npm package in the frontend pnpm workspace, and a source of
embedded resources for the `haste-artifacts` crate.

## Patching upstream artifacts

Packages we maintain (e.g. `r4/hastehealth-core`) are edited directly. Packages
published by someone else (e.g. HL7's `r4/hl7-core`) are **never edited**. We keep
them exactly as published, so an upstream update can be dropped in and every
difference from upstream is recorded in one place.

Changes to upstream resources are made with patches and rules. These are applied
at build time by the `haste-artifact-patcher` crate
(`backend/crates/artifact-patcher`), and the result is written to the package's
`.min.json` files.

```
r4/hl7-core/
├── definitions/hl7/
│   ├── profiles-resources.json       upstream HL7 file, never edited
│   ├── profiles-resources.min.json   generated: upstream + patches + rules
│   └── ...
└── patches/
    ├── manifest.toml                 which sources to build, which rules to run
    └── haste-health-resource-types.json   a patch file
```

The `.min.json` files are what everything loads: the server (embedded through
`rust-embed`), Rust codegen (`backend/scripts/types_build.sh`,
`operation_build.sh`) and the frontend (`.index.json`). The upstream `.json` files
are only read by the patcher.

### Pipeline

For each source file listed in the manifest, and each resource in it (bundle
entries are handled one by one):

1. **Patches** are applied in patch file path order.
2. **Rules** run in manifest order.
3. The resource is **minimized**. For `StructureDefinition` this keeps only the
   identifying fields and the snapshot, without element mappings. This is a port
   of the TypeScript `minimize artifacts` command.

`foo.json` is written as `foo.min.json` alongside it. JSON files that aren't
resources, such as `fhir.schema.json`, are skipped. Key order is preserved, so a
rebuild only changes the files whose content changed.

### Manifest

`patches/manifest.toml`:

```toml
# Upstream files or directories, relative to this manifest.
sources = ["../definitions/hl7"]

[[rules]]
name = "clippy-doc-markdown"
# Optional: only run the rule on these resource types.
# resource_types = ["StructureDefinition"]
```

### Patch files

Every other `*.json` file under `patches/` is a patch file. It contains one or
more targeted edits, each made of standard
[JSON Patch (RFC 6902)](https://datatracker.ietf.org/doc/html/rfc6902)
operations. Paths are relative to the targeted resource, not to the bundle
containing it.

```json
{
  "description": "Why this change is needed.",
  "patches": [
    {
      "target": { "resourceType": "ValueSet", "id": "resource-types" },
      "operations": [
        {
          "op": "test",
          "path": "/compose/include/0/system",
          "value": "http://hl7.org/fhir/resource-types"
        },
        {
          "op": "add",
          "path": "/compose/include/-",
          "value": { "system": "https://haste.health/fhir/resource-types" }
        }
      ]
    }
  ]
}
```

- A `target` needs `resourceType` plus an `id`, a `url`, or both. It must match
  **exactly one** resource across the package's sources; otherwise the build fails.
  This catches patches that no longer apply after an upstream update.
- Use `test` operations to guard any path that indexes into an array. If an
  upstream update reorders things, the patch then fails instead of editing the
  wrong element.
- Keep one concern per file and say why in `description`. The file name and
  description are what reviewers see.

### Rules

Rules are transforms written in Rust, for changes that would be impractical as
targeted patches because they touch thousands of resources. They live in
`backend/crates/artifact-patcher/src/rules/` and implement the `Rule` trait:

```rust
pub trait Rule: Sync {
    fn name(&self) -> &'static str;                     // name used in manifest.toml
    fn apply(&self, resource: &mut Value) -> Vec<Edit>; // rewrite in place, report edits
}
```

To add a rule, implement the trait, add it to `RULES` in `rules/mod.rs`, and enable
it in a manifest. Every change a rule makes must be returned as an `Edit` so it
shows up in `artifacts diff`.

#### `clippy-doc-markdown`

Codegen turns `ElementDefinition.definition`, `OperationDefinition.description`
and operation parameter `documentation` into `#[doc]` attributes. This rule
rewrites those fields so the generated code passes `clippy::doc_markdown` under
`-Dclippy::pedantic`:

- Identifiers get backticks: `CamelCase`, `snake_case`, `foo::bar`, `foo()`.
- Bare URLs get angle brackets: `<http://...>`.

It parses the text as markdown and uses clippy's own word heuristics, including
its default `doc-valid-idents`. Code spans, code blocks, links and escaped text are
left alone, so running it on already-fixed text changes nothing. All of these
fields are markdown in FHIR, so the rewritten text still renders correctly.

The rule doesn't touch CodeSystem/ValueSet `display` values. Validation compares
display values, so doc lints on generated terminology have to be fixed in codegen
instead.

## Commands

See [`haste-health artifacts`](../backend/documentation/cli_commands.md#haste-health-artifacts)
in the CLI reference. Run the commands from `backend/` with `cargo run artifacts ...`.
