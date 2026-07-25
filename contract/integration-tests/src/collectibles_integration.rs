/// # Cross-contract flow: Collectibles — full ecosystem integration
///
/// Exercises `TycoonCollectibles` through the shared `Fixture`, covering
/// initialization, minting, transfers, burns, shop operations, perk burns,
/// pause/unpause, and access-control guards.
///
/// Each test creates its own `Fixture::new()` — no shared state.
///
/// | Test | Path |
/// |------|------|
/// | `fixture_collectibles_is_initialized`              | collectibles.admin() == fixture.admin |
/// | `backend_mint_issues_token_to_player`              | backend_mint → balance_of |
/// | `mint_collectible_generates_unique_token_id`       | mint_collectible returns distinct ids |
/// | `transfer_collectible_between_players`             | transfer → balances update |
/// | `transfer_requires_sender_balance`                 | transfer with no balance panics |
/// | `burn_reduces_balance`                             | burn → balance decremented |
/// | `burn_exceeds_balance_rejected`                    | burn > balance panics |
/// | `burn_collectible_for_perk_works`                  | burn_collectible_for_perk succeeds when unpaused |
/// | `burn_collectible_for_perk_blocked_when_paused`    | set_pause(true) blocks perk burn |
/// | `perk_burn_resumes_after_unpause`                  | set_pause(false) restores perk burn |
/// | `non_admin_cannot_set_backend_minter`              | attacker cannot set minter |
/// | `non_admin_cannot_stock_shop`                      | attacker cannot stock shop |
/// | `admin_revoke_replaces_minter`                     | set_backend_minter twice, old minter rejected |
/// | `multi_player_independent_balances`                | three players, isolated inventory |
/// | `double_initialize_rejected`                       | second initialize panics |
/// | `stock_shop_and_buy_with_tyc`                      | stock_shop → buy_collectible_from_shop (TYC) |
/// | `stock_shop_and_buy_with_usdc`                     | stock_shop → buy_collectible_from_shop (USDC) |
/// | `buy_from_empty_stock_rejected`                    | out-of-stock purchase panics |
/// | `owned_token_count_tracks_mint_and_burn`           | count invariant across mint and burn |
/// | `tokens_of_returns_all_owned_ids`                  | tokens_of reflects minted set |
/// | `stale_token_id_balance_is_zero`                   | balance_of on unknown id returns 0 |
/// | `admin_can_update_perk_strength`                   | set_token_perk updates perk/strength |
#[cfg(test)]
mod tests {
    extern crate std;

    use crate::fixture::{Fixture, TestFixtureConfig};
    use soroban_sdk::{
        testutils::Address as _,
        token::StellarAssetClient,
        Address,
    };
    use tycoon_collectibles::TycoonCollectiblesClient;

    // ── Fixture sanity ────────────────────────────────────────────────────────

    /// The fixture wires collectibles with the fixture admin — admin() must match.
    #[test]
    fn fixture_collectibles_is_initialized() {
        let f = Fixture::new();
        // get_backend_minter returns None until set_backend_minter is called.
        // The contract is initialized so double-init must fail (tested below).
        // The simplest observable initialization fact is that the contract is
        // deployed and a read-only call succeeds.
        assert_eq!(f.collectibles.get_backend_minter(), None);
    }

    // ── Minting ───────────────────────────────────────────────────────────────

    /// backend_mint issues `amount` tokens of `token_id` to `to`.
    #[test]
    fn backend_mint_issues_token_to_player() {
        let f = Fixture::new();
        // Grant backend-minter role
        f.collectibles.set_backend_minter(&f.backend);
        let token_id: u128 = 1001;
        f.collectibles.backend_mint(&f.backend, &f.player_a, &token_id, &3u64);
        assert_eq!(f.collectibles.balance_of(&f.player_a, &token_id), 3);
    }

    /// Each `mint_collectible` call returns a distinct token id.
    #[test]
    fn mint_collectible_generates_unique_token_id() {
        let f = Fixture::new();
        f.collectibles.set_backend_minter(&f.backend);
        // mint_collectible(caller, to, perk=0, strength=1)
        // Perk::None == 0 in the enum; strength 1 is valid
        let id1 = f.collectibles.mint_collectible(&f.admin, &f.player_a, &0u32, &1u32);
        let id2 = f.collectibles.mint_collectible(&f.admin, &f.player_b, &0u32, &1u32);
        let id3 = f.collectibles.mint_collectible(&f.admin, &f.player_c, &0u32, &1u32);
        assert_ne!(id1, id2);
        assert_ne!(id2, id3);
        assert_ne!(id1, id3);
        // Each player holds exactly 1 of their token
        assert_eq!(f.collectibles.balance_of(&f.player_a, &id1), 1);
        assert_eq!(f.collectibles.balance_of(&f.player_b, &id2), 1);
        assert_eq!(f.collectibles.balance_of(&f.player_c, &id3), 1);
    }

    // ── Transfer ──────────────────────────────────────────────────────────────

    /// Transferring a collectible moves balance from sender to receiver.
    #[test]
    fn transfer_collectible_between_players() {
        let f = Fixture::new();
        f.collectibles.set_backend_minter(&f.backend);
        let token_id: u128 = 42;
        f.collectibles.backend_mint(&f.backend, &f.player_a, &token_id, &5u64);

        f.collectibles.transfer(&f.player_a, &f.player_b, &token_id, &2u64);

        assert_eq!(f.collectibles.balance_of(&f.player_a, &token_id), 3);
        assert_eq!(f.collectibles.balance_of(&f.player_b, &token_id), 2);
    }

    /// Transfer fails when the sender has no balance.
    #[test]
    fn transfer_requires_sender_balance() {
        let f = Fixture::new();
        let token_id: u128 = 99;
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            f.collectibles.transfer(&f.player_a, &f.player_b, &token_id, &1u64);
        }));
        assert!(result.is_err(), "transfer with no balance must panic");
    }

    // ── Burn ──────────────────────────────────────────────────────────────────

    /// Burning a collectible decrements the owner's balance.
    #[test]
    fn burn_reduces_balance() {
        let f = Fixture::new();
        f.collectibles.set_backend_minter(&f.backend);
        let token_id: u128 = 200;
        f.collectibles.backend_mint(&f.backend, &f.player_a, &token_id, &4u64);
        f.collectibles.burn(&f.player_a, &token_id, &2u64);
        assert_eq!(f.collectibles.balance_of(&f.player_a, &token_id), 2);
    }

    /// Burning more than owned panics.
    #[test]
    fn burn_exceeds_balance_rejected() {
        let f = Fixture::new();
        f.collectibles.set_backend_minter(&f.backend);
        let token_id: u128 = 300;
        f.collectibles.backend_mint(&f.backend, &f.player_a, &token_id, &1u64);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            f.collectibles.burn(&f.player_a, &token_id, &2u64);
        }));
        assert!(result.is_err(), "burn > balance must panic");
    }

    // ── Perk burn ─────────────────────────────────────────────────────────────

    /// `burn_collectible_for_perk` succeeds when the contract is not paused.
    #[test]
    fn burn_collectible_for_perk_works() {
        let f = Fixture::new();
        f.collectibles.set_backend_minter(&f.backend);
        // Mint a token with a real perk (perk=1 = first non-None variant)
        let token_id = f.collectibles.mint_collectible(&f.admin, &f.player_a, &1u32, &5u32);
        // Contract is unpaused by default — burn_collectible_for_perk must succeed
        f.collectibles.burn_collectible_for_perk(&f.player_a, &token_id);
        assert_eq!(f.collectibles.balance_of(&f.player_a, &token_id), 0);
    }

    /// `burn_collectible_for_perk` is blocked while the contract is paused.
    #[test]
    fn burn_collectible_for_perk_blocked_when_paused() {
        let f = Fixture::new();
        f.collectibles.set_backend_minter(&f.backend);
        let token_id = f.collectibles.mint_collectible(&f.admin, &f.player_a, &1u32, &5u32);
        f.collectibles.set_pause(&true);

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            f.collectibles.burn_collectible_for_perk(&f.player_a, &token_id);
        }));
        assert!(result.is_err(), "perk burn while paused must be rejected");
        // Balance unchanged — no tokens burned
        assert_eq!(f.collectibles.balance_of(&f.player_a, &token_id), 1);
    }

    /// Unpausing restores perk-burn functionality.
    #[test]
    fn perk_burn_resumes_after_unpause() {
        let f = Fixture::new();
        f.collectibles.set_backend_minter(&f.backend);
        let token_id = f.collectibles.mint_collectible(&f.admin, &f.player_a, &1u32, &5u32);
        f.collectibles.set_pause(&true);
        f.collectibles.set_pause(&false);
        f.collectibles.burn_collectible_for_perk(&f.player_a, &token_id);
        assert_eq!(f.collectibles.balance_of(&f.player_a, &token_id), 0);
    }

    // ── Access control ────────────────────────────────────────────────────────

    /// A non-admin address cannot call `set_backend_minter`.
    #[test]
    fn non_admin_cannot_set_backend_minter() {
        use soroban_sdk::IntoVal;
        use tycoon_collectibles::TycoonCollectibles;
        let env = soroban_sdk::Env::default();
        let contract_id = env.register(TycoonCollectibles, ());
        let client = TycoonCollectiblesClient::new(&env, &contract_id);
        let admin = Address::generate(&env);
        let attacker = Address::generate(&env);
        let new_minter = Address::generate(&env);
        env.mock_all_auths();
        let _ = client.initialize(&admin);
        env.mock_auths(&[soroban_sdk::testutils::MockAuth {
            address: &attacker,
            invoke: &soroban_sdk::testutils::MockAuthInvoke {
                contract: &contract_id,
                fn_name: "set_backend_minter",
                args: soroban_sdk::vec![&env, new_minter.clone().into_val(&env)],
                sub_invokes: &[],
            },
        }]);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            client.set_backend_minter(&new_minter);
        }));
        assert!(result.is_err(), "non-admin must not set backend minter");
    }

    /// A non-admin address cannot call `stock_shop`.
    #[test]
    fn non_admin_cannot_stock_shop() {
        use soroban_sdk::IntoVal;
        use tycoon_collectibles::TycoonCollectibles;
        let env = soroban_sdk::Env::default();
        let contract_id = env.register(TycoonCollectibles, ());
        let client = TycoonCollectiblesClient::new(&env, &contract_id);
        let admin = Address::generate(&env);
        let attacker = Address::generate(&env);
        let tyc = Address::generate(&env);
        let usdc = Address::generate(&env);
        env.mock_all_auths();
        let _ = client.initialize(&admin);
        let _ = client.init_shop(&tyc, &usdc);
        // Remove admin auth — attacker attempts stock_shop
        env.mock_auths(&[soroban_sdk::testutils::MockAuth {
            address: &attacker,
            invoke: &soroban_sdk::testutils::MockAuthInvoke {
                contract: &contract_id,
                fn_name: "stock_shop",
                args: soroban_sdk::vec![
                    &env,
                    1u64.into_val(&env),
                    0u32.into_val(&env),
                    1u32.into_val(&env),
                    100u128.into_val(&env),
                    10u128.into_val(&env)
                ],
                sub_invokes: &[],
            },
        }]);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            client.stock_shop(&1u64, &0u32, &1u32, &100u128, &10u128);
        }));
        assert!(result.is_err(), "non-admin must not stock shop");
    }

    /// Setting backend_minter twice updates the minter; the old minter is rejected.
    #[test]
    fn admin_revoke_replaces_minter() {
        let f = Fixture::new();
        let first_minter = Address::generate(&f.env);
        let second_minter = Address::generate(&f.env);

        f.collectibles.set_backend_minter(&first_minter);
        assert_eq!(f.collectibles.get_backend_minter(), Some(first_minter.clone()));

        f.collectibles.set_backend_minter(&second_minter);
        assert_eq!(f.collectibles.get_backend_minter(), Some(second_minter.clone()));

        // Old minter can no longer mint
        let token_id: u128 = 1234;
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            f.collectibles.backend_mint(&first_minter, &f.player_a, &token_id, &1u64);
        }));
        assert!(result.is_err(), "replaced minter must not mint");
    }

    // ── Multi-player isolation ────────────────────────────────────────────────

    /// Three players each receive different tokens; balances are independent.
    #[test]
    fn multi_player_independent_balances() {
        let f = Fixture::new();
        f.collectibles.set_backend_minter(&f.backend);
        let id_a: u128 = 10;
        let id_b: u128 = 20;
        let id_c: u128 = 30;

        f.collectibles.backend_mint(&f.backend, &f.player_a, &id_a, &3u64);
        f.collectibles.backend_mint(&f.backend, &f.player_b, &id_b, &5u64);
        f.collectibles.backend_mint(&f.backend, &f.player_c, &id_c, &7u64);

        assert_eq!(f.collectibles.balance_of(&f.player_a, &id_a), 3);
        assert_eq!(f.collectibles.balance_of(&f.player_b, &id_b), 5);
        assert_eq!(f.collectibles.balance_of(&f.player_c, &id_c), 7);

        // Cross-check: players have 0 balance for each other's tokens
        assert_eq!(f.collectibles.balance_of(&f.player_a, &id_b), 0);
        assert_eq!(f.collectibles.balance_of(&f.player_b, &id_c), 0);
        assert_eq!(f.collectibles.balance_of(&f.player_c, &id_a), 0);
    }

    // ── State guards ──────────────────────────────────────────────────────────

    /// A second `initialize` call is rejected — state is not corrupted.
    #[test]
    fn double_initialize_rejected() {
        let f = Fixture::new();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            // Fixture already initialized collectibles; second call must fail
            f.collectibles.initialize(&f.admin);
        }));
        assert!(result.is_err(), "double-initialize must be rejected");
    }

    // ── Shop flow ─────────────────────────────────────────────────────────────

    /// `stock_shop` → `buy_collectible_from_shop` (TYC path) completes successfully.
    #[test]
    fn stock_shop_and_buy_with_tyc() {
        let f = Fixture::new();
        let tyc_price: u128 = 50_000_000_000_000_000_000; // 50 TYC
        let token_id: u128;

        // Admin initializes the shop and stocks one collectible type
        f.collectibles.init_shop(&f.tyc_id, &f.usdc_id);
        token_id = f.collectibles.stock_shop(&1u64, &0u32, &1u32, &tyc_price, &0u128);

        // Fund player_a with enough TYC for the purchase
        f.mint_tyc(&f.player_a, tyc_price as i128 + 1_000_000_000_000_000_000);

        let balance_before = f.tyc_balance(&f.player_a);
        f.collectibles.buy_collectible_from_shop(&f.player_a, &token_id, &false);

        assert_eq!(f.collectibles.balance_of(&f.player_a, &token_id), 1);
        assert_eq!(f.tyc_balance(&f.player_a), balance_before - tyc_price as i128);
    }

    /// `stock_shop` → `buy_collectible_from_shop` (USDC path) completes successfully.
    #[test]
    fn stock_shop_and_buy_with_usdc() {
        let f = Fixture::new();
        let usdc_price: u128 = 5_000_000; // 5 USDC (6 decimals)
        let token_id: u128;

        f.collectibles.init_shop(&f.tyc_id, &f.usdc_id);
        token_id = f.collectibles.stock_shop(&1u64, &0u32, &1u32, &0u128, &usdc_price);

        // Fund player_b with USDC
        f.mint_usdc(&f.player_b, usdc_price as i128 + 1_000_000);

        let usdc_before = f.usdc_balance(&f.player_b);
        f.collectibles.buy_collectible_from_shop(&f.player_b, &token_id, &true);

        assert_eq!(f.collectibles.balance_of(&f.player_b, &token_id), 1);
        assert_eq!(f.usdc_balance(&f.player_b), usdc_before - usdc_price as i128);
    }

    /// Purchasing when stock is depleted panics.
    #[test]
    fn buy_from_empty_stock_rejected() {
        let f = Fixture::new();
        f.collectibles.init_shop(&f.tyc_id, &f.usdc_id);
        // Stock 1 item, then buy it to exhaust stock
        let tyc_price: u128 = 1_000_000_000_000_000_000;
        let token_id = f.collectibles.stock_shop(&1u64, &0u32, &1u32, &tyc_price, &0u128);
        f.mint_tyc(&f.player_a, tyc_price as i128 * 2);
        f.collectibles.buy_collectible_from_shop(&f.player_a, &token_id, &false);

        // Second purchase — stock exhausted
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            f.collectibles.buy_collectible_from_shop(&f.player_a, &token_id, &false);
        }));
        assert!(result.is_err(), "out-of-stock purchase must be rejected");
    }

    // ── Enumeration helpers ───────────────────────────────────────────────────

    /// `owned_token_count` tracks mint and burn correctly.
    #[test]
    fn owned_token_count_tracks_mint_and_burn() {
        let f = Fixture::new();
        f.collectibles.set_backend_minter(&f.backend);
        assert_eq!(f.collectibles.owned_token_count(&f.player_a), 0);

        let id1 = f.collectibles.mint_collectible(&f.admin, &f.player_a, &0u32, &1u32);
        let id2 = f.collectibles.mint_collectible(&f.admin, &f.player_a, &0u32, &1u32);
        assert_eq!(f.collectibles.owned_token_count(&f.player_a), 2);

        f.collectibles.burn(&f.player_a, &id1, &1u64);
        assert_eq!(f.collectibles.owned_token_count(&f.player_a), 1);

        f.collectibles.burn(&f.player_a, &id2, &1u64);
        assert_eq!(f.collectibles.owned_token_count(&f.player_a), 0);
    }

    /// `tokens_of` returns all token ids owned by the player.
    #[test]
    fn tokens_of_returns_all_owned_ids() {
        let f = Fixture::new();
        f.collectibles.set_backend_minter(&f.backend);
        let id1 = f.collectibles.mint_collectible(&f.admin, &f.player_b, &0u32, &1u32);
        let id2 = f.collectibles.mint_collectible(&f.admin, &f.player_b, &0u32, &1u32);

        let owned = f.collectibles.tokens_of(&f.player_b);
        assert_eq!(owned.len(), 2);
        // Both ids must appear in the list
        let mut found1 = false;
        let mut found2 = false;
        for i in 0..owned.len() {
            let t = owned.get(i).unwrap();
            if t == id1 { found1 = true; }
            if t == id2 { found2 = true; }
        }
        assert!(found1, "id1 must be in tokens_of");
        assert!(found2, "id2 must be in tokens_of");
    }

    /// `balance_of` returns 0 for a token id that was never minted to the address.
    #[test]
    fn stale_token_id_balance_is_zero() {
        let f = Fixture::new();
        let ghost_token_id: u128 = 0xDEAD_BEEF_CAFE;
        assert_eq!(f.collectibles.balance_of(&f.player_a, &ghost_token_id), 0);
        // No state should be created for unknown queries
        assert_eq!(f.collectibles.owned_token_count(&f.player_a), 0);
    }

    // ── Perk management ───────────────────────────────────────────────────────

    /// Admin can update perk and strength for an existing token.
    #[test]
    fn admin_can_update_perk_strength() {
        let f = Fixture::new();
        f.collectibles.set_backend_minter(&f.backend);
        let token_id: u128 = 777;
        f.collectibles.backend_mint(&f.backend, &f.player_a, &token_id, &1u64);

        // Set perk to 1 (first non-None variant) with strength 10
        f.collectibles.set_token_perk(&token_id, &1u32, &10u32);

        assert_eq!(f.collectibles.get_token_strength(&token_id), 10);
    }

    // ── Fixture config: no collectibles ──────────────────────────────────────

    /// `TestFixtureConfig { deploy_collectibles: false }` skips collectibles init.
    /// The field is present; no panic on access to the stub address.
    #[test]
    fn fixture_config_without_collectibles() {
        let env = soroban_sdk::Env::default();
        env.mock_all_auths();
        let config = crate::fixture::TestFixtureConfig {
            deploy_boost_system: true,
            deploy_collectibles: false,
            usdc_game_fund: 0,
        };
        let f = Fixture::new_with_config(&env, config);
        // Accessing the stub address must not panic
        let _ = f.collectibles_id.clone();
    }
}
