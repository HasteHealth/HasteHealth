# haste-fhir-model

The FHIR R4 data model Haste Health is built on: a Rust struct for every R4 resource, data type and backbone element, a bound code type for every required value set, and the `Resource` and `ResourceType` enums that tie them together. The types are generated from the HL7 R4 StructureDefinitions in the repository's `artifacts/` directory (`bash scripts/types_build.sh` from `backend/`, never edited by hand), so field names, cardinalities, choice types, reference targets and documentation follow the specification. Next to the standard resources it defines the resources the Haste Health server manages: `User`, `Membership`, `Project`, `ClientApplication`, `IdentityProvider`, `AccessPolicyV2` and the SQL on FHIR `ViewDefinition`.

Every other Haste Health crate, from the FHIR client and the FHIRPath engine to the repository and the server, builds on this one. JSON serialization follows the FHIR rules rather than plain serde (see [`haste-fhir-serialization-json`](https://crates.io/crates/haste-fhir-serialization-json)), and every type implements `MetaValue` from [`haste-reflect`](https://crates.io/crates/haste-reflect) so it can be walked without knowing its concrete type.

## Install

```bash
cargo add haste-fhir-model
```

## Usage

```rust
use haste_fhir_model::r4::generated::{
    resources::{Patient, Resource},
    terminology::AdministrativeGender,
    types::{FHIRString, HumanName},
};

// Every field is optional and primitives carry id and extension next to
// their value, so `..Default::default()` keeps construction short.
let patient = Patient {
    id: Some("example".to_string()),
    gender: Some(AdministrativeGender::male()),
    name: Some(vec![HumanName {
        family: Some(Box::new(FHIRString {
            value: Some("Doe".to_string()),
            ..Default::default()
        })),
        ..Default::default()
    }]),
    ..Default::default()
};

// FHIR JSON in and out; `resourceType` picks the struct.
let json = serde_json::to_string(&patient)?;
let resource: Resource = serde_json::from_str(&json)?;
assert!(matches!(resource, Resource::Patient(_)));
```

Deserialization validates as it parses. Unknown fields, malformed `_field` extension entries, out of range primitives (a negative `positiveInt`), cardinality violations and references to a type the element does not allow are all errors.

## Modules

| Module                       | Contents                                                                                                                                                               |
| ---------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `r4::generated::resources`   | One struct per resource and backbone element, the `Resource` and `ResourceType` enums, and a `*TypeChoice` enum for every `value[x]` element                           |
| `r4::generated::types`       | Primitives (`FHIRString`, `FHIRDate`, `FHIRDecimal`, ...) with `id`, `extension` and `value`, and the complex types (`HumanName`, `CodeableConcept`, `Reference`, ...) |
| `r4::generated::terminology` | `BoundCode<VS>` for required bindings, with one `ValueSetDef` type per value set and a constructor per code, such as `AdministrativeGender::male()`                    |
| `r4::datetime`               | `Date`, `DateTime`, `Instant` and `Time` that keep FHIR's partial precision (`Year`, `YearMonth`, `YearMonthDay`, full ISO 8601), their parsers, and chrono conversion |
| `r4::conversion`             | Downcasts from a `MetaValue` to `bool`, `String`, `f64` or a datetime string, and the sets of FHIR type names each one covers                                          |
| `r4::sqlx`                   | `FHIRJson<T>` and `FHIRJsonRef<T>`, which read and write any model type as a PostgreSQL `jsonb` column, and `ResourceType` as a column type (feature `sqlx`)           |
| `r4::strategies`             | `proptest` generators for FHIR primitives (feature `testing`)                                                                                                          |

## Features

| Feature   | What it enables                                                                                   |
| --------- | ------------------------------------------------------------------------------------------------- |
| `sqlx`    | The `r4::sqlx` module: PostgreSQL `jsonb` encode and decode for the model types                   |
| `testing` | The `r4::strategies` module, so other crates can use the primitive generators in dev-dependencies |

## Links

- [API reference](https://rust.haste.health/haste_fhir_model/)
- [Source](https://github.com/HasteHealth/HasteHealth/tree/main/backend/crates/fhir-model)
- [Rust SDK overview](https://haste.health/docs/api/sdks/rust), the [data model](https://haste.health/docs/core_concepts/data_model) and the per resource reference, for example [Patient](https://haste.health/docs/reference/fhir/model/resources/Patient)
- Related crates: [haste-fhir-serialization-json](https://crates.io/crates/haste-fhir-serialization-json), [haste-reflect](https://crates.io/crates/haste-reflect), [haste-fhirpath](https://crates.io/crates/haste-fhirpath)
- TypeScript equivalent: [@haste-health/fhir-types](https://www.npmjs.com/package/@haste-health/fhir-types)

Part of [Haste Health](https://haste.health), licensed under [Apache-2.0](https://github.com/HasteHealth/HasteHealth/blob/main/LICENSE).
