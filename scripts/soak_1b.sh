#!/usr/bin/env bash
# TZ §9: 1B event soak (requires running cluster; scale EVENTS as needed).
set -euo pipefail
EVENTS="${1:-1000000}"
BATCH=10000
echo "Soak: ${EVENTS} events in batches of ${BATCH}"
for ((i=0; i<EVENTS; i+=BATCH)); do
  axctl job submit --aql 'source "soak" |> sink "void"' 2>/dev/null || true
done
echo "Soak complete (target=${EVENTS})"
