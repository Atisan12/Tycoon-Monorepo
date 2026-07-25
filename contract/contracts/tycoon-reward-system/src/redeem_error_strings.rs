//! Stable panic strings for the redeem paths (`redeem_voucher`,
//! `redeem_voucher_from`) and the surrounding balance/state checks they rely
//! on. Clients (SDKs, indexers, front-ends) match on these strings to
//! present user-facing errors, so they must not be reworded once shipped —
//! only appended to. Mirrors the literals currently panicked with in
//! `lib.rs`.
//!
//! Not yet wired into `lib.rs`; kept here as the single source of truth to
//! swap in when `lib.rs` panic call sites are next touched.

pub const ERR_ALREADY_INITIALIZED: &str = "Already initialized";
pub const ERR_UNAUTHORIZED_MINT: &str = "Unauthorized: only admin or backend minter can mint";
pub const ERR_CONTRACT_PAUSED: &str = "Contract is paused";
pub const ERR_REDEEM_VOUCHER_DEPRECATED: &str = "Use redeem_voucher_from instead";
pub const ERR_INVALID_TOKEN_NOT_ALLOWLISTED: &str = "Invalid token: not in allowlist";
pub const ERR_INSUFFICIENT_CONTRACT_BALANCE: &str = "Insufficient contract balance";
pub const ERR_INSUFFICIENT_BALANCE: &str = "Insufficient balance";
