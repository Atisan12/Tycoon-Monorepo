# Upgrade / Migration Governance — tycoon-boost-system

Scope: who holds upgrade auth.

## Who holds upgrade auth

The stored `admin` address is the sole holder of upgrade authority for
this contract. Any WASM upgrade path (`update_current_contract_wasm`)
must be gated behind `admin.require_auth()`, matching the auth pattern
already used for every other admin-only entrypoint in `src/lib.rs`.

## Governance model

This contract follows the shared upgrade-governance primitives defined
in `tycoon-lib::migration` (`MigrationKey`, `MigrationGuard`,
`MigrationState`) and the time-lock policy documented there
(minimum 1 day, default 3 day delay between proposing and executing an
upgrade). See `contract/contracts/tycoon-lib/src/migration.rs` for the
authoritative policy.
