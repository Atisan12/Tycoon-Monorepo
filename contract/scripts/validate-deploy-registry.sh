#!/usr/bin/env bash
# =============================================================================
# validate-deploy-registry.sh — Validate Deployed Contract Registry
#
# Usage:
#   ./scripts/validate-deploy-registry.sh testnet
#   ./scripts/validate-deploy-registry.sh mainnet --verbose
#
# Validates entries in deployed-contracts-*.txt files for correctness and
# consistency. Checks contract IDs, WASM hashes, and timestamp formats.
# =============================================================================

set -euo pipefail

# ─── Defaults ─────────────────────────────────────────────────────────────────
NETWORK="${1:-}"
VERBOSE="${VERBOSE:-false}"

# ─── Colours ──────────────────────────────────────────────────────────────────
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

info()    { echo -e "${GREEN}[INFO]${NC}  $*"; }
warn()    { echo -e "${YELLOW}[WARN]${NC}  $*"; }
error()   { echo -e "${RED}[ERROR]${NC} $*"; exit 1; }
debug()   { [[ "$VERBOSE" == "true" ]] && echo -e "${BLUE}[DEBUG]${NC} $*"; }

# ─── Argument validation ──────────────────────────────────────────────────────
[[ -z "$NETWORK" ]] && error "Usage: $0 <network> [--verbose]\nExample: $0 testnet"

case "$2" in
  --verbose) VERBOSE="true" ;;
  "") ;;
  *) warn "Unknown option: $2" ;;
esac

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REGISTRY_FILE="$SCRIPT_DIR/../deploy/deployed-contracts-${NETWORK}.txt"
VALID_CONTRACTS=("tycoon_token" "tycoon_reward_system" "tycoon_collectibles" "tycoon_boost_system" "tycoon_game" "tycoon_main_game")

[[ ! -f "$REGISTRY_FILE" ]] && error "Registry file not found: $REGISTRY_FILE"

info "Validating deployed contracts registry: $REGISTRY_FILE"

# ─── Validation functions ─────────────────────────────────────────────────────

is_valid_contract_id() {
  local id="$1"
  # Soroban contract IDs are 56 characters, alphanumeric (base32 encoded)
  [[ "$id" =~ ^[A-Z0-9]{56}$ ]] && return 0 || return 1
}

is_valid_wasm_hash() {
  local hash="$1"
  # WASM hashes are SHA-256: 64 hex chars, lowercase
  [[ "$hash" =~ ^[a-f0-9]{64}$ ]] && return 0 || return 1
}

is_valid_timestamp() {
  local ts="$1"
  # ISO 8601 format: YYYY-MM-DDTHH:MM:SSZ
  [[ "$ts" =~ ^[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}Z$ ]] && return 0 || return 1
}

is_valid_contract_name() {
  local name="$1"
  for contract in "${VALID_CONTRACTS[@]}"; do
    [[ "$name" == "$contract" ]] && return 0
  done
  return 1
}

# ─── Main validation ──────────────────────────────────────────────────────────

ERRORS=0
WARNINGS=0
ENTRIES=0

while IFS= read -r line; do
  # Skip empty lines and comments
  [[ -z "$line" || "$line" =~ ^# ]] && continue

  ((ENTRIES++))

  # Parse entry
  read -r CONTRACT_NAME CONTRACT_ID WASM_HASH TIMESTAMP <<< "$line"

  debug "Entry $ENTRIES: $CONTRACT_NAME | $CONTRACT_ID | $WASM_HASH | $TIMESTAMP"

  # Validate contract name
  if ! is_valid_contract_name "$CONTRACT_NAME"; then
    echo -e "${RED}✗ Entry $ENTRIES: Invalid contract name '$CONTRACT_NAME'${NC}"
    echo "  Valid names: ${VALID_CONTRACTS[*]}"
    ((ERRORS++))
  fi

  # Validate contract ID
  if ! is_valid_contract_id "$CONTRACT_ID"; then
    echo -e "${RED}✗ Entry $ENTRIES: Invalid contract ID format '$CONTRACT_ID'${NC}"
    echo "  Expected: 56 alphanumeric characters"
    ((ERRORS++))
  fi

  # Validate WASM hash
  if ! is_valid_wasm_hash "$WASM_HASH"; then
    echo -e "${RED}✗ Entry $ENTRIES: Invalid WASM hash format '$WASM_HASH'${NC}"
    echo "  Expected: 64 lowercase hex characters (SHA-256)"
    ((ERRORS++))
  fi

  # Validate timestamp
  if ! is_valid_timestamp "$TIMESTAMP"; then
    echo -e "${RED}✗ Entry $ENTRIES: Invalid timestamp format '$TIMESTAMP'${NC}"
    echo "  Expected: YYYY-MM-DDTHH:MM:SSZ (ISO 8601 UTC)"
    ((ERRORS++))
  fi

  [[ "$VERBOSE" == "true" ]] && echo -e "${GREEN}✓ Entry $ENTRIES: $CONTRACT_NAME${NC}"

done < "$REGISTRY_FILE"

# ─── Summary ──────────────────────────────────────────────────────────────────

echo ""
if [[ $ERRORS -eq 0 ]]; then
  info "✅ Validation passed: $ENTRIES entries checked, 0 errors"
  exit 0
else
  error "❌ Validation failed: $ENTRIES entries checked, $ERRORS errors found"
fi
