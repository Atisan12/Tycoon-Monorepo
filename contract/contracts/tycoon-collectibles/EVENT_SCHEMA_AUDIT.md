# Event Schema Audit — tycoon-collectibles

Scope: topics stable for indexers.

## Guarantee

Event topic symbols are append-only: existing topic strings and their
positional data-tuple shape are never renamed or reordered in a released
version. New fields are added at the end of the data tuple only.

## Current topics (from `src/events.rs`)

| Topic | Data tuple |
|-------|------------|
| `("transfer", from, to)` | `(token_id, amount)` |
| `("burn", "coll", burner)` | `(token_id, perk, strength)` |
| `("perk", "cash", activator)` | `(token_id, cash_value)` |
| `("coll_buy", buyer)` | `(token_id, price, use_usdc)` |
| `("coll_stock", ...)` | see `emit_collectible_stocked_event` |

Any new event must preserve this ordering guarantee and be added to this
table before release.
