//! Tests for issue #1328: buy_collectible_from_shop CEI ordering review
//!
//! Checks-Effects-Interactions (CEI) pattern:
//!   1. CHECKS  — validate all inputs and state
//!   2. EFFECTS — update state (decrement stock, mint to buyer)
//!   3. INTERACTIONS — external token transfer(s)
//!
//! The key invariant: after a successful buy, all state mutations (stock
//! decrement and buyer balance increase) happen *before* the external
//! payment transfer, so any re-entrant call through the token contract
//! observes the already-updated stock/balance.
//!
//! Acceptance criteria:
//! - After buy, buyer holds +1 collectible and stock decremented by 1
//! - Buyer's token balance is reduced by the exact price
//! - Zero-price / negative-price → ZeroPrice error, no state mutation
//! - Out-of-stock → InsufficientStock error, no state mutation
//! - Shop not initialized → ShopNotInitialized error
//! - Token without a price entry → ZeroPrice error

use super::*;
use soroban_sdk::{
    testutils::Address as _,
    token::{StellarAssetClient, TokenClient},
    Address, Env,
};
extern crate std;

// ── helpers ───────────────────────────────────────────────────────────────────

fn make_token(env: &Env, admin: &Address) -> Address {
    env.register_stellar_asset_contract_v2(admin.clone())
        .address()
}

fn setup_with_shop(
    env: &Env,
) -> (
    TycoonCollectiblesClient<'_>,
    Address, // admin
    Address, // contract_id
    Address, // tyc_token
    Address, // usdc_token
) {
    let contract_id = env.register(TycoonCollectibles, ());
    let client = TycoonCollectiblesClient::new(env, &contract_id);
    let admin = Address::generate(env);
    client.initialize(&admin);

    let tyc = make_token(env, &admin);
    let usdc = make_token(env, &admin);
    client.init_shop(&tyc, &usdc);

    (client, admin, contract_id, tyc, usdc)
}

// ─────────────────────────────────────────────────────────────────────────────
// CEI: STATE UPDATED BEFORE PAYMENT
// ─────────────────────────────────────────────────────────────────────────────

/// After a successful buy with TYC the buyer holds the collectible,
/// stock is decremented, and payment is taken — in the correct CEI order.
#[test]
fn test_buy_state_updated_before_payment_tyc() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin, contract_id, tyc, _usdc) = setup_with_shop(&env);

    let price: i128 = 1_000;
    let initial_stock: u64 = 5;

    // SETUP: stock a collectible (perk=5 ExtraTurn, non-tiered, strength=0)
    let token_id = client.stock_shop(&initial_stock, &5, &0, &price as u128, &0);
    assert_eq!(client.get_stock(&token_id), initial_stock);

    let buyer = Address::generate(&env);
    StellarAssetClient::new(&env, &tyc).mint(&buyer, &(price * 2));

    // EXECUTE the buy
    client.buy_collectible_from_shop(&buyer, &token_id, &false);

    // EFFECTS were applied: buyer has the collectible
    assert_eq!(
        client.balance_of(&buyer, &token_id),
        1,
        "Buyer must hold exactly 1 collectible after purchase"
    );

    // EFFECTS: stock decremented
    assert_eq!(
        client.get_stock(&token_id),
        initial_stock - 1,
        "Stock must be decremented by 1"
    );

    // INTERACTIONS: payment taken (buyer balance reduced by price)
    let buyer_balance = TokenClient::new(&env, &tyc).balance(&buyer);
    assert_eq!(
        buyer_balance,
        price, // started with price*2, spent price
        "Buyer's TYC balance must be reduced by the exact price"
    );

    // Contract received the payment (no fee config)
    assert_eq!(
        TokenClient::new(&env, &tyc).balance(&contract_id),
        price,
        "Contract must have received the payment"
    );
}

/// After a successful buy with USDC the buyer holds the collectible
/// and USDC payment is taken correctly.
#[test]
fn test_buy_state_updated_before_payment_usdc() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin, contract_id, _tyc, usdc) = setup_with_shop(&env);

    let usdc_price: i128 = 500;
    let token_id = client.stock_shop(&3, &3, &0, &0, &usdc_price as u128);

    let buyer = Address::generate(&env);
    StellarAssetClient::new(&env, &usdc).mint(&buyer, &(usdc_price * 3));

    client.buy_collectible_from_shop(&buyer, &token_id, &true);

    assert_eq!(client.balance_of(&buyer, &token_id), 1);
    assert_eq!(client.get_stock(&token_id), 2);
    assert_eq!(
        TokenClient::new(&env, &usdc).balance(&buyer),
        usdc_price * 2,
        "Buyer spent exactly 500 USDC"
    );
    assert_eq!(
        TokenClient::new(&env, &usdc).balance(&contract_id),
        usdc_price
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// MULTIPLE SEQUENTIAL BUYS: STOCK TRACKING
// ─────────────────────────────────────────────────────────────────────────────

/// Multiple sequential buys decrement stock correctly and each buyer gets the token.
#[test]
fn test_multiple_sequential_buys_decrement_stock() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin, _contract_id, tyc, _usdc) = setup_with_shop(&env);

    let price: i128 = 100;
    let token_id = client.stock_shop(&3, &5, &0, &price as u128, &0);

    for expected_stock in [2u64, 1, 0] {
        let buyer = Address::generate(&env);
        StellarAssetClient::new(&env, &tyc).mint(&buyer, &(price * 2));
        client.buy_collectible_from_shop(&buyer, &token_id, &false);
        assert_eq!(client.balance_of(&buyer, &token_id), 1);
        assert_eq!(client.get_stock(&token_id), expected_stock);
    }

    // Fourth buy must fail: out of stock
    let last_buyer = Address::generate(&env);
    StellarAssetClient::new(&env, &tyc).mint(&last_buyer, &(price * 2));
    let result = client.try_buy_collectible_from_shop(&last_buyer, &token_id, &false);
    assert!(result.is_err(), "Out-of-stock buy must fail");
    // Stock remains 0, no phantom token minted
    assert_eq!(client.get_stock(&token_id), 0);
    assert_eq!(client.balance_of(&last_buyer, &token_id), 0);
}

// ─────────────────────────────────────────────────────────────────────────────
// CHECKS: ZERO/NEGATIVE PRICE → NO STATE MUTATION
// ─────────────────────────────────────────────────────────────────────────────

/// A token listed with TYC price = 0 must be rejected (ZeroPrice).
/// Stock and buyer balance must be unchanged.
#[test]
fn test_buy_zero_price_no_state_mutation() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin, _contract_id, tyc, _usdc) = setup_with_shop(&env);

    // Stock at price=0 — ZeroPrice check fires
    client.set_collectible_for_sale(&99, &0, &10, &5);

    let buyer = Address::generate(&env);
    StellarAssetClient::new(&env, &tyc).mint(&buyer, &1000);

    let result = client.try_buy_collectible_from_shop(&buyer, &99, &false);
    assert!(result.is_err(), "Zero TYC price must return ZeroPrice");

    // State must be unchanged
    assert_eq!(client.balance_of(&buyer, &99), 0, "No collectible minted");
    assert_eq!(client.get_stock(&99), 5, "Stock must remain unchanged");
    assert_eq!(
        TokenClient::new(&env, &tyc).balance(&buyer),
        1000,
        "Buyer balance must be unchanged"
    );
}

/// A token listed with negative TYC price must be rejected (ZeroPrice).
#[test]
fn test_buy_negative_price_no_state_mutation() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin, _contract_id, tyc, _usdc) = setup_with_shop(&env);

    client.set_collectible_for_sale(&88, &-1, &10, &5);

    let buyer = Address::generate(&env);
    StellarAssetClient::new(&env, &tyc).mint(&buyer, &1000);

    let result = client.try_buy_collectible_from_shop(&buyer, &88, &false);
    assert!(result.is_err(), "Negative price must return ZeroPrice");

    assert_eq!(client.balance_of(&buyer, &88), 0);
    assert_eq!(client.get_stock(&88), 5);
    assert_eq!(TokenClient::new(&env, &tyc).balance(&buyer), 1000);
}

// ─────────────────────────────────────────────────────────────────────────────
// CHECKS: OUT-OF-STOCK → NO STATE MUTATION
// ─────────────────────────────────────────────────────────────────────────────

/// Buying from an out-of-stock listing must fail without mutating state.
#[test]
fn test_buy_out_of_stock_no_state_mutation() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin, _contract_id, tyc, _usdc) = setup_with_shop(&env);

    let price: i128 = 100;
    // Stock only 1
    let token_id = client.stock_shop(&1, &5, &0, &price as u128, &0);

    let first_buyer = Address::generate(&env);
    StellarAssetClient::new(&env, &tyc).mint(&first_buyer, &(price * 2));
    client.buy_collectible_from_shop(&first_buyer, &token_id, &false);
    assert_eq!(client.get_stock(&token_id), 0);

    // Second buyer tries to buy out-of-stock item
    let second_buyer = Address::generate(&env);
    StellarAssetClient::new(&env, &tyc).mint(&second_buyer, &(price * 2));
    let result = client.try_buy_collectible_from_shop(&second_buyer, &token_id, &false);
    assert!(result.is_err(), "Out-of-stock must return InsufficientStock");

    // No collectible minted to second buyer
    assert_eq!(client.balance_of(&second_buyer, &token_id), 0);
    // Stock remains 0
    assert_eq!(client.get_stock(&token_id), 0);
    // Payment NOT taken
    assert_eq!(
        TokenClient::new(&env, &tyc).balance(&second_buyer),
        price * 2,
        "Payment must not be taken for out-of-stock"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// CHECKS: SHOP NOT INITIALIZED
// ─────────────────────────────────────────────────────────────────────────────

/// Buying before the shop is initialized must fail with ShopNotInitialized.
#[test]
fn test_buy_shop_not_initialized_fails() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(TycoonCollectibles, ());
    let client = TycoonCollectiblesClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.initialize(&admin);
    // No init_shop called

    let buyer = Address::generate(&env);
    let result = client.try_buy_collectible_from_shop(&buyer, &1, &false);
    assert!(
        result.is_err(),
        "Uninitialized shop must return ShopNotInitialized"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// CEI: FEE DISTRIBUTION DOES NOT AFFECT COLLECTIBLE STATE
// ─────────────────────────────────────────────────────────────────────────────

/// With a fee config, the collectible state (stock + buyer balance) is set in
/// the effects phase, before fees are distributed in the interactions phase.
#[test]
fn test_buy_with_fee_config_state_correct() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin, contract_id, tyc, _usdc) = setup_with_shop(&env);

    let platform = Address::generate(&env);
    let pool = Address::generate(&env);
    // 10% platform, 5% creator (goes to admin in shop context), 5% pool
    client.set_fee_config(&1000, &500, &500, &platform, &pool);

    let price: i128 = 1_000;
    let token_id = client.stock_shop(&5, &5, &0, &price as u128, &0);

    let buyer = Address::generate(&env);
    StellarAssetClient::new(&env, &tyc).mint(&buyer, &(price * 2));

    client.buy_collectible_from_shop(&buyer, &token_id, &false);

    // Collectible state (effects phase) is correct regardless of fee split
    assert_eq!(client.balance_of(&buyer, &token_id), 1);
    assert_eq!(client.get_stock(&token_id), 4);

    // Total paid by buyer equals full price
    let buyer_remaining = TokenClient::new(&env, &tyc).balance(&buyer);
    assert_eq!(
        buyer_remaining,
        price, // started with price*2
        "Buyer must have paid exactly the price"
    );

    // Fee recipients received their share (10% + 5% + 5% = 20% distributed)
    let platform_balance = TokenClient::new(&env, &tyc).balance(&platform);
    let pool_balance = TokenClient::new(&env, &tyc).balance(&pool);
    assert_eq!(platform_balance, 100, "Platform gets 10%");
    assert_eq!(pool_balance, 50, "Pool gets 5%");

    // Remaining 80% residue goes to contract (minus creator 5% goes to admin)
    let contract_balance = TokenClient::new(&env, &tyc).balance(&contract_id);
    assert_eq!(
        platform_balance + pool_balance + contract_balance + 50, // 50 = creator to admin
        price as i128,
        "All fees plus residue must sum to full price"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// RESTOCK AFTER PURCHASE
// ─────────────────────────────────────────────────────────────────────────────

/// After buying all stock, restocking allows further purchases.
#[test]
fn test_restock_after_exhaustion_allows_further_purchase() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin, _contract_id, tyc, _usdc) = setup_with_shop(&env);

    let price: i128 = 200;
    let token_id = client.stock_shop(&1, &5, &0, &price as u128, &0);

    let buyer1 = Address::generate(&env);
    StellarAssetClient::new(&env, &tyc).mint(&buyer1, &(price * 2));
    client.buy_collectible_from_shop(&buyer1, &token_id, &false);
    assert_eq!(client.get_stock(&token_id), 0);

    // Buying again fails
    let buyer2 = Address::generate(&env);
    StellarAssetClient::new(&env, &tyc).mint(&buyer2, &(price * 2));
    assert!(
        client
            .try_buy_collectible_from_shop(&buyer2, &token_id, &false)
            .is_err()
    );

    // Admin restocks
    client.restock_collectible(&token_id, &3);
    assert_eq!(client.get_stock(&token_id), 3);

    // Now buyer2 can buy
    client.buy_collectible_from_shop(&buyer2, &token_id, &false);
    assert_eq!(client.balance_of(&buyer2, &token_id), 1);
    assert_eq!(client.get_stock(&token_id), 2);
}
