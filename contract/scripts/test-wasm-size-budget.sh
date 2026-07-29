#!/usr/bin/env bash
# Unit checks for the WASM size budget process (contract/ci/wasm-size-budget.json
# + scripts/check-wasm-sizes.sh). Does not build WASM — validates the budget
# file's shape and the "justification required" rule that check-wasm-sizes.sh
# enforces at CI time, so contributors get fast feedback before a full build.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CONTRACT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
BUDGET="$CONTRACT_ROOT/ci/wasm-size-budget.json"

if ! command -v jq >/dev/null 2>&1; then
  echo "jq is required for this test"
  exit 1
fi

FAILED=0

# Every contract entry must carry a non-empty justification string.
while IFS= read -r line; do
  name="$(echo "$line" | jq -r '.key')"
  justification="$(echo "$line" | jq -r '.value.justification // empty')"
  baseline="$(echo "$line" | jq -r '.value.baseline_bytes // empty')"

  if [[ -z "$baseline" ]]; then
    echo "FAIL: $name is missing baseline_bytes"
    FAILED=1
  fi
  if [[ -z "$justification" ]]; then
    echo "FAIL: $name is missing a 'justification' string (required: PR must justify baseline bumps)"
    FAILED=1
  fi
done < <(jq -c '.contracts | to_entries[]' "$BUDGET")

if [[ "$FAILED" -ne 0 ]]; then
  echo "wasm-size-budget.json failed validation." >&2
  exit 1
fi

echo "wasm-size-budget.json: all entries have baseline_bytes and justification. OK."
exit 0
