# Auth Requirements Matrix — tycoon-collectibles

Referenced from `README.md`. Table of who must sign for each entrypoint.

| Entrypoint | Auth required | Notes |
|------------|---------------|-------|
| `initialize` | `admin.require_auth()` | One-time setup |
| `mint_batch` / `stock` | `admin.require_auth()` | Admin-only supply control |
| `set_*` config setters | `admin.require_auth()` | Admin-only |
| `buy` | `buyer.require_auth()` | Buyer authorizes payment |
| `transfer` | `from.require_auth()` | Owner authorizes the transfer |
| `burn` | `owner.require_auth()` / `caller.require_auth()` | See `src/lib.rs` for the caller-vs-owner distinction |

All other read-only views require no auth. See `entrypoint_auth_tests.rs`
for the enforced test coverage behind this table.
