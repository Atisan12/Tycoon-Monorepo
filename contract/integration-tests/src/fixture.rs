/// # Shared test fixtures for cross-contract integration tests (#411)
///
/// `Fixture::new()` deploys and wires all contracts in a single isolated
/// Soroban sandbox environment. Each test creates its own `Fixture` — the
/// `Env::default()` is completely isolated, so there is no shared state.
///
/// ## Fixture accounts
///
/// | Name       | Role                                      |
/// |------------|-------------------------------------------|
/// | `admin`    | Owns / administers all contracts          |
/// | `backend`  | Backend minter + game controller          |
/// | `player_a` | First test player                         |
/// | `player_b` | Second test player                        |
/// | `player_c` | Third test player (multi-player tests)    |
///
/// ## Deployed contracts
///
/// | Field              | Contract                           |
/// |--------------------|------------------------------------|
/// | `tyc_id`           | TYC token (Stellar asset / SEP-41) |
/// | `usdc_id`          | USDC mock token (SEP-41)           |
/// | `reward_id`        | TycoonRewardSystem                 |
/// | `game_id`          | TycoonContract (tycoon-game)       |
/// | `boost_system_id`  | TycoonBoostSystem                  |
/// | `collectibles_id`  | TycoonCollectibles                 |
#[cfg(test)]
pub use inner::{Fixture, TestFixtureConfig, GAME_FUND, REWARD_FUND};

/// Type alias for backward compatibility with tests that use `TestFixture`.
#[cfg(test)]
pub use inner::Fixture as TestFixture;

#[cfg(test)]
mod inner {
    use soroban_sdk::{
        testutils::Address as _,
        token::{Client as TokenClient, StellarAssetClient},
        Address, Env, Vec,
    };
    use tycoon_boost_system::{TycoonBoostSystem, TycoonBoostSystemClient};
    use tycoon_collectibles::{TycoonCollectibles, TycoonCollectiblesClient};
    use tycoon_game::TycoonContractClient;
    use tycoon_reward_system::{TycoonRewardSystem, TycoonRewardSystemClient};

    /// TYC pre-funded to the reward contract (1 000 000 TYC, 18 decimals).
    pub const REWARD_FUND: i128 = 1_000_000_000_000_000_000_000_000;
    /// TYC pre-funded to the game contract (500 000 TYC, 18 decimals).
    pub const GAME_FUND: i128 = 500_000_000_000_000_000_000_000;
    /// Default USDC seed minted to the game contract (10 000 USDC, 6 decimals).
    pub const DEFAULT_USDC_GAME_FUND: i128 = 10_000_000_000;

    /// Configuration for test fixture setup.
    ///
    /// Use `TestFixtureConfig::default()` for standard cross-contract tests.
    /// Override individual fields to opt-in to optional subsystems or change
    /// seed balances.
    #[derive(Clone)]
    pub struct TestFixtureConfig {
        /// Deploy and initialize the boost system. Default: `true`.
        pub deploy_boost_system: bool,
        /// Deploy and initialize the collectibles contract. Default: `true`.
        pub deploy_collectibles: bool,
        /// USDC amount minted to the game contract on setup (6 decimals).
        /// Set to 0 to skip. Default: `DEFAULT_USDC_GAME_FUND`.
        pub usdc_game_fund: i128,
    }

    impl Default for TestFixtureConfig {
        fn default() -> Self {
            Self {
                deploy_boost_system: true,
                deploy_collectibles: true,
                usdc_game_fund: DEFAULT_USDC_GAME_FUND,
            }
        }
    }

    #[allow(dead_code)]
    pub struct Fixture<'a> {
        pub env: Env,
        // Accounts
        pub admin: Address,
        pub backend: Address,
        pub player_a: Address,
        pub player_b: Address,
        pub player_c: Address,
        // Token addresses
        pub tyc_id: Address,
        pub usdc_id: Address,
        // Contract addresses
        pub reward_id: Address,
        pub game_id: Address,
        pub boost_system_id: Address,
        pub collectibles_id: Address,
        // Clients
        pub tyc: TokenClient<'a>,
        pub reward: TycoonRewardSystemClient<'a>,
        pub game: TycoonContractClient<'a>,
        pub boost_system: TycoonBoostSystemClient<'a>,
        pub collectibles: TycoonCollectiblesClient<'a>,
    }

    #[allow(dead_code)]
    impl Fixture<'_> {
        /// Create a fully-wired fixture with all subsystems enabled.
        pub fn new() -> Self {
            let env = Env::default();
            let config = TestFixtureConfig::default();
            Self::new_with_config(&env, config)
        }

        /// Create a fixture with a custom configuration.
        ///
        /// Use this when you need to disable optional subsystems (e.g. boost
        /// system or collectibles) or override the USDC seed fund.
        pub fn new_with_config(env: &Env, config: TestFixtureConfig) -> Self {
            env.mock_all_auths();

            // Accounts
            let admin = Address::generate(env);
            let backend = Address::generate(env);
            let player_a = Address::generate(env);
            let player_b = Address::generate(env);
            let player_c = Address::generate(env);

            // Stellar asset contracts (SEP-41 compatible)
            let tyc_id = env
                .register_stellar_asset_contract_v2(Address::generate(env))
                .address();
            let usdc_id = env
                .register_stellar_asset_contract_v2(Address::generate(env))
                .address();

            let tyc = TokenClient::new(env, &tyc_id);

            // Reward system
            let reward_id = env.register(TycoonRewardSystem, ());
            let reward = TycoonRewardSystemClient::new(env, &reward_id);
            reward.initialize(&admin, &tyc_id, &usdc_id);
            StellarAssetClient::new(env, &tyc_id).mint(&reward_id, &REWARD_FUND);
            reward.set_backend_minter(&backend);

            // Game contract
            let game_id = env.register(tycoon_game::TycoonContract, ());
            let game = TycoonContractClient::new(env, &game_id);
            game.initialize(&tyc_id, &usdc_id, &admin, &reward_id);
            StellarAssetClient::new(env, &tyc_id).mint(&game_id, &GAME_FUND);
            game.set_backend_game_controller(&backend);

            // Seed USDC to the game contract if requested
            if config.usdc_game_fund > 0 {
                StellarAssetClient::new(env, &usdc_id).mint(&game_id, &config.usdc_game_fund);
            }

            // Boost system (optional)
            let boost_system_id = if config.deploy_boost_system {
                env.register(TycoonBoostSystem, ())
            } else {
                Address::generate(env)
            };
            let boost_system = TycoonBoostSystemClient::new(env, &boost_system_id);
            if config.deploy_boost_system {
                boost_system.initialize(&admin);
            }

            // Collectibles contract (optional)
            let collectibles_id = if config.deploy_collectibles {
                env.register(TycoonCollectibles, ())
            } else {
                Address::generate(env)
            };
            let collectibles = TycoonCollectiblesClient::new(env, &collectibles_id);
            if config.deploy_collectibles {
                collectibles.initialize(&admin);
            }

            Fixture {
                env: env.clone(),
                admin,
                backend,
                player_a,
                player_b,
                player_c,
                tyc_id,
                usdc_id,
                reward_id,
                game_id,
                boost_system_id,
                collectibles_id,
                tyc,
                reward,
                game,
                boost_system,
                collectibles,
            }
        }

        // ── Token helpers ─────────────────────────────────────────────────────

        /// TYC balance of any address.
        pub fn tyc_balance(&self, addr: &Address) -> i128 {
            self.tyc.balance(addr)
        }

        /// USDC balance of any address.
        pub fn usdc_balance(&self, addr: &Address) -> i128 {
            TokenClient::new(&self.env, &self.usdc_id).balance(addr)
        }

        /// Return a `TokenClient` for the USDC contract.
        ///
        /// Useful when tests need repeated USDC queries without constructing the
        /// client inline each time.
        pub fn usdc_client(&self) -> TokenClient<'_> {
            TokenClient::new(&self.env, &self.usdc_id)
        }

        /// Assert that `addr` holds exactly `expected` TYC, with a descriptive
        /// message on failure.
        pub fn assert_tyc_balance(&self, addr: &Address, expected: i128, msg: &str) {
            let actual = self.tyc_balance(addr);
            assert_eq!(
                actual, expected,
                "TYC balance mismatch for {msg}: got {actual}, expected {expected}"
            );
        }

        /// Assert that `addr` holds exactly `expected` USDC, with a descriptive
        /// message on failure.
        pub fn assert_usdc_balance(&self, addr: &Address, expected: i128, msg: &str) {
            let actual = self.usdc_balance(addr);
            assert_eq!(
                actual, expected,
                "USDC balance mismatch for {msg}: got {actual}, expected {expected}"
            );
        }

        /// Mint TYC to an address (admin operation).
        pub fn mint_tyc(&self, to: &Address, amount: i128) {
            StellarAssetClient::new(&self.env, &self.tyc_id).mint(to, &amount);
        }

        /// Mint USDC to an address (admin operation).
        pub fn mint_usdc(&self, to: &Address, amount: i128) {
            StellarAssetClient::new(&self.env, &self.usdc_id).mint(to, &amount);
        }

        /// Mint USDC directly into the game contract and return the new balance.
        ///
        /// Convenience wrapper for tests that need to top-up the game's USDC
        /// treasury without constructing a `StellarAssetClient` themselves.
        pub fn mint_usdc_to_game(&self, amount: i128) -> i128 {
            StellarAssetClient::new(&self.env, &self.usdc_id).mint(&self.game_id, &amount);
            self.usdc_balance(&self.game_id)
        }

        /// Transfer TYC from one address to another.
        pub fn transfer_tyc(&self, from: &Address, to: &Address, amount: i128) {
            self.tyc.transfer(from, to, &amount);
        }

        // ── Ledger helpers ────────────────────────────────────────────────────

        /// Get the current ledger sequence number.
        pub fn current_ledger(&self) -> u32 {
            self.env.ledger().sequence()
        }

        /// Advance ledger by a number of sequences.
        ///
        /// Each sequence is assumed to be 5 seconds (Stellar mainnet cadence).
        pub fn advance_ledger(&self, sequences: u32) {
            use soroban_sdk::testutils::{Ledger, LedgerInfo};
            let current = self.env.ledger().get();
            self.env.ledger().set(LedgerInfo {
                sequence_number: current.sequence_number + sequences,
                timestamp: current.timestamp + (sequences as u64 * 5),
                ..current
            });
        }

        /// Set ledger to a specific sequence number.
        ///
        /// Timestamp is derived as `sequence * 5` seconds from epoch.
        pub fn set_ledger(&self, sequence: u32) {
            use soroban_sdk::testutils::{Ledger, LedgerInfo};
            let current = self.env.ledger().get();
            self.env.ledger().set(LedgerInfo {
                sequence_number: sequence,
                timestamp: sequence as u64 * 5,
                ..current
            });
        }

        /// Advance the ledger by approximately `seconds` of wall-clock time.
        ///
        /// Converts seconds to sequences using the 5 s/ledger Stellar cadence.
        /// Fractional sequences are truncated; a minimum of 1 sequence is
        /// always advanced so the ledger actually moves forward.
        pub fn advance_ledger_by_seconds(&self, seconds: u64) {
            let sequences = u32::max(1, (seconds / 5) as u32);
            self.advance_ledger(sequences);
        }

        // ── Player helpers ────────────────────────────────────────────────────

        /// Create a new player address (for dynamic test scenarios).
        pub fn new_player(&self) -> Address {
            Address::generate(&self.env)
        }

        /// Create a new address and register it in the game contract under
        /// `username`, returning the address.
        ///
        /// Allows tests to set up registered players in a single line:
        /// ```ignore
        /// let alice = f.new_registered_player("alice");
        /// ```
        pub fn new_registered_player(&self, username: &str) -> Address {
            use soroban_sdk::String;
            let player = Address::generate(&self.env);
            let uname = String::from_str(&self.env, username);
            self.game.register_player(&uname, &player);
            player
        }

        // ── Boost system helpers ──────────────────────────────────────────────

        /// Get all active boost IDs for a player.
        pub fn get_boost_ids(&self, player: &Address) -> Vec<u128> {
            let boosts = self.boost_system.get_active_boosts(player);
            let mut ids = Vec::new(&self.env);
            for i in 0..boosts.len() {
                if let Some(boost) = boosts.get(i) {
                    ids.push_back(boost.id);
                }
            }
            ids
        }

        /// Check if a specific boost id is active for a player.
        pub fn has_boost(&self, player: &Address, boost_id: u128) -> bool {
            let boosts = self.boost_system.get_active_boosts(player);
            for i in 0..boosts.len() {
                if let Some(boost) = boosts.get(i) {
                    if boost.id == boost_id {
                        return true;
                    }
                }
            }
            false
        }

        // ── Reward helpers ────────────────────────────────────────────────────

        /// Mint a voucher and immediately redeem it, returning the TYC amount
        /// transferred to `player`.
        ///
        /// Useful in tests that care about the end balance but not the voucher
        /// lifecycle details.
        pub fn mint_and_redeem(&self, player: &Address, value: u128) -> i128 {
            let tid = self.reward.mint_voucher(&self.admin, player, &value);
            self.reward.redeem_voucher_from(player, &tid);
            value as i128
        }
    }
}
