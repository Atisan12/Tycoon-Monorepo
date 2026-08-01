//! Tests for issue #1330: calculate_total_boost ledger expiry inclusive boundary
//!
//! # Expiry Boundary Contract
//!
//! `expires_at_ledger` semantics in `calculate_total_boost`:
//!
//! | expires_at_ledger | current_ledger | Status  |
//! |-------------------|----------------|---------|
//! | 0                 | any            | Active  (sentinel: never expires)  |
//! | N                 | < N            | Active  (ledger has not reached expiry) |
//! | N                 | == N           | EXPIRED (inclusive: expired at this ledger) |
//! | N                 | > N            | EXPIRED |
//!
//! This means a boost with `expires_at_ledger = N` contributes to
//! `calculate_total_boost` only when `current_ledger < N`. At ledger N it is
//! already expired.
//!
//! # Documentation (required by issue #1330)
//!
//! `calculate_total_boost` uses a strict `>` check:
//!
//! ```text
//! if b.expires_at_ledger == 0 || b.expires_at_ledger > current_ledger {
//!     // active
//! }
//! ```
//!
//! A boost expires **at** `expires_at_ledger` (inclusive boundary).
//! It is active for ledgers strictly less than `expires_at_ledger`.

#[cfg(test)]
extern crate std;

use crate::{Boost, BoostType, TycoonBoostSystem, TycoonBoostSystemClient};
use soroban_sdk::{
    testutils::{Address as _, Ledger, LedgerInfo},
    Address, Env,
};

// ── helpers ───────────────────────────────────────────────────────────────────

fn make_env() -> Env {
    let env = Env::default();
    env.mock_all_auths();
    env
}

fn setup(env: &Env) -> (TycoonBoostSystemClient<'_>, Address) {
    let id = env.register(TycoonBoostSystem, ());
    let client = TycoonBoostSystemClient::new(env, &id);
    let admin = Address::generate(env);
    let player = Address::generate(env);
    client.initialize(&admin);
    (client, player)
}

fn set_seq(env: &Env, seq: u32) {
    env.ledger().set(LedgerInfo {
        sequence_number: seq,
        timestamp: seq as u64 * 5,
        protocol_version: 23,
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 1,
        min_persistent_entry_ttl: 1,
        max_entry_ttl: 6_312_000,
    });
}

fn additive(id: u128, value: u32, expires_at_ledger: u32) -> Boost {
    Boost {
        id,
        boost_type: BoostType::Additive,
        value,
        priority: 0,
        expires_at_ledger,
    }
}

fn multiplicative(id: u128, value: u32, expires_at_ledger: u32) -> Boost {
    Boost {
        id,
        boost_type: BoostType::Multiplicative,
        value,
        priority: 0,
        expires_at_ledger,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// INCLUSIVE BOUNDARY: expires_at_ledger == current_ledger → EXPIRED
// ─────────────────────────────────────────────────────────────────────────────

/// EXP-INCL-1: A boost with `expires_at_ledger == current_ledger` must NOT
/// contribute to `calculate_total_boost` (inclusive expiry).
#[test]
fn test_boost_expired_at_exact_expiry_ledger() {
    let env = make_env();
    let (client, player) = setup(&env);

    set_seq(&env, 50);
    client.add_boost(&player, &additive(1, 1000, 100)); // +10%, expires at 100

    // At exactly ledger 100 the boost is expired
    set_seq(&env, 100);
    assert_eq!(
        client.calculate_total_boost(&player),
        10000,
        "Boost must be expired (inclusive) at ledger == expires_at_ledger"
    );
}

/// EXP-INCL-2: One ledger before expiry the boost is still active.
#[test]
fn test_boost_active_one_before_expiry() {
    let env = make_env();
    let (client, player) = setup(&env);

    set_seq(&env, 50);
    client.add_boost(&player, &additive(1, 1000, 100)); // expires at 100

    set_seq(&env, 99); // one ledger before
    assert_eq!(
        client.calculate_total_boost(&player),
        11000,
        "Boost must be active one ledger before expires_at_ledger"
    );
}

/// EXP-INCL-3: One ledger after expiry the boost is expired (redundant sanity check).
#[test]
fn test_boost_expired_one_after_expiry() {
    let env = make_env();
    let (client, player) = setup(&env);

    set_seq(&env, 50);
    client.add_boost(&player, &additive(1, 1000, 100));

    set_seq(&env, 101);
    assert_eq!(client.calculate_total_boost(&player), 10000);
}

// ─────────────────────────────────────────────────────────────────────────────
// SENTINEL: expires_at_ledger == 0 → NEVER EXPIRES
// ─────────────────────────────────────────────────────────────────────────────

/// Sentinel-0: A boost with `expires_at_ledger == 0` never expires.
#[test]
fn test_boost_never_expires_sentinel_zero() {
    let env = make_env();
    let (client, player) = setup(&env);

    set_seq(&env, 1);
    client.add_boost(&player, &additive(1, 500, 0)); // never expires

    // Advance ledger far into the future
    set_seq(&env, 999_999);
    assert_eq!(
        client.calculate_total_boost(&player),
        10500, // 10000 * (1 + 0.05) = 10500
        "Sentinel boost (expires_at_ledger=0) must never expire"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// MIXED: expired + active boosts at boundary
// ─────────────────────────────────────────────────────────────────────────────

/// MIXED-1: One expired, one active — only active boost contributes.
#[test]
fn test_mixed_one_expired_one_active_at_boundary() {
    let env = make_env();
    let (client, player) = setup(&env);

    set_seq(&env, 50);
    client.add_boost(&player, &additive(1, 1000, 100)); // expires at 100: +10%
    client.add_boost(&player, &additive(2, 500, 200)); // expires at 200: +5%

    // At ledger 100: boost 1 expired, boost 2 still active
    set_seq(&env, 100);
    assert_eq!(
        client.calculate_total_boost(&player),
        10500, // only boost 2: 10000 * (1 + 0.05) = 10500
        "Only non-expired boost must contribute at boundary"
    );
}

/// MIXED-2: Both boosts expire at different inclusive boundaries.
#[test]
fn test_mixed_both_boosts_expire_at_respective_boundaries() {
    let env = make_env();
    let (client, player) = setup(&env);

    set_seq(&env, 10);
    client.add_boost(&player, &additive(1, 2000, 50)); // +20%, expires at 50
    client.add_boost(&player, &additive(2, 1000, 75)); // +10%, expires at 75

    // At 49: both active → 10000 * (1 + 0.30) = 13000
    set_seq(&env, 49);
    assert_eq!(client.calculate_total_boost(&player), 13000);

    // At 50: boost 1 expired → 10000 * (1 + 0.10) = 11000
    set_seq(&env, 50);
    assert_eq!(client.calculate_total_boost(&player), 11000);

    // At 74: boost 2 still active → 11000
    set_seq(&env, 74);
    assert_eq!(client.calculate_total_boost(&player), 11000);

    // At 75: boost 2 expired → base 10000
    set_seq(&env, 75);
    assert_eq!(client.calculate_total_boost(&player), 10000);
}

/// MIXED-3: Never-expiring sentinel + expiring boost at boundary.
#[test]
fn test_sentinel_plus_expiring_boost_at_boundary() {
    let env = make_env();
    let (client, player) = setup(&env);

    set_seq(&env, 1);
    client.add_boost(&player, &additive(1, 500, 0)); // never expires: +5%
    client.add_boost(&player, &additive(2, 1000, 100)); // expires at 100: +10%

    // At 99: both active → 10000 * (1 + 0.15) = 11500
    set_seq(&env, 99);
    assert_eq!(client.calculate_total_boost(&player), 11500);

    // At 100: boost 2 expired → only sentinel: 10000 * (1 + 0.05) = 10500
    set_seq(&env, 100);
    assert_eq!(
        client.calculate_total_boost(&player),
        10500,
        "At expiry ledger only sentinel survives"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// MULTIPLICATIVE BOOSTS: expiry boundary
// ─────────────────────────────────────────────────────────────────────────────

/// Multiplicative boost expired at exact boundary returns base value.
#[test]
fn test_multiplicative_expired_at_exact_boundary() {
    let env = make_env();
    let (client, player) = setup(&env);

    set_seq(&env, 10);
    client.add_boost(&player, &multiplicative(1, 15000, 100)); // 1.5×, expires at 100

    // At 99: active → 10000 * 1.5 = 15000
    set_seq(&env, 99);
    assert_eq!(client.calculate_total_boost(&player), 15000);

    // At 100: expired → base
    set_seq(&env, 100);
    assert_eq!(client.calculate_total_boost(&player), 10000);
}

/// Multiplicative boost one ledger before expiry is still active.
#[test]
fn test_multiplicative_active_one_before_boundary() {
    let env = make_env();
    let (client, player) = setup(&env);

    set_seq(&env, 1);
    client.add_boost(&player, &multiplicative(1, 12000, 50)); // 1.2×

    set_seq(&env, 49);
    assert_eq!(client.calculate_total_boost(&player), 12000);
}

// ─────────────────────────────────────────────────────────────────────────────
// get_active_boosts: same inclusive boundary
// ─────────────────────────────────────────────────────────────────────────────

/// `get_active_boosts` also treats expires_at_ledger == current as expired.
#[test]
fn test_get_active_boosts_treats_exact_expiry_as_expired() {
    let env = make_env();
    let (client, player) = setup(&env);

    set_seq(&env, 10);
    client.add_boost(&player, &additive(1, 1000, 100)); // expires at 100
    client.add_boost(&player, &additive(2, 500, 0)); // never expires

    set_seq(&env, 100);
    let active = client.get_active_boosts(&player);
    assert_eq!(
        active.len(),
        1,
        "Only the never-expiring boost must be returned at exact expiry ledger"
    );
    assert_eq!(
        active.get(0).unwrap().id,
        2,
        "The returned boost must be the never-expiring sentinel"
    );
}

/// `get_active_boosts` returns all boosts when none have reached expiry yet.
#[test]
fn test_get_active_boosts_returns_all_before_expiry() {
    let env = make_env();
    let (client, player) = setup(&env);

    set_seq(&env, 1);
    client.add_boost(&player, &additive(1, 1000, 100));
    client.add_boost(&player, &additive(2, 500, 200));
    client.add_boost(&player, &additive(3, 200, 0));

    set_seq(&env, 50); // all active
    assert_eq!(client.get_active_boosts(&player).len(), 3);
}

// ─────────────────────────────────────────────────────────────────────────────
// PRUNE: same boundary applies to prune_expired_boosts
// ─────────────────────────────────────────────────────────────────────────────

/// `prune_expired_boosts` removes a boost whose `expires_at_ledger == current`.
#[test]
fn test_prune_removes_boost_at_exact_expiry_ledger() {
    let env = make_env();
    let (client, player) = setup(&env);

    set_seq(&env, 10);
    client.add_boost(&player, &additive(1, 1000, 100)); // expires at 100
    client.add_boost(&player, &additive(2, 500, 0)); // never expires

    set_seq(&env, 100); // boost 1 expired
    let removed = client.prune_expired_boosts(&player);
    assert_eq!(removed, 1, "Exactly one boost must be pruned at boundary");
    assert_eq!(
        client.get_active_boosts(&player).len(),
        1,
        "Only sentinel boost must remain after prune"
    );
}

/// After pruning at boundary, `calculate_total_boost` reflects only surviving boost.
#[test]
fn test_calculate_total_boost_after_prune_at_boundary() {
    let env = make_env();
    let (client, player) = setup(&env);

    set_seq(&env, 1);
    client.add_boost(&player, &additive(1, 2000, 50)); // expires at 50: +20%
    client.add_boost(&player, &additive(2, 500, 0)); // never expires: +5%

    set_seq(&env, 50);
    client.prune_expired_boosts(&player);

    // Only boost 2 remains: 10000 * (1 + 0.05) = 10500
    assert_eq!(client.calculate_total_boost(&player), 10500);
}

// ─────────────────────────────────────────────────────────────────────────────
// EMPTY PLAYER: no boosts → base 10000
// ─────────────────────────────────────────────────────────────────────────────

/// Player with no boosts always returns base (10000) regardless of ledger.
#[test]
fn test_no_boosts_returns_base_at_any_ledger() {
    let env = make_env();
    let id = env.register(TycoonBoostSystem, ());
    let client = TycoonBoostSystemClient::new(&env, &id);
    let player = Address::generate(&env);

    for seq in [0u32, 1, 100, 99_999] {
        set_seq(&env, seq);
        assert_eq!(
            client.calculate_total_boost(&player),
            10000,
            "Empty player must always return base at ledger={}",
            seq
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// EDGE: expiry at ledger 1 (genesis + 1)
// ─────────────────────────────────────────────────────────────────────────────

/// A boost that expires at ledger 1 is already expired at ledger 1 and active
/// only at ledger 0.
#[test]
fn test_expiry_at_ledger_one_inclusive() {
    let env = make_env();
    set_seq(&env, 0);
    let (client, player) = setup(&env);

    // expires_at_ledger=1 is the minimum valid expiry when current_ledger=0
    client.add_boost(&player, &additive(1, 1000, 1));

    // Active at ledger 0 (current < 1)
    assert_eq!(client.calculate_total_boost(&player), 11000);

    // Expired at ledger 1 (current == 1, inclusive boundary)
    set_seq(&env, 1);
    assert_eq!(client.calculate_total_boost(&player), 10000);
}

// ─────────────────────────────────────────────────────────────────────────────
// EDGE: expiry at u32::MAX (far-future boost is effectively permanent for tests)
// ─────────────────────────────────────────────────────────────────────────────

/// A boost with `expires_at_ledger == u32::MAX` is active at any reasonable ledger.
#[test]
fn test_expiry_at_u32_max_is_active_at_normal_ledgers() {
    let env = make_env();
    let (client, player) = setup(&env);

    set_seq(&env, 1);
    client.add_boost(&player, &additive(1, 1000, u32::MAX));

    set_seq(&env, 1_000_000); // far into the future but << u32::MAX
    assert_eq!(
        client.calculate_total_boost(&player),
        11000,
        "u32::MAX expiry boost must be active at ledger 1_000_000"
    );
}
