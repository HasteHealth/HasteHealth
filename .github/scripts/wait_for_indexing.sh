#!/bin/bash
# Waits until the search worker has indexed every write in the repo database:
# each tenant's indexing position has reached its latest resource.
#
# TestScripts wait for their own writes (`--index-wait-ms`), but not for data
# loaded before them, such as `admin migrate artifacts` or the US Core
# profiles, which terminology and profile validation look up through search.
#
# usage: wait_for_indexing.sh [timeout-seconds]   (default 600)
# Reads the repo database URL from `HASTE_REPO.database_url`.
set -euo pipefail

timeout=${1:-600}
database_url=$(printenv "HASTE_REPO.database_url")

behind_query="SELECT count(*) FROM tenants t
  WHERE t.index_sequence_position_v2 <
    COALESCE((SELECT max(sequence) FROM resources r WHERE r.tenant = t.id), 0)"

for ((elapsed = 0; elapsed < timeout; elapsed += 2)); do
  behind=$(psql "$database_url" -tAc "$behind_query")
  if [[ "$behind" == "0" ]]; then
    echo "Search index caught up after ${elapsed}s."
    exit 0
  fi
  echo "Waiting for indexing: $behind tenant(s) behind..."
  sleep 2
done

echo "Search indexing did not catch up within ${timeout}s." >&2
exit 1
