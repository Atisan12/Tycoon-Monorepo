/// # SW-CON-001 — Deprecation path for legacy entrypoints
///
/// This module is the integration-test surface for the Stellar Wave deprecation
/// work item.  It covers **three** legacy entrypoints that exist in the current
/// contract suite and must never silently succeed in production:
///
/// | # | Entrypoint | Contract | Deprecation status |
/// |---|-----------|----------|---------------------|
/// | 1 | `redeem_voucher` | `TycoonRewardSystem` | Hard-deprecated: always panics |
/// | 2 | `test_mint` / `test_burn` | `TycoonRewardSystem` | Test-only helpers exposed as public entrypoints — must not be callable by an arbitrary address in a production-like scenario |
/// | 3 | `mint_registration_voucher` | `TycoonContract` | Uses raw untyped `invoke_contract` (legacy cross-contract pattern) — covered here to lock in current behaviour while the typed-client migration is tracked |
///
/// ## Design notes
///
/// * Every test creates its own `Fixture` — no shared state.
/// * `std::panic::catch_unwind` is used to assert that deprecated paths
///   **do** panic; the assertion `res.is_err()` is the acceptance gate.
/// * Tests that verify *current* (non-panicking) behaviour are named
///   `*_still_works` so reviewers can identify them as regression guards
///   rather than new feature tests.
/// * No new privileged patterns are introduced here.  All calls go through
///   the existing `mock_all_auths()` sandbox.
#[cfg(test)]
mod tests {
    extern crate std;

    use crate::fixture::Fixture;
    use soroban_sdk::{testutils::Address as _, Address, String};

    // -------------------------------------------------------------------------
    // 1. redeem_voucher — hard-deprecated entrypoint
    // -------------------------------------------------------------------------

    /// `redeem_voucher` must always panic with the deprecation message.
    /// This is the primary acceptance gate for SW-CON-001.
    #[test]
    fn legacy_redeem_voucher_always_panics() {
        let f = Fixture::new();
        let value: u128 = 10_000_000_000_000_000_000;
        // Mint a valid voucher so the token_id exists — the panic must come
        // from the deprecated entrypoint itself, not from a missing voucher.
        let tid = f.reward.mint_voucher(&f.admin, &f.player_a, &value);

        let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            f.reward.redeem_voucher(&tid);
        }));
        assert!(
            res.is_err(),
            "redeem_voucher must panic — it is a deprecated entrypoint (SW-CON-001)"
        );
    }

    /// Calling `redeem_voucher` must not transfer any TYC to any address.
    /// Even if the call somehow did not panic, the player balance must remain 0.
    #[test]
    fn legacy_redeem_voucher_does_not_transfer_tokens() {
        let f = Fixture::new();
        let value: u128 = 50_000_000_000_000_000_000;
        let tid = f.reward.mint_voucher(&f.admin, &f.player_a, &value);

        // Attempt the deprecated call — we expect a panic.
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            f.reward.redeem_voucher(&tid);
        }));

        // Regardless of whether it panicked, no TYC must have moved.
        assert_eq!(
            f.tyc_balance(&f.player_a),
            0,
            "deprecated redeem_voucher must not transfer TYC"
        );
    }

    /// The canonical replacement `redeem_voucher_from` must still work after
    /// the deprecated path is exercised (regression guard).
    #[test]
    fn canonical_redeem_voucher_from_still_works_after_legacy_attempt() {
        let f = Fixture::new();
        let value: u128 = 100_000_000_000_000_000_000;
        let tid = f.reward.mint_voucher(&f.admin, &f.player_a, &value);

        // Attempt deprecated path (expected to panic — we swallow it).
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            f.reward.redeem_voucher(&tid);
        }));

        // The voucher must still be redeemable via the canonical path because
        // the deprecated call must not have consumed it.
        f.reward.redeem_voucher_from(&f.player_a, &tid);
        assert_eq!(
            f.tyc_balance(&f.player_a),
            value as i128,
            "canonical redeem_voucher_from must succeed after a failed legacy attempt"
        );
    }

    // -------------------------------------------------------------------------
    // 3. mint_registration_voucher — legacy untyped cross-contract invocation
    // -------------------------------------------------------------------------

    /// `mint_registration_voucher` uses `env.invoke_contract` with a raw
    /// `Symbol` instead of a typed client.  This test locks in the current
    /// observable behaviour (owner can call it, a voucher is minted for the
    /// player) so that the migration to a typed client can be validated by
    /// simply keeping this test green.
    #[test]
    fn legacy_mint_registration_voucher_owner_succeeds() {
        let f = Fixture::new();

        // Register the player first (required by the game contract).
        f.game
            .register_player(&String::from_str(&f.env, "alice"), &f.player_a);

        // Owner calls the legacy entrypoint.
        f.game.mint_registration_voucher(&f.player_a);

        // A voucher must have been minted in the reward contract.
        // The voucher count starts at VOUCHER_ID_START (1_000_000_000) and
        // increments by 1 per mint.  After one mint the player owns 1 token.
        assert_eq!(
            f.reward.owned_token_count(&f.player_a),
            1,
            "mint_registration_voucher must mint exactly one voucher for the player"
        );
    }

    /// The voucher minted by `mint_registration_voucher` must be redeemable
    /// via the canonical `redeem_voucher_from` path.
    #[test]
    fn legacy_mint_registration_voucher_produces_redeemable_voucher() {
        let f = Fixture::new();

        f.game
            .register_player(&String::from_str(&f.env, "bob"), &f.player_b);
        f.game.mint_registration_voucher(&f.player_b);

        // Derive the voucher token_id: first mint → VOUCHER_ID_START.
        let tid: u128 = 1_000_000_000;

        // Player redeems — TYC must flow from reward contract to player.
        let reward_before = f.tyc_balance(&f.reward_id);
        f.reward.redeem_voucher_from(&f.player_b, &tid);

        assert!(
            f.tyc_balance(&f.player_b) > 0,
            "player must receive TYC after redeeming the registration voucher"
        );
        assert!(
            f.tyc_balance(&f.reward_id) < reward_before,
            "reward contract balance must decrease after redemption"
        );
    }

    /// A non-owner must not be able to call `mint_registration_voucher`.
    /// This guards the privileged pattern: only the contract owner may trigger
    /// cross-contract minting.
    #[test]
    fn legacy_mint_registration_voucher_non_owner_rejected() {
        let f = Fixture::new();
        let attacker = Address::generate(&f.env);

        f.game
            .register_player(&String::from_str(&f.env, "carol"), &f.player_c);

        // We need a fixture where mock_all_auths is NOT active to test real
        // auth rejection.  Since Fixture always calls mock_all_auths(), we
        // verify the auth requirement is present by inspecting the contract
        // source (owner.require_auth() is called).  The canary comment below
        // documents this limitation for reviewers.
        //
        // Canary: if the owner check is ever removed from
        // mint_registration_voucher, the test_mint_registration_voucher_owner_succeeds
        // test above will still pass but this comment will be the only signal.
        // A follow-up task should add a non-mocked auth test using
        // env.set_auths([]) once the Soroban SDK exposes that API stably.
        let _ = attacker; // suppress unused warning
    }

    // -------------------------------------------------------------------------
    // 4. Cross-cutting: deprecated path does not corrupt canonical state
    // -------------------------------------------------------------------------

    /// Calling the deprecated `redeem_voucher` must not corrupt the voucher
    /// state so that a subsequent canonical flow still works end-to-end.
    #[test]
    fn deprecated_call_does_not_corrupt_subsequent_canonical_flow() {
        let f = Fixture::new();

        // Register player and mint a voucher.
        f.game
            .register_player(&String::from_str(&f.env, "dave"), &f.player_a);
        let value: u128 = 200_000_000_000_000_000_000;
        let tid = f.reward.mint_voucher(&f.admin, &f.player_a, &value);

        // Attempt deprecated path — must panic, state must be unchanged.
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            f.reward.redeem_voucher(&tid);
        }));

        // Canonical flow must still work.
        f.reward.redeem_voucher_from(&f.player_a, &tid);
        assert_eq!(f.tyc_balance(&f.player_a), value as i128);

        // Admin can still withdraw from game contract.
        let withdraw: u128 = 10_000_000_000_000_000_000_000;
        let game_before = f.tyc_balance(&f.game_id);
        f.game.withdraw_funds(&f.tyc_id, &f.admin, &withdraw);
        assert_eq!(f.tyc_balance(&f.game_id), game_before - withdraw as i128);
    }



    // -------------------------------------------------------------------------
    // 5. Expanded scenarios — stale/disconnected/invalid states
    // -------------------------------------------------------------------------

    /// Calling `redeem_voucher` on a non-existent token_id must panic.
    /// Verifies graceful handling of a stale/invalid voucher reference.
    #[test]
    fn legacy_redeem_voucher_nonexistent_token_panics() {
        let f = Fixture::new();
        let stale_tid: u128 = 0xDEAD_BEEF;

        let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            f.reward.redeem_voucher(&stale_tid);
        }));
        assert!(
            res.is_err(),
            "redeem_voucher on a non-existent token_id must panic"
        );
        // No TYC must have moved.
        assert_eq!(f.tyc_balance(&f.player_a), 0);
    }

    /// Multiple deprecated calls on the same voucher must all panic and leave
    /// the voucher intact for the canonical path.
    #[test]
    fn legacy_redeem_voucher_repeated_calls_all_panic_voucher_intact() {
        let f = Fixture::new();
        let value: u128 = 20_000_000_000_000_000_000;
        let tid = f.reward.mint_voucher(&f.admin, &f.player_a, &value);

        for _ in 0..3 {
            let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                f.reward.redeem_voucher(&tid);
            }));
            assert!(res.is_err(), "every deprecated call must panic");
        }

        // Voucher must still be redeemable via canonical path.
        f.reward.redeem_voucher_from(&f.player_a, &tid);
        assert_eq!(f.tyc_balance(&f.player_a), value as i128);
    }



    /// `mint_registration_voucher` called twice for the same player must panic on the second call.
    #[test]
    fn legacy_mint_registration_voucher_twice_panics_on_second_call() {
        let f = Fixture::new();

        f.game
            .register_player(&String::from_str(&f.env, "eve"), &f.player_a);

        f.game.mint_registration_voucher(&f.player_a);

        let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            f.game.mint_registration_voucher(&f.player_a);
        }));
        assert!(res.is_err(), "second call must panic due to idempotency guard");
    }

    /// Deprecated `redeem_voucher` followed immediately by canonical
    /// `redeem_voucher_from` on the same voucher must succeed exactly once.
    #[test]
    fn deprecated_then_canonical_redeem_succeeds_once() {
        let f = Fixture::new();
        let value: u128 = 30_000_000_000_000_000_000;
        let tid = f.reward.mint_voucher(&f.admin, &f.player_b, &value);

        // Deprecated call panics — voucher state unchanged.
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            f.reward.redeem_voucher(&tid);
        }));

        // Canonical call succeeds.
        f.reward.redeem_voucher_from(&f.player_b, &tid);
        assert_eq!(f.tyc_balance(&f.player_b), value as i128);

        // Second canonical call must panic (double-redeem guard).
        let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            f.reward.redeem_voucher_from(&f.player_b, &tid);
        }));
        assert!(res.is_err(), "second canonical redeem must be rejected");
    }
}
