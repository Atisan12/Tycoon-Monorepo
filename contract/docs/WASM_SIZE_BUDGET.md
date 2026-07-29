# WASM size budget (Tycoon contracts)

## Why this exists

On Soroban/Stellar, **WASM bytecode size** contributes to **deployment cost** and **resource limits**. We track **release** artifact size per contract and **fail CI** when a contract grows more than the allowed percentage vs the last committed baseline—so regressions are visible on every PR.

## Threshold (documented)

| Setting | Value |
|--------|--------|
| **Regression limit** | **3%** over `baseline_bytes` per contract (see `ci/wasm-size-budget.json`) |
| **Baseline file** | `contract/ci/wasm-size-budget.json` |
| **Release profile** | Workspace `contract/Cargo.toml`: `opt-level = "z"`, `debug = 0`, `strip = "symbols"`, `lto = true` |

## PR expectations

- The **contract** workflow appends a **markdown table** to the GitHub Actions **job summary** (and PR comment where configured): baseline vs current, delta, and pass/fail.
- If you **intentionally** increase WASM size (new features), update **`baseline_bytes`** for that `.wasm` in `wasm-size-budget.json` in the **same PR** and note why in the PR description.

## Contracts tracked

One entry per deployable `cdylib` artifact under `target/wasm32-unknown-unknown/release/`:

- `tycoon_boost_system.wasm`
- `tycoon_token.wasm`
- `tycoon_reward_system.wasm`
- `tycoon_main_game.wasm`
- `tycoon_game.wasm`
- `tycoon_collectibles.wasm`

`tycoon-lib` is a shared **library** crate only (no WASM). Integration tests do not emit WASM.

## Dependencies

| Tool | Package | Purpose |
|------|---------|--------|
| `wasm-opt` | `binaryen` (apt) | Runs `-Oz` size-optimisation pass on every release WASM artifact after `cargo build`. Shrinks bytecode before the size-budget check and before on-chain deployment, reducing Soroban storage rent. |
| `jq` | `jq` (apt, pre-installed on `ubuntu-latest`) | Parses `wasm-size-budget.json` in `check-wasm-sizes.sh`. |

In CI (`contract-ci.yml`) binaryen is installed with:
```bash
sudo apt-get install -y binaryen
```
The `Verify wasm-opt presence` step immediately after confirms the binary is on `PATH` and prints its version.

Locally, install binaryen via your package manager:
```bash
# macOS
brew install binaryen
# Ubuntu/Debian
sudo apt-get install -y binaryen
```

## Local check

```bash
cd contract
cargo build --target wasm32-unknown-unknown --release   # also runs wasm-opt -Oz
./scripts/check-wasm-sizes.sh
```

## Updating baselines after a deliberate shrink

If sizes **decrease**, you may lower `baseline_bytes` to lock in the improvement (optional but encouraged).
