# Auth Requirements Matrix — tycoon-lib

Referenced from `README.md`. tycoon-lib is a shared crate, not a deployed
contract; this documents the auth *patterns* it provides for consuming
contracts (see `src/admin.rs`, `src/auth.rs`).

| Pattern | Auth required | Used for |
|---------|---------------|----------|
| `AdminOnly` | `admin.require_auth()` | Mutating privileged/global state |
| `SelfAuth` | `caller.require_auth()` | Caller acting on their own resource |
| `SpenderAuth` | `spender.require_auth()` | Third party acting via prior approval |

Consuming contracts (tycoon-token, tycoon-game, tycoon-collectibles,
tycoon-boost-system, tycoon-reward-system) map their entrypoints onto one
of these three patterns; see each contract's own auth matrix doc.
