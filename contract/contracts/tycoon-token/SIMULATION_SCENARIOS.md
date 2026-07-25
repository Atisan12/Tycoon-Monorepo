# Simulation Scenarios — Supply Conservation

Scope: `src/simulation_scenarios.rs`

## Invariant

Total supply is conserved across every transfer path. Minting and burning
are the only operations allowed to change `total_supply`; `transfer`,
`transfer_from`, and `approve` must never create or destroy tokens.

## Coverage

| Scenario | Path exercised | Conservation check |
|----------|-----------------|---------------------|
| Reward payout | `transfer` (pool -> winner) | `total_supply` unchanged before/after |
| Approval spend | `approve` + `transfer_from` | sender debit == recipient credit |
| Multi-hop chain | `transfer` -> `transfer_from` -> `burn_from` | `total_supply` only drops by the burned amount |
| Batch distribution | repeated `transfer` to N players | sum of balances == `total_supply` |
| Admin rotation mid-flow | `set_admin` interleaved with transfers | `total_supply` unaffected by admin changes |

## Result

No scenario in `simulation_scenarios.rs` mutates `total_supply` via a
transfer-family call; supply only moves via explicit `mint`/`burn`. This
file documents the invariant so future scenarios are held to the same bar.
