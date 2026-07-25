# Event Schema Audit — tycoon-boost-system

Scope: topics stable for indexers.

## Guarantee

Event topic symbols emitted by this contract are append-only: existing
topic strings and their positional data-tuple shape are never renamed or
reordered in a released version. New fields are added at the end of the
data tuple, never inserted in the middle, so indexers decoding by position
keep working across upgrades.

## Current topics

See `src/lib.rs` for the authoritative event emission call sites. Any new
event added to this contract must be listed here alongside its topic
symbol and data tuple shape before release.
