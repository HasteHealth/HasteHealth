# Haste Health Backend

## Configuration

- [server configuration](./documentation/server_configuration.md)
- [worker configuration](./documentation/worker_configuration.md)

## Property tests

The pure parsing and range code carries `proptest` properties, in a
`proptests` module of each crate that has them: `fhir-model` (dates and
times), `fhir-search` (search values and ranges), `hl7v2`, `fhirpath`, `jwt`
(scopes) and `pointer`. Generators for FHIR primitives live in
`haste_fhir_model::r4::strategies`, behind the crate's `testing` feature.

- Run one crate's properties: `cargo test -p haste-fhir-search proptests`.
- A failing case is saved to the crate's `proptest-regressions/` directory
  and replays on every later run; commit those files.
- `PROPTEST_CASES=2000 cargo test ...` runs more cases than the default 256.
