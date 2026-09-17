#!/bin/bash
cargo run generate types \
    -i ../artifacts/r4/hastehealth-core/definitions/haste-health/structure_definition \
    -i ../artifacts/r4/hastehealth-core/definitions/haste-health/terminology \
    -i ../artifacts/r4/hl7-core/definitions/hl7/profiles-types.json \
    -i ../artifacts/r4/hl7-core/definitions/hl7/profiles-resources.json \
    -i ../artifacts/r4/hl7-core/definitions/hl7/valuesets.json \
    -i ../artifacts/r4/hl7-core/definitions/hl7/v3-codesystems.json \
    -i ../artifacts/r4/hastehealth-core/definitions/sql-on-fhir/definitions/ViewDefinition.json \
    -i ../artifacts/r4/hastehealth-core/definitions/sql-on-fhir/terminology \
    -o ./crates/fhir-model/src/r4/generated