#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
OPENAPI="${ROOT}/openapi/axiom-v1.yaml"
mkdir -p "${ROOT}/sdk/python" "${ROOT}/sdk/js"
if command -v openapi-generator-cli >/dev/null 2>&1; then
  openapi-generator-cli generate -i "$OPENAPI" -g python -o "${ROOT}/sdk/python/axiom_client"
  openapi-generator-cli generate -i "$OPENAPI" -g typescript-fetch -o "${ROOT}/sdk/js"
  echo "SDK generated"
else
  echo "openapi-generator-cli not found; wrote placeholder README only"
  echo "# Run: openapi-generator-cli generate -i openapi/axiom-v1.yaml" > "${ROOT}/sdk/python/README.md"
fi
