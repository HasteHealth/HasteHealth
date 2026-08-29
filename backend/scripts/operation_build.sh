#!/bin/bash
cargo run generate operations \
    # Operations used for backporting R4 to R5 subscriptions. 
    # See https://build.fhir.org/ig/HL7/fhir-subscription-backport-ig/artifacts.html#2
    # Includes Operations like $status on Subscription.
    -i ./crates/artifacts/artifacts/r4/r4-to-r5-subscription-backport/operation_definition \
    # Haste Health Custom operations. These are operations unique/exclusive to Haste Health
    # Includes operations like $scopes, $evaluate-policy and other custom operations specific to Haste Health.
    -i ./crates/artifacts/artifacts/r4/haste_health/operation \
    # Base FHIR operations. These are standard operations defined by the FHIR specification.
    # Includes standard operations like $validate, $expand, $everything, and other base FHIR operations.
    -i ./crates/artifacts/artifacts/r4/hl7/original/profiles-resources.json \
    # SQL-on-FHIR operations includes standard SQL-on-FHIR operations.
    # See https://build.fhir.org/ig/FHIR/sql-on-fhir-v2/
    # Includes operations $viewdefinition-run and other SQL-on-FHIR operations.
    -i ./crates/artifacts/artifacts/universal/sql-on-fhir/operations \
    # Output file for generated operations
    -o ./crates/fhir-generated-ops/src/generated.rs