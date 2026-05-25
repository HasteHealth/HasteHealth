#!/bin/bash
cargo run generate test-scripts -i ./crates/artifacts/artifacts/r4/test_data/us_core -o ./testscripts/us-core/generated
cargo run generate test-scripts -i ./crates/artifacts/artifacts/r4/test_data/r4_r5_backport -o ./testscripts/r4-r5-backport/generated