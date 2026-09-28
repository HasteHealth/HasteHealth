#!/bin/bash

echo "Resetting haste_health database..."
dropdb haste_health
createdb haste_health
echo "Resetting haste_health_search database..."
dropdb --if-exists haste_health_search
createdb haste_health_search
echo "Build schemas and artifacts..."
cargo run admin migrate all
echo "Creating tenant..."
cargo run admin tenant create --id=my-health '--owner-email=myuser@health.org' --owner-password=testing_password --subscription-tier=unlimited
echo "Reset complete."
