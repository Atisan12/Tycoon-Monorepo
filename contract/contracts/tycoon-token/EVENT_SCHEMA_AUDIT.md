# Event Schema Audit — tycoon-token

Scope: topics stable for indexers.

## Guarantee

Event topic symbols emitted by this contract (standard SEP-41 token
events: `transfer`, `mint`, `burn`, `approve`, `set_admin`) are append-only.
Existing topic strings and their positional data-tuple shape are never
renamed or reordered in a released version; new fields are added at the
end of the data tuple only, so indexers decoding by position keep working
across upgrades.

## Current topics

See `src/lib.rs` for the authoritative event emission call sites. Any new
event added to this contract must be listed here before release.
