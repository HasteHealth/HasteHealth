#!/bin/bash
cargo run generate types \
    -i ./crates/artifacts/artifacts/r4/haste_health/structure_definition \
    -i ./crates/artifacts/artifacts/r4/haste_health/terminology \
    -i ./crates/artifacts/artifacts/r4/hl7/original/profiles-types.json \
    -i ./crates/artifacts/artifacts/r4/hl7/original/profiles-resources.json \
    -i ./crates/artifacts/artifacts/r4/hl7/original/valuesets.json \
    -i ./crates/artifacts/artifacts/r4/hl7/original/v3-codesystems.json \
    -i ./crates/artifacts/artifacts/universal/sql-on-fhir/definitions/ViewDefinition.json \
    -i ./crates/artifacts/artifacts/universal/sql-on-fhir/terminology \
    -o ./crates/fhir-model/src/r4/generated