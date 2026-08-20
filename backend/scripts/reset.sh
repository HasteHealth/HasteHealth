#!/bin/bash

echo "Resetting haste_health database..."
dropdb haste_health
createdb haste_health
echo "Deleting r4_search_index from Elasticsearch..."
curl -u "elastic:SZxWWFbG"  -k http://localhost:9200/r4_search_index/ -XDELETE -H 'Content-Type: application/json'
echo "Build schemas and artifacts..."
cargo run admin migrate all
echo "Creating tenant..."
cargo run admin tenant create --id=my-health '--owner-email=myuser@health.org' --owner-password=testing_password --subscription-tier=unlimited
echo "Reset complete."