# Auth Requirements Matrix — tycoon-boost-system

Referenced from `README.md`. Table of who must sign for each entrypoint.

| Entrypoint | Auth required | Notes |
|------------|---------------|-------|
| `initialize` | `admin.require_auth()` | One-time setup |
| `set_boost_config` | `admin.require_auth()` | Admin-only config write |
| `grant_boost` | `admin.require_auth()` | Admin-only privileged grant |
| `admin`-gated setters | `admin.require_auth()` | See `src/lib.rs` for full list |

All other read-only views require no auth. Any entrypoint not listed here
should be treated as admin-gated until confirmed otherwise in `src/lib.rs`.
