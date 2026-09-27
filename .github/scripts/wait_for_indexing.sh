#!/bin/bash
# Waits until the search worker has indexed every write in the repo database:
# every project's search_index_locks position, for the active backend, has
# reached its latest resource.
#
# TestScripts wait for their own writes (`--index-wait-ms`), but not for data
# loaded before them, such as `admin migrate artifacts` or the US Core
# profiles, which terminology and profile validation look up through search.
#
# usage: wait_for_indexing.sh [timeout-seconds]   (default 600)
# Reads the repo database URL from `HASTE_REPO.database_url` and the active
# search backend from `HASTE_SEARCH.backend`.
set -euo pipefail

timeout=${1:-600}
database_url=$(printenv "HASTE_REPO.database_url")
backend=$(printenv "HASTE_SEARCH.backend")

behind_query="SELECT count(*) FROM search_index_locks l
  WHERE l.backend = :'backend'
  AND l.index_sequence_position <
    COALESCE((SELECT max(r.sequence) FROM resources r
      WHERE r.tenant = l.tenant AND r.project = l.project), 0)"

for ((elapsed = 0; elapsed < timeout; elapsed += 2)); do
  # `-v`-substituted `:'backend'` only expands for script input, not `-c`, so
  # the query is piped in on stdin rather than passed as a `-c` argument.
  behind=$(echo "$behind_query" | psql "$database_url" -v backend="$backend" -tA)
  if [[ "$behind" == "0" ]]; then
    echo "Search index caught up after ${elapsed}s."
    exit 0
  fi
  echo "Waiting for indexing: $behind project lock(s) behind..."
  sleep 2
done

echo "Search indexing did not catch up within ${timeout}s." >&2
exit 1
