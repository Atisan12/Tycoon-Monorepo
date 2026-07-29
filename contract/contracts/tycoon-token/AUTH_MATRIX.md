# Auth Requirements Matrix — tycoon-token

Referenced from `README.md`. Summarized from the inline matrix already in
`src/lib.rs`, extracted here so it's discoverable from the README without
opening source.

| Entrypoint | Auth required | Notes |
|------------|---------------|-------|
| `mint` | `admin.require_auth()` | Admin must pre-sign TX |
| `set_admin` | `admin.require_auth()` | Admin key holder only |
| `transfer` | `from.require_auth()` | Caller = `from` or has signature |
| `transfer_from` | `spender.require_auth()` | Spender contract with its own auth |
| `approve` | `from.require_auth()` | Owner must sign |
| `burn` | `from.require_auth()` | `from` must sign |
| `burn_from` | `spender.require_auth()` | Spender contract with its own auth |

All balance/allowance/supply views are read-only and require no auth.
