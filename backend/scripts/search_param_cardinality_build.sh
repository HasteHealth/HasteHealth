#!/bin/bash
cargo run generate search-param-cardinality \
    -d ../artifacts/r4/hastehealth-core/definitions/haste-health/structure_definition \
    -d ../artifacts/r4/hl7-core/definitions/hl7/profiles-resources.min.json \
    -d ../artifacts/r4/hl7-core/definitions/hl7/profiles-types.min.json \
    -p ../artifacts/r4/hl7-core/definitions/hl7/search-parameters.min.json \
    -p ../artifacts/r4/hastehealth-core/definitions/haste-health/search_parameter \
    -o ./crates/fhir-search/src/search_parameter_cardinality.rs
