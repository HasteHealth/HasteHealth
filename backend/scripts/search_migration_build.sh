#!/bin/bash
# Migrates the configured PG search database (haste.toml or HASTE_SEARCH.*),
# then writes a new migration creating whatever the search schema needs that
# the database still lacks; nothing when it lacks nothing. Run after anything
# that changes the search parameters or their cardinality (e.g.
# scripts/search_param_cardinality_build.sh), then rebuild.
cargo run generate search-migration -d ./crates/fhir-search/pg-migrations "$@"
