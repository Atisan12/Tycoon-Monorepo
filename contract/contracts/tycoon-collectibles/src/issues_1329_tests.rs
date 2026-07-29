//! Tests for issue #1329: burn_collectible_for_perk strength validation matrix
//!
//! Tiered perks (CashTiered=1, TaxRefund=2) require strength in [1, 5].
//! Non-tiered perks (RentBoost=3, PropertyDiscount=4, ExtraTurn=5, JailFree=6,
//! DoubleRent=7, RollBoost=8, Teleport=9, Shield=10, RollExact=11) accept any
//! strength value including 0.
//!
//! Acceptance criteria:
//! - Tiered perks with strength 0 → InvalidStrength
//! - Tiered perks with strength > 5 → InvalidStrength
//! - Tiered perks with strength in [1,5] → Ok (perk activated, token burned)
//! - Non-tiered perks with any strength → Ok (perk activated, token burned)
//! - Perk::None → InvalidPerk regardless of strength
//! - Paused contract → ContractPaused regardless of perk

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env};
extern crate std;

// ── helpers ───────────────────────────────────────────────────────────────────

fn setup(env: &Env) -> (TycoonCollectiblesClient<'_>, Address) {
    let contract_id = env.register(TycoonCollectibles, ());
    let client = TycoonCollectiblesClient::new(env, &contract_id);
    let admin = Address::generate(env);
    client.initialize(&admin);
    (client, admin)
}

/// Stock a collectible with the given perk/strength and give the caller 1 unit.
/// Returns the token_id.
fn stock_and_give(
    env: &Env,
    client: &TycoonCollectiblesClient<'_>,
    caller: &Address,
    perk: u32,
    strength: u32,
) -> u128 {
    let token_id = client.stock_shop(&10, &perk, &strength, &0, &0);
    // Transfer from contract (shop inventory) to caller via buy_collectible
    client.buy_collectible(caller, &token_id, &1);
    token_id
}

// ─────────────────────────────────────────────────────────────────────────────
// TIERED PERK STRENGTH VALIDATION
// ─────────────────────────────────────────────────────────────────────────────

/// CashTiered (perk=1) with strength=0 must return InvalidStrength.
#[test]
fn test_burn_tiered_cash_strength_zero_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _) = setup(&env);

    let caller = Address::generate(&env);
    // Stock at strength=0 (invalid for tiered) — stock_shop bypasses burn validation.
    // We must manually set perk+strength via set_token_perk after minting.
    let token_id = client.stock_shop(&1, &1, &1, &0, &0); // stock with strength=1 first
    client.buy_collectible(&caller, &token_id, &1);
    // Override strength to 0 via set_token_perk
    client.set_token_perk(&token_id, &Perk::CashTiered, &0);

    let result = client.try_burn_collectible_for_perk(&caller, &token_id);
    assert!(
        result.is_err(),
        "CashTiered with strength=0 must fail with InvalidStrength"
    );
}

/// CashTiered (perk=1) with strength=6 must return InvalidStrength.
#[test]
fn test_burn_tiered_cash_strength_six_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _) = setup(&env);

    let caller = Address::generate(&env);
    let token_id = client.stock_shop(&1, &1, &1, &0, &0);
    client.buy_collectible(&caller, &token_id, &1);
    client.set_token_perk(&token_id, &Perk::CashTiered, &6);

    let result = client.try_burn_collectible_for_perk(&caller, &token_id);
    assert!(
        result.is_err(),
        "CashTiered with strength=6 must fail with InvalidStrength"
    );
}

/// CashTiered at each valid strength tier [1..=5] must succeed and burn the token.
#[test]
fn test_burn_tiered_cash_all_valid_strengths() {
    for strength in 1u32..=5 {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _) = setup(&env);

        let caller = Address::generate(&env);
        let token_id = client.stock_shop(&10, &1, &strength, &0, &0);
        client.buy_collectible(&caller, &token_id, &1);

        let result = client.try_burn_collectible_for_perk(&caller, &token_id);
        assert!(
            result.is_ok(),
            "CashTiered with strength={} must succeed",
            strength
        );
        // Token burned: balance must be 0
        assert_eq!(
            client.balance_of(&caller, &token_id),
            0,
            "Balance must be 0 after burn for strength={}",
            strength
        );
    }
}

/// TaxRefund (perk=2) with strength=0 must return InvalidStrength.
#[test]
fn test_burn_tiered_tax_refund_strength_zero_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _) = setup(&env);

    let caller = Address::generate(&env);
    let token_id = client.stock_shop(&1, &2, &1, &0, &0);
    client.buy_collectible(&caller, &token_id, &1);
    client.set_token_perk(&token_id, &Perk::TaxRefund, &0);

    let result = client.try_burn_collectible_for_perk(&caller, &token_id);
    assert!(
        result.is_err(),
        "TaxRefund with strength=0 must fail with InvalidStrength"
    );
}

/// TaxRefund (perk=2) with strength=6 must return InvalidStrength.
#[test]
fn test_burn_tiered_tax_refund_strength_six_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _) = setup(&env);

    let caller = Address::generate(&env);
    let token_id = client.stock_shop(&1, &2, &1, &0, &0);
    client.buy_collectible(&caller, &token_id, &1);
    client.set_token_perk(&token_id, &Perk::TaxRefund, &6);

    let result = client.try_burn_collectible_for_perk(&caller, &token_id);
    assert!(
        result.is_err(),
        "TaxRefund with strength=6 must fail with InvalidStrength"
    );
}

/// TaxRefund at each valid strength tier [1..=5] must succeed and burn the token.
#[test]
fn test_burn_tiered_tax_refund_all_valid_strengths() {
    for strength in 1u32..=5 {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _) = setup(&env);

        let caller = Address::generate(&env);
        let token_id = client.stock_shop(&10, &2, &strength, &0, &0);
        client.buy_collectible(&caller, &token_id, &1);

        let result = client.try_burn_collectible_for_perk(&caller, &token_id);
        assert!(
            result.is_ok(),
            "TaxRefund with strength={} must succeed",
            strength
        );
        assert_eq!(
            client.balance_of(&caller, &token_id),
            0,
            "Balance must be 0 after burn for TaxRefund strength={}",
            strength
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// NON-TIERED PERK STRENGTH VALIDATION (any strength accepted)
// ─────────────────────────────────────────────────────────────────────────────

/// All non-tiered perks must succeed at strength=0 (no strength requirement).
#[test]
fn test_burn_non_tiered_perks_strength_zero_succeeds() {
    // (perk_value, perk_enum) for non-tiered perks
    let non_tiered: &[(u32, Perk)] = &[
        (3, Perk::RentBoost),
        (4, Perk::PropertyDiscount),
        (5, Perk::ExtraTurn),
        (6, Perk::JailFree),
        (7, Perk::DoubleRent),
        (8, Perk::RollBoost),
        (9, Perk::Teleport),
        (10, Perk::Shield),
        (11, Perk::RollExact),
    ];

    for (perk_val, perk_enum) in non_tiered {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _) = setup(&env);

        let caller = Address::generate(&env);
        let token_id = client.stock_shop(&5, perk_val, &0, &0, &0);
        client.buy_collectible(&caller, &token_id, &1);

        // Ensure perk enum is correctly stored
        assert_eq!(
            client.get_token_perk(&token_id),
            *perk_enum,
            "Perk enum mismatch for perk_val={}",
            perk_val
        );

        let result = client.try_burn_collectible_for_perk(&caller, &token_id);
        assert!(
            result.is_ok(),
            "Non-tiered perk={} with strength=0 must succeed",
            perk_val
        );
        assert_eq!(
            client.balance_of(&caller, &token_id),
            0,
            "Token must be burned for perk={}",
            perk_val
        );
    }
}

/// All non-tiered perks must also succeed at strength=5 (arbitrary non-zero).
#[test]
fn test_burn_non_tiered_perks_strength_five_succeeds() {
    let non_tiered: &[u32] = &[3, 4, 5, 6, 7, 8, 9, 10, 11];

    for &perk_val in non_tiered {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _) = setup(&env);

        let caller = Address::generate(&env);
        let token_id = client.stock_shop(&5, &perk_val, &5, &0, &0);
        client.buy_collectible(&caller, &token_id, &1);

        let result = client.try_burn_collectible_for_perk(&caller, &token_id);
        assert!(
            result.is_ok(),
            "Non-tiered perk={} with strength=5 must succeed",
            perk_val
        );
        assert_eq!(client.balance_of(&caller, &token_id), 0);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// PERK::NONE REJECTION
// ─────────────────────────────────────────────────────────────────────────────

/// A token with Perk::None must return InvalidPerk.
#[test]
fn test_burn_perk_none_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _) = setup(&env);

    let caller = Address::generate(&env);
    // Mint token 42 directly with no perk set (perk defaults to None)
    client.buy_collectible(&caller, &42, &1);
    // Explicitly set to None
    client.set_token_perk(&42, &Perk::None, &0);

    let result = client.try_burn_collectible_for_perk(&caller, &42);
    assert!(
        result.is_err(),
        "Perk::None must be rejected with InvalidPerk"
    );
    // Token must not have been burned
    assert_eq!(client.balance_of(&caller, &42), 1);
}

// ─────────────────────────────────────────────────────────────────────────────
// CONTRACT PAUSED
// ─────────────────────────────────────────────────────────────────────────────

/// burn_collectible_for_perk on a paused contract must return ContractPaused.
#[test]
fn test_burn_paused_contract_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _) = setup(&env);

    let caller = Address::generate(&env);
    let token_id = client.stock_shop(&5, &1, &3, &0, &0);
    client.buy_collectible(&caller, &token_id, &1);

    client.set_pause(&true);

    let result = client.try_burn_collectible_for_perk(&caller, &token_id);
    assert!(result.is_err(), "Paused contract must reject burn");
    // Token must not have been burned
    assert_eq!(client.balance_of(&caller, &token_id), 1);
}

// ─────────────────────────────────────────────────────────────────────────────
// INSUFFICIENT BALANCE
// ─────────────────────────────────────────────────────────────────────────────

/// Caller with zero balance must get InsufficientBalance, not proceed to perk check.
#[test]
fn test_burn_insufficient_balance_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _) = setup(&env);

    let caller = Address::generate(&env);
    let token_id = client.stock_shop(&5, &1, &2, &0, &0);
    // Do NOT give the caller any tokens

    let result = client.try_burn_collectible_for_perk(&caller, &token_id);
    assert!(result.is_err(), "Zero balance must return InsufficientBalance");
}

// ─────────────────────────────────────────────────────────────────────────────
// MULTIPLE BURNS OF SAME PERK
// ─────────────────────────────────────────────────────────────────────────────

/// Burning multiple copies of a tiered collectible works correctly each time.
#[test]
fn test_burn_multiple_copies_tiered_perk() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _) = setup(&env);

    let caller = Address::generate(&env);
    let token_id = client.stock_shop(&5, &1, &3, &0, &0);
    client.buy_collectible(&caller, &token_id, &3);
    assert_eq!(client.balance_of(&caller, &token_id), 3);

    // Burn first copy
    client.burn_collectible_for_perk(&caller, &token_id);
    assert_eq!(client.balance_of(&caller, &token_id), 2);

    // Burn second copy
    client.burn_collectible_for_perk(&caller, &token_id);
    assert_eq!(client.balance_of(&caller, &token_id), 1);

    // Burn third copy
    client.burn_collectible_for_perk(&caller, &token_id);
    assert_eq!(client.balance_of(&caller, &token_id), 0);

    // Fourth burn must fail
    let result = client.try_burn_collectible_for_perk(&caller, &token_id);
    assert!(result.is_err(), "Fourth burn with empty balance must fail");
}

// ─────────────────────────────────────────────────────────────────────────────
// BOUNDARY: strength at exact edge values for tiered perks
// ─────────────────────────────────────────────────────────────────────────────

/// CashTiered strength=1 (lower boundary) succeeds.
#[test]
fn test_burn_tiered_cash_strength_boundary_lower() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _) = setup(&env);

    let caller = Address::generate(&env);
    let token_id = client.stock_shop(&5, &1, &1, &0, &0);
    client.buy_collectible(&caller, &token_id, &1);

    assert!(client.try_burn_collectible_for_perk(&caller, &token_id).is_ok());
    assert_eq!(client.balance_of(&caller, &token_id), 0);
}

/// CashTiered strength=5 (upper boundary) succeeds.
#[test]
fn test_burn_tiered_cash_strength_boundary_upper() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _) = setup(&env);

    let caller = Address::generate(&env);
    let token_id = client.stock_shop(&5, &1, &5, &0, &0);
    client.buy_collectible(&caller, &token_id, &1);

    assert!(client.try_burn_collectible_for_perk(&caller, &token_id).is_ok());
    assert_eq!(client.balance_of(&caller, &token_id), 0);
}

/// CashTiered strength=6 (one above upper boundary) is rejected.
#[test]
fn test_burn_tiered_cash_strength_one_above_upper_boundary() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _) = setup(&env);

    let caller = Address::generate(&env);
    let token_id = client.stock_shop(&5, &1, &1, &0, &0);
    client.buy_collectible(&caller, &token_id, &1);
    client.set_token_perk(&token_id, &Perk::CashTiered, &6);

    let result = client.try_burn_collectible_for_perk(&caller, &token_id);
    assert!(result.is_err());
    // Token is NOT burned
    assert_eq!(client.balance_of(&caller, &token_id), 1);
}
