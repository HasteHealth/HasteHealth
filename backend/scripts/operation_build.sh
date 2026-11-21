#!/bin/bash
# Inputs, in order:
#
# 1. Operations used for backporting R4 to R5 subscriptions.
#    See https://build.fhir.org/ig/HL7/fhir-subscription-backport-ig/artifacts.html#2
#    Includes Operations like $status on Subscription.
# 2. Haste Health Custom operations. These are operations unique/exclusive to Haste Health
#    Includes operations like $scopes, $evaluate-policy and other custom operations specific to Haste Health.
# 3. Base FHIR operations. These are standard operations defined by the FHIR specification.
#    Includes standard operations like $validate, $expand, $everything, and other base FHIR operations.
# 4. SQL-on-FHIR operations includes standard SQL-on-FHIR operations.
#    See https://build.fhir.org/ig/FHIR/sql-on-fhir-v2/
#    Includes operations $viewdefinition-run and other SQL-on-FHIR operations.
cargo run generate operations \
    -i ../artifacts/r4/r5-subscription-backport/operation_definition \
    -i ../artifacts/r4/hastehealth-core/definitions/haste-health/operation \
    -i ../artifacts/r4/hl7-core/definitions/hl7/profiles-resources.min.json \
    -i ../artifacts/r4/hastehealth-core/definitions/sql-on-fhir/operations \
    -o ./crates/fhir-generated-ops/src/generated.rs
