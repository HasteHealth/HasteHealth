#!/bin/bash
cargo run generate test-scripts \
    -i ../artifacts/r4/test-data/testscripts/us_core \
    -o ./testscripts/us-core/generated
cargo run generate test-scripts \
    -i ../artifacts/r4/test-data/testscripts/r4_r5_backport \
    -o ./testscripts/r4-r5-backport/generated