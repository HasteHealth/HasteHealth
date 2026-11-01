#!/bin/bash
cargo run generate test-scripts \
    -i ../artifacts/r4/test-data/testscripts/us_core \
    -o ./testscripts/us-core/9.0.0/examples/generated
# US Core 7.0.0's examples, read from the package exactly as HL7 publishes it.
mkdir -p ./testscripts/us-core/7.0.0/examples/generated
cargo run generate test-scripts \
    -i ../artifacts/r4/hl7.fhir.us.core-7.0.0/example \
    -o ./testscripts/us-core/7.0.0/examples/generated
cargo run generate test-scripts \
    -i ../artifacts/r4/test-data/testscripts/r4_r5_backport \
    -o ./testscripts/r4-r5-backport/generated