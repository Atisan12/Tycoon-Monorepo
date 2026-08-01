//! SW-CT-PAUSE-001: Pause flag — mutation entrypoints must return ContractPaused
//! when the contract is paused; read-only queries must remain unaffected.
//!
//! Covered entrypoints:
//!   - stock_shop
//!   - restock_collectible
//!   - buy_collectible_from_shop
//!   - buy_collectible
//!   - transfer
//!   - burn
//!   - burn_collectible_for_perk
//!   - backend_mint
//!   - mint_collectible
//!
//! Read-only queries verified to pass while paused:
//!   - is_contract_paused, balance_of, get_stock, get_token_perk, tokens_of

extern crate std;

use crate::{CollectibleError, TycoonCollectibles, TycoonCollectiblesClient};
use soroban_sdk::{testutils::Address as _, token::StellarAssetClient, Address, Env};

// ── helpers ───────────────────────────────────────────────────────────────────

fn setup(env: &Env) -> (TycoonCollectiblesClient<'_>, Address, Address) {
    let id = env.register(TycoonCollectibles, ());
    let client = TycoonCollectiblesClient::new(env, &id);
    let admin = Address::generate(env);
    client.initialize(&admin);
    (client, admin, id)
}

fn make_token(env: &Env, admin: &Address) -> Address {
    env.register_stellar_asset_contract_v2(admin.clone())
        .address()
}

fn assert_paused_err(
    result: Result<impl core::fmt::Debug, Result<CollectibleError, soroban_sdk::InvokeError>>,
) {
    match result {
        Err(Ok(e)) => assert_eq!(
            e,
            CollectibleError::ContractPaused,
            "expected ContractPaused"
        ),
        other => panic!("expected ContractPaused error, got {:?}", other),
    }
}

// ── stock_shop blocked when paused ───────────────────────────────────────────

#[test]
fn test_stock_shop_blocked_when_paused() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _) = setup(&env);

    client.set_pause(&true);
    assert_paused_err(client.try_stock_shop(&10, &1, &1, &100, &0));
}

#[test]
fn test_stock_shop_allowed_after_unpause() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _) = setup(&env);

    client.set_pause(&true);
    client.set_pause(&false);
    let id = client.stock_shop(&5, &3, &0, &50, &0);
    assert_eq!(client.get_stock(&id), 5);
}

// ── restock_collectible blocked when paused ───────────────────────────────────

#[test]
fn test_restock_blocked_when_paused() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _) = setup(&env);

    let id = client.stock_shop(&5, &1, &1, &100, &0);
    client.set_pause(&true);
    assert_paused_err(client.try_restock_collectible(&id, &5));
}

// ── buy_collectible_from_shop blocked when paused ─────────────────────────────

#[test]
fn test_buy_from_shop_blocked_when_paused() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, _) = setup(&env);

    let tyc = make_token(&env, &admin);
    let usdc = make_token(&env, &admin);
    client.init_shop(&tyc, &usdc);
    let id = client.stock_shop(&5, &3, &0, &100, &0);

    let buyer = Address::generate(&env);
    StellarAssetClient::new(&env, &tyc).mint(&buyer, &500);

    client.set_pause(&true);
    assert_paused_err(client.try_buy_collectible_from_shop(&buyer, &id, &false));

    // Balance unchanged
    assert_eq!(client.balance_of(&buyer, &id), 0);
    assert_eq!(client.get_stock(&id), 5);
}

#[test]
fn test_buy_from_shop_allowed_after_unpause() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, _) = setup(&env);

    let tyc = make_token(&env, &admin);
    let usdc = make_token(&env, &admin);
    client.init_shop(&tyc, &usdc);
    let id = client.stock_shop(&5, &3, &0, &100, &0);

    let buyer = Address::generate(&env);
    StellarAssetClient::new(&env, &tyc).mint(&buyer, &500);

    client.set_pause(&true);
    client.set_pause(&false);
    client.buy_collectible_from_shop(&buyer, &id, &false);
    assert_eq!(client.balance_of(&buyer, &id), 1);
}

// ── buy_collectible (raw) blocked when paused ─────────────────────────────────

#[test]
fn test_buy_collectible_blocked_when_paused() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _) = setup(&env);

    let buyer = Address::generate(&env);
    client.set_pause(&true);
    assert_paused_err(client.try_buy_collectible(&buyer, &1, &1));
    assert_eq!(client.balance_of(&buyer, &1), 0);
}

// ── transfer blocked when paused ─────────────────────────────────────────────

#[test]
fn test_transfer_blocked_when_paused() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _) = setup(&env);

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    client.buy_collectible(&alice, &1, &5);

    client.set_pause(&true);
    assert_paused_err(client.try_transfer(&alice, &bob, &1, &2));

    // Balances unchanged
    assert_eq!(client.balance_of(&alice, &1), 5);
    assert_eq!(client.balance_of(&bob, &1), 0);
}

#[test]
fn test_transfer_allowed_after_unpause() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _) = setup(&env);

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    client.buy_collectible(&alice, &1, &5);

    client.set_pause(&true);
    client.set_pause(&false);
    client.transfer(&alice, &bob, &1, &3);
    assert_eq!(client.balance_of(&alice, &1), 2);
    assert_eq!(client.balance_of(&bob, &1), 3);
}

// ── burn blocked when paused ──────────────────────────────────────────────────

#[test]
fn test_burn_blocked_when_paused() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _) = setup(&env);

    let user = Address::generate(&env);
    client.buy_collectible(&user, &1, &3);

    client.set_pause(&true);
    assert_paused_err(client.try_burn(&user, &1, &1));
    assert_eq!(client.balance_of(&user, &1), 3);
}

// ── burn_collectible_for_perk blocked when paused (pre-existing) ──────────────

#[test]
fn test_burn_for_perk_blocked_when_paused() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _) = setup(&env);

    let user = Address::generate(&env);
    client.buy_collectible(&user, &1, &1);
    client.set_token_perk(&1, &crate::types::Perk::RentBoost, &1);

    client.set_pause(&true);
    assert_paused_err(client.try_burn_collectible_for_perk(&user, &1));
    assert_eq!(client.balance_of(&user, &1), 1);
}

// ── backend_mint blocked when paused ─────────────────────────────────────────

#[test]
fn test_backend_mint_blocked_when_paused() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, _) = setup(&env);

    let user = Address::generate(&env);
    client.set_pause(&true);
    assert_paused_err(client.try_backend_mint(&admin, &user, &10, &1));
    assert_eq!(client.balance_of(&user, &10), 0);
}

// ── mint_collectible blocked when paused ─────────────────────────────────────

#[test]
fn test_mint_collectible_blocked_when_paused() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, _) = setup(&env);

    let user = Address::generate(&env);
    client.set_pause(&true);
    assert_paused_err(client.try_mint_collectible(&admin, &user, &3, &1));
}

#[test]
fn test_mint_collectible_allowed_after_unpause() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, _) = setup(&env);

    let user = Address::generate(&env);
    client.set_pause(&true);
    client.set_pause(&false);
    let id = client.mint_collectible(&admin, &user, &5, &1);
    assert_eq!(client.balance_of(&user, &id), 1);
}

// ── read-only queries unaffected by pause ─────────────────────────────────────

#[test]
fn test_read_queries_work_while_paused() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _) = setup(&env);

    let user = Address::generate(&env);
    client.buy_collectible(&user, &42, &7);
    client.set_token_perk(&42, &crate::types::Perk::Shield, &1);

    client.set_pause(&true);

    assert!(client.is_contract_paused());
    assert_eq!(client.balance_of(&user, &42), 7);
    assert_eq!(client.get_token_perk(&42), crate::types::Perk::Shield);
    assert_eq!(client.owned_token_count(&user), 1);
}

// ── pause is idempotent ───────────────────────────────────────────────────────

#[test]
fn test_pause_idempotent() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _) = setup(&env);

    client.set_pause(&true);
    client.set_pause(&true); // no-op, must not error
    assert!(client.is_contract_paused());

    client.set_pause(&false);
    client.set_pause(&false); // no-op
    assert!(!client.is_contract_paused());
}
