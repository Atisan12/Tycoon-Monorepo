/// # Gas snapshot tests — `_mint` / `_burn` storage-op accounting (#1356)
///
/// These tests verify the storage-operation counts documented in
/// `contract/GAS_SNAPSHOT_DIFF.md` for the hot-path `_mint` and `_burn`
/// helpers. They use Soroban's built-in `env.cost_estimate()` is not yet
/// stable in the current SDK version, so we instead assert observable
/// side-effects that correspond to each storage branch described in the doc:
///
/// | Scenario | Expected behaviour |
/// |----------|--------------------|
/// | First mint (balance 0 → N) | Balance set + OwnedTokenCount set |
/// | Subsequent mint (balance N → M) | Balance set only (no count touch) |
/// | Burn partial (balance N → M > 0) | Balance set only |
/// | Burn to zero (balance N → 0) | Balance removed + OwnedTokenCount decremented |
/// | Burn single token (count → 0) | Balance removed + OwnedTokenCount removed |
#[cfg(test)]
mod tests {
    use crate::{TycoonRewardSystem, TycoonRewardSystemClient};
    use soroban_sdk::{testutils::Address as _, Address, Env};

    fn setup() -> (Env, TycoonRewardSystemClient<'static>, Address) {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let tyc = env
            .register_stellar_asset_contract_v2(Address::generate(&env))
            .address();
        let usdc = env
            .register_stellar_asset_contract_v2(Address::generate(&env))
            .address();

        let id = env.register(TycoonRewardSystem, ());
        let client = TycoonRewardSystemClient::new(&env, &id);
        client.initialize(&admin, &tyc, &usdc);
        (env, client, admin)
    }

    // ── _mint ─────────────────────────────────────────────────────────────────

    /// First mint (balance 0 → 1): OwnedTokenCount transitions from 0 to 1.
    #[test]
    fn mint_first_increments_owned_token_count() {
        let (env, client, _) = setup();
        let user = Address::generate(&env);
        let token_id: u128 = 1;

        assert_eq!(client.get_balance(&user, &token_id), 0);
        assert_eq!(client.owned_token_count(&user), 0);

        client.test_mint(&user, &token_id, &1);

        assert_eq!(client.get_balance(&user, &token_id), 1);
        // Balance 0→1: OwnedTokenCount must increment
        assert_eq!(client.owned_token_count(&user), 1);
    }

    /// Subsequent mint (balance already > 0): OwnedTokenCount must NOT change.
    #[test]
    fn mint_subsequent_does_not_touch_owned_token_count() {
        let (env, client, _) = setup();
        let user = Address::generate(&env);
        let token_id: u128 = 2;

        client.test_mint(&user, &token_id, &1);
        assert_eq!(client.owned_token_count(&user), 1);

        // Second mint: count must stay at 1
        client.test_mint(&user, &token_id, &3);
        assert_eq!(client.get_balance(&user, &token_id), 4);
        assert_eq!(
            client.owned_token_count(&user),
            1,
            "count must not change on subsequent mint"
        );
    }

    /// Zero-amount mint is a no-op: no state changes.
    #[test]
    fn mint_zero_is_noop() {
        let (env, client, _) = setup();
        let user = Address::generate(&env);
        let token_id: u128 = 3;

        client.test_mint(&user, &token_id, &0);

        assert_eq!(client.get_balance(&user, &token_id), 0);
        assert_eq!(client.owned_token_count(&user), 0);
    }

    /// Multiple distinct token IDs each contribute to OwnedTokenCount.
    #[test]
    fn mint_multiple_tokens_accumulates_count() {
        let (env, client, _) = setup();
        let user = Address::generate(&env);

        for token_id in 10u128..15 {
            client.test_mint(&user, &token_id, &1);
        }

        assert_eq!(client.owned_token_count(&user), 5);
    }

    // ── _burn ─────────────────────────────────────────────────────────────────

    /// Partial burn (balance N → M > 0): OwnedTokenCount must NOT change.
    #[test]
    fn burn_partial_does_not_touch_owned_token_count() {
        let (env, client, _) = setup();
        let user = Address::generate(&env);
        let token_id: u128 = 20;

        client.test_mint(&user, &token_id, &5);
        assert_eq!(client.owned_token_count(&user), 1);

        client.test_burn(&user, &token_id, &3);

        assert_eq!(client.get_balance(&user, &token_id), 2);
        assert_eq!(
            client.owned_token_count(&user),
            1,
            "partial burn must not change count"
        );
    }

    /// Burn to zero (balance N → 0): balance entry removed, OwnedTokenCount decremented.
    #[test]
    fn burn_to_zero_removes_balance_and_decrements_count() {
        let (env, client, _) = setup();
        let user = Address::generate(&env);
        let token_id: u128 = 21;

        client.test_mint(&user, &token_id, &2);
        assert_eq!(client.owned_token_count(&user), 1);

        client.test_burn(&user, &token_id, &2);

        // Balance entry removed — reads back as 0
        assert_eq!(client.get_balance(&user, &token_id), 0);
        // OwnedTokenCount decremented to 0 and removed
        assert_eq!(client.owned_token_count(&user), 0);
    }

    /// Burn last of multiple tokens: only that token's count decrements.
    #[test]
    fn burn_one_of_many_decrements_count_by_one() {
        let (env, client, _) = setup();
        let user = Address::generate(&env);

        client.test_mint(&user, &100, &1);
        client.test_mint(&user, &101, &1);
        assert_eq!(client.owned_token_count(&user), 2);

        client.test_burn(&user, &100, &1);

        assert_eq!(client.get_balance(&user, &100), 0);
        assert_eq!(client.get_balance(&user, &101), 1);
        assert_eq!(client.owned_token_count(&user), 1);
    }

    /// Zero-amount burn is a no-op: no state changes.
    #[test]
    fn burn_zero_is_noop() {
        let (env, client, _) = setup();
        let user = Address::generate(&env);
        let token_id: u128 = 30;

        client.test_mint(&user, &token_id, &3);
        client.test_burn(&user, &token_id, &0);

        assert_eq!(client.get_balance(&user, &token_id), 3);
        assert_eq!(client.owned_token_count(&user), 1);
    }

    /// Burn more than balance must panic.
    #[test]
    #[should_panic(expected = "Insufficient balance")]
    fn burn_exceeds_balance_panics() {
        let (env, client, _) = setup();
        let user = Address::generate(&env);
        let token_id: u128 = 40;

        client.test_mint(&user, &token_id, &1);
        client.test_burn(&user, &token_id, &2);
    }

    // ── mint + burn round-trip ────────────────────────────────────────────────

    /// Full mint → burn cycle: state returns to clean slate.
    #[test]
    fn mint_burn_round_trip_cleans_state() {
        let (env, client, _) = setup();
        let user = Address::generate(&env);
        let token_id: u128 = 50;

        client.test_mint(&user, &token_id, &10);
        client.test_burn(&user, &token_id, &10);

        assert_eq!(client.get_balance(&user, &token_id), 0);
        assert_eq!(client.owned_token_count(&user), 0);
    }
}
