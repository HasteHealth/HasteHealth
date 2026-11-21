#!/bin/bash
cargo run generate test-scripts \
    -i ../artifacts/r4/test-data/testscripts/us_core \
    -o ./testscripts/us-core/9.0.0/examples/generated
# US Core 7.0.0's examples, read from the package exactly as HL7 publishes it. Each
# resource file is passed on its own: the example directory also holds a binary .index.db.
us_core_7_examples=()
for f in ../artifacts/r4/hl7.fhir.us.core-7.0.0/example/[A-Z]*.json; do
    us_core_7_examples+=(-i "$f")
done
mkdir -p ./testscripts/us-core/7.0.0/examples/generated
cargo run generate test-scripts \
    "${us_core_7_examples[@]}" \
    -o ./testscripts/us-core/7.0.0/examples/generated
# CARIN Blue Button 2.2.0's examples, read from the package exactly as HL7 publishes it.
carin_bb_examples=()
for f in ../artifacts/r4/hl7.fhir.us.carin-bb/example/[A-Z]*.json; do
    carin_bb_examples+=(-i "$f")
done
mkdir -p ./testscripts/carin-bb/2.2.0/examples/generated
cargo run generate test-scripts \
    "${carin_bb_examples[@]}" \
    -o ./testscripts/carin-bb/2.2.0/examples/generated
cargo run generate test-scripts \
    -i ../artifacts/r4/test-data/testscripts/r4_r5_backport \
    -o ./testscripts/r4-r5-backport/generated