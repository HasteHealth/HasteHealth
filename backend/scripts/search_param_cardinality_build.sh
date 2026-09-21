#!/bin/bash
cargo run generate search-param-cardinality \
    -d ../artifacts/r4/hl7-core/definitions/hl7/profiles-resources.min.json \
    -d ../artifacts/r4/hl7-core/definitions/hl7/profiles-types.min.json \
    -p ../artifacts/r4/hl7-core/definitions/hl7/search-parameters.min.json \
    -o ./crates/fhir-search/src/search_parameter_cardinality.rs
