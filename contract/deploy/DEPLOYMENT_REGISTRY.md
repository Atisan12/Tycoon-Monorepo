# Deployment Registry — Maintenance Guide

## Overview

The Tycoon contract deployment registry tracks all contracts deployed to Stellar testnet and mainnet environments. Registry files are immutable audit logs that serve as the source of truth for deployed contract addresses and verification hashes.

## Registry Files

- **`deployed-contracts-testnet.txt`** — Testnet deployments (can be re-deployed)
- **`deployed-contracts-mainnet.txt`** — Mainnet deployments (permanent records)

## Entry Format

```
<contract_name> <contract_id> <wasm_hash> <deployed_at_iso8601>
```

### Field Specifications

| Field | Format | Example | Rules |
|-------|--------|---------|-------|
| `contract_name` | Underscore-separated | `tycoon_token` | Must match one of the canonical contract names |
| `contract_id` | Base32 (56 chars) | `CAVC5WDKFWT3N3K4K5L6M7N8O9P0Q1R2S3T4U5V6W7X8Y9Z0A1B2C3D4E5F` | Valid Soroban contract ID |
| `wasm_hash` | SHA-256 hex (64 chars) | `abc123def456...` | Lowercase hexadecimal, from build artifact |
| `deployed_at_iso8601` | UTC timestamp | `2024-07-23T15:30:45Z` | ISO 8601 format, UTC timezone |

## Canonical Contract Names

The following contract names are valid in the registry:

- `tycoon_token`
- `tycoon_reward_system`
- `tycoon_collectibles`
- `tycoon_boost_system`
- `tycoon_game`
- `tycoon_main_game`

Any other name will fail validation.

## Validation

### Automated Validation

Run the validation script to check for inconsistencies:

```bash
# Validate testnet registry
./scripts/validate-deploy-registry.sh testnet

# Validate mainnet registry
./scripts/validate-deploy-registry.sh mainnet

# Verbose output
./scripts/validate-deploy-registry.sh testnet --verbose
```

The validator checks:
- Contract name is in the canonical list
- Contract ID format (56 alphanumeric characters)
- WASM hash format (64 lowercase hex characters)
- Timestamp format (ISO 8601 UTC)

### Manual Validation Checklist

Before committing registry changes:

- [ ] Each entry has exactly 4 fields (name, ID, hash, timestamp)
- [ ] No leading/trailing whitespace on values
- [ ] All contract IDs start with `C` (Soroban addresses)
- [ ] All WASM hashes use lowercase hex only
- [ ] All timestamps end with `Z` (UTC indicator)
- [ ] Entry order is chronological (newest deployments last)

## Adding Entries

**Automated (Recommended)**

Entries are added automatically by `scripts/deploy.sh` during deployment:

```bash
DEPLOYER_ACCOUNT=GXXX... ./scripts/deploy.sh --network testnet --contract tycoon-token
```

The deploy script appends a new entry in the correct format.

**Manual (Not Recommended)**

If manual entry is necessary (e.g., correcting audit records):

1. Ensure the deployment is complete and verified
2. Gather: contract name, ID, WASM hash, deployment timestamp
3. Append a new line to the appropriate registry file
4. Validate with `./scripts/validate-deploy-registry.sh`
5. Submit a PR with validation output

## Removing/Editing Entries

**Testnet**: Entries should be removed only in rare cases (e.g., duplicate entries, broken deployments). Each removal must include justification in the PR description.

**Mainnet**: **Entries are permanent and immutable.** No removal or editing is permitted. If an error is discovered:
- Document the error in a separate audit note
- Plan for a corrected redeployment with a new entry
- Never modify existing entries

## Stale Entry Management

### Testnet

Deployments older than 90 days without activity should be reviewed quarterly:

1. Check recent deployment activity in CI/CD logs
2. If no activity, decide whether to:
   - **Keep**: Document why (e.g., stable baseline for regression testing)
   - **Remove**: Archive as historical reference, remove from active registry

### Mainnet

All entries are kept indefinitely for compliance and audit trail purposes.

## Audit Trail

The registry serves as an immutable audit trail. Each entry proves:
- **When** a contract was deployed (timestamp)
- **Which** contract was deployed (name)
- **Where** it was deployed (contract ID)
- **What** bytecode was deployed (WASM hash)

This enables verification that the live network is running exactly the contracts intended.

## Common Tasks

### Verify a Deployed Contract

```bash
# 1. Find the entry in the registry
grep "tycoon_token" deployed-contracts-testnet.txt

# 2. Record the contract ID and WASM hash
# Example: tycoon_token CAVC5... abc123... 2024-07-23T15:30:45Z

# 3. Verify the contract is live on the network
stellar contract read --contract-id CAVC5... --rpc-url https://soroban-testnet.stellar.org

# 4. Cross-check the WASM hash
# - Download the contract bytecode from the network
# - Compute SHA-256: sha256sum <bytecode>
# - Compare with the hash in the registry
```

### Compare Testnet and Mainnet

```bash
# Show contracts deployed to both environments
diff <(cut -d' ' -f1 deployed-contracts-testnet.txt | grep -v '^#') \
     <(cut -d' ' -f1 deployed-contracts-mainnet.txt | grep -v '^#') | sort
```

### Export for Monitoring/Alerts

```bash
# List all testnet contract IDs
awk 'NR>6 && NF>0 {print $2}' deployed-contracts-testnet.txt | sort -u

# List all contract names with deployment dates
awk 'NR>6 && NF>0 {print $1, $4}' deployed-contracts-testnet.txt | sort -k2 -r
```

## Security Considerations

- **Read-only in production**: The registry should be read-only in production, writable only by CI/CD pipelines and authorized deployments
- **Version control**: All changes are tracked in git for audit purposes
- **CI/CD integration**: Only `scripts/deploy.sh` should modify these files
- **Mainnet immutability**: Enforce that mainnet entries cannot be deleted or modified

## Links

- [Deployment Script](./scripts/deploy.sh) — Automated deployment and entry creation
- [Validation Script](./scripts/validate-deploy-registry.sh) — Validation tool
- [Stellar Soroban Docs](https://developers.stellar.org/docs/contracts) — Contract deployment reference
