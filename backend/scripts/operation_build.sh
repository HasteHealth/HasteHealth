#!/bin/bash
cargo run generate operations \
    -i ./crates/artifacts/artifacts/r4/r4-to-r5-subscription-backport/operation_definition \
    -i ./crates/artifacts/artifacts/r4/haste_health/operation \
    -i ./crates/artifacts/artifacts/r4/hl7/original/profiles-resources.json \
    -i ./crates/artifacts/artifacts/universal/sql-on-fhir/operations \
    -o ./crates/fhir-generated-ops/src/generated.rs