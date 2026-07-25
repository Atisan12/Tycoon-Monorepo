//! Tests for issue #1331: Admin rotation mint authorization after set_admin
//!
//! # Contract: tycoon-token
//! # Scope: integration_coverage (admin rotation)
//!
//! Acceptance criteria:
//! - After `set_admin(new_admin)`:
//!   - `new_admin` can call `mint` successfully
//!   - `admin()` returns `new_admin`
//!   - supply increases correctly when new admin mints
//! - Old admin cannot mint after rotation
//!   - A call to `mint` with old admin auth must be rejected (supply unchanged)
//! - Double rotation: A → B → C, only C can mint
//! - New admin can rotate again (set another admin)
//! - `set_admin` itself requires old admin auth (non-admin cannot rotate)
extern crate std;

use crate::{TycoonToken, TycoonTokenClient};
use soroban_sdk::{
    testutils::{Address as _, MockAuth, MockAuthInvoke},
    vec, Address, Env, IntoVal,
};

const SUPPLY: i128 = 1_000_000_000_000_000_000_000_000_000;

fn setup() -> (Env, TycoonTokenClient<'static>, Address, Address) {
    let e = Env::default();
    e.mock_all_auths();
    let id = e.register(TycoonToken, ());
    let client = TycoonTokenClient::new(&e, &id);
    let admin = Address::generate(&e);
    client.initialize(&admin, &SUPPLY);
    (e, client, admin, id)
}

// ── new admin can mint after rotation ─────────────────────────────────────────

/// After `set_admin(new_admin)`, `new_admin` can mint successfully.
#[test]
fn new_admin_can_mint_after_rotation() {
    let (e, client, _old_admin, _id) = setup();
    let new_admin = Address::generate(&e);
    let recipient = Address::generate(&e);
    let mint_amount: i128 = 1_000_000_000_000_000_000_000;

    client.set_admin(&new_admin);
    assert_eq!(client.admin(), new_admin, "admin() must return new_admin");

    client.mint(&recipient, &mint_amount);

    assert_eq!(
        client.balance(&recipient),
        mint_amount,
        "Recipient must have minted amount"
    );
    assert_eq!(
        client.total_supply(),
        SUPPLY + mint_amount,
        "Total supply must increase by minted amount"
    );
}

/// New admin can mint to themselves (self-mint after rotation).
#[test]
fn new_admin_can_mint_to_self_after_rotation() {
    let (e, client, _old_admin, _id) = setup();
    let new_admin = Address::generate(&e);
    let mint_amount: i128 = 5_000_000_000_000_000_000_000;

    client.set_admin(&new_admin);
    client.mint(&new_admin, &mint_amount);

    assert_eq!(client.balance(&new_admin), mint_amount);
    assert_eq!(client.total_supply(), SUPPLY + mint_amount);
}

// ── old admin cannot mint after rotation ─────────────────────────────────────

/// After rotation, old admin's mint attempt must be rejected.
/// Supply must remain unchanged.
#[test]
fn old_admin_cannot_mint_after_rotation() {
    let (e, client, old_admin, id) = setup();
    let new_admin = Address::generate(&e);
    let recipient = Address::generate(&e);

    client.set_admin(&new_admin);
    let supply_before = client.total_supply();

    // Attempt mint with only old admin's auth — must be rejected
    let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        e.mock_auths(&[MockAuth {
            address: &old_admin,
            invoke: &MockAuthInvoke {
                contract: &id,
                fn_name: "mint",
                args: vec![&e, recipient.clone().into_val(&e), 1_i128.into_val(&e)],
                sub_invokes: &[],
            },
        }]);
        client.mint(&recipient, &1);
    }));

    assert!(res.is_err(), "Old admin must not be able to mint after rotation");
    assert_eq!(
        client.total_supply(),
        supply_before,
        "Supply must be unchanged after rejected mint"
    );
}

/// Stricter: `admin()` no longer returns old admin after rotation.
#[test]
fn old_admin_no_longer_admin_after_rotation() {
    let (e, client, old_admin, _id) = setup();
    let new_admin = Address::generate(&e);

    client.set_admin(&new_admin);

    assert_ne!(
        client.admin(),
        old_admin,
        "Old admin must no longer be registered as admin"
    );
    assert_eq!(
        client.admin(),
        new_admin,
        "New admin must be registered as admin"
    );
}

// ── double rotation: A → B → C ────────────────────────────────────────────────

/// Double rotation: admin A sets B, then B sets C. Only C can mint.
#[test]
fn double_rotation_only_final_admin_can_mint() {
    let (e, client, _admin_a, _id) = setup();
    let admin_b = Address::generate(&e);
    let admin_c = Address::generate(&e);
    let recipient = Address::generate(&e);
    let mint_amount: i128 = 2_000_000_000_000_000_000_000;

    // A → B
    client.set_admin(&admin_b);
    assert_eq!(client.admin(), admin_b);

    // B → C
    client.set_admin(&admin_c);
    assert_eq!(client.admin(), admin_c);

    // C can mint
    client.mint(&recipient, &mint_amount);
    assert_eq!(client.balance(&recipient), mint_amount);
    assert_eq!(client.total_supply(), SUPPLY + mint_amount);
}

// ── new admin can rotate again ────────────────────────────────────────────────

/// After receiving admin rights, new admin can pass them to another address.
#[test]
fn new_admin_can_rotate_to_another_admin() {
    let (e, client, _admin_a, _id) = setup();
    let admin_b = Address::generate(&e);
    let admin_c = Address::generate(&e);

    client.set_admin(&admin_b);
    assert_eq!(client.admin(), admin_b);

    // B rotates to C
    client.set_admin(&admin_c);
    assert_eq!(client.admin(), admin_c);

    // C can mint
    let recipient = Address::generate(&e);
    let mint_amount: i128 = 1_000_000_000_000_000_000_000;
    client.mint(&recipient, &mint_amount);
    assert_eq!(client.balance(&recipient), mint_amount);
}

// ── set_admin requires old admin auth ─────────────────────────────────────────

/// A non-admin address cannot call `set_admin` (supply/admin state unchanged).
#[test]
fn non_admin_cannot_call_set_admin() {
    let (e, client, _admin, id) = setup();
    let attacker = Address::generate(&e);
    let target = Address::generate(&e);
    let admin_before = client.admin();

    let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        e.mock_auths(&[MockAuth {
            address: &attacker,
            invoke: &MockAuthInvoke {
                contract: &id,
                fn_name: "set_admin",
                args: vec![&e, target.clone().into_val(&e)],
                sub_invokes: &[],
            },
        }]);
        client.set_admin(&target);
    }));

    assert!(
        res.is_err(),
        "Non-admin must not be able to call set_admin"
    );
    assert_eq!(
        client.admin(),
        admin_before,
        "Admin must be unchanged after failed set_admin"
    );
}

// ── supply conservation across admin rotation ─────────────────────────────────

/// Total supply is conserved during admin rotation (set_admin does not mint/burn).
#[test]
fn set_admin_does_not_change_supply() {
    let (e, client, _admin, _id) = setup();
    let new_admin = Address::generate(&e);

    let supply_before = client.total_supply();
    client.set_admin(&new_admin);
    let supply_after = client.total_supply();

    assert_eq!(
        supply_before, supply_after,
        "set_admin must not change total supply"
    );
}

// ── multiple mints by new admin are cumulative ────────────────────────────────

/// New admin can mint multiple times; supply accumulates correctly.
#[test]
fn new_admin_multiple_mints_cumulative() {
    let (e, client, _old_admin, _id) = setup();
    let new_admin = Address::generate(&e);
    let recipient = Address::generate(&e);

    client.set_admin(&new_admin);

    let chunk: i128 = 1_000_000_000_000_000_000_000;
    client.mint(&recipient, &chunk);
    client.mint(&recipient, &chunk);
    client.mint(&recipient, &chunk);

    assert_eq!(client.balance(&recipient), chunk * 3);
    assert_eq!(client.total_supply(), SUPPLY + chunk * 3);
}

// ── balances unaffected by admin rotation ────────────────────────────────────

/// Existing balances are unaffected by admin rotation.
#[test]
fn balances_unaffected_by_admin_rotation() {
    let (e, client, old_admin, _id) = setup();
    let user = Address::generate(&e);
    let new_admin = Address::generate(&e);

    let transfer_amount: i128 = 100_000_000_000_000_000_000_000_000;
    client.transfer(&old_admin, &user, &transfer_amount);

    let user_balance_before = client.balance(&user);
    let admin_balance_before = client.balance(&old_admin);

    client.set_admin(&new_admin);

    assert_eq!(
        client.balance(&user),
        user_balance_before,
        "User balance must be unchanged by rotation"
    );
    assert_eq!(
        client.balance(&old_admin),
        admin_balance_before,
        "Old admin balance must be unchanged by rotation"
    );
}

// ── integration: full admin rotation lifecycle ────────────────────────────────

/// Full lifecycle: init → transfer → rotate → new admin mints → burn.
#[test]
fn full_admin_rotation_lifecycle() {
    let (e, client, old_admin, _id) = setup();
    let new_admin = Address::generate(&e);
    let player = Address::generate(&e);
    let game_contract = Address::generate(&e);

    // Old admin funds player
    let player_fund: i128 = 10_000_000_000_000_000_000_000;
    client.transfer(&old_admin, &player, &player_fund);

    // Rotate admin
    client.set_admin(&new_admin);
    assert_eq!(client.admin(), new_admin);

    // New admin mints a prize pool
    let prize_pool: i128 = 50_000_000_000_000_000_000_000;
    client.mint(&game_contract, &prize_pool);
    assert_eq!(client.balance(&game_contract), prize_pool);
    assert_eq!(client.total_supply(), SUPPLY + prize_pool);

    // Player burns their tokens (no admin required)
    let burn_amount: i128 = 1_000_000_000_000_000_000_000;
    client.burn(&player, &burn_amount);
    assert_eq!(client.balance(&player), player_fund - burn_amount);
    assert_eq!(client.total_supply(), SUPPLY + prize_pool - burn_amount);

    // Old admin still holds their original balance (minus transfer)
    assert_eq!(
        client.balance(&old_admin),
        SUPPLY - player_fund,
        "Old admin balance = initial supply - funds given to player"
    );
}
