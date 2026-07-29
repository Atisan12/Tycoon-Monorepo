# NEP Standards Compliance Checklist (Issue #404)

> **Important Platform Note:** Tycoon contracts are deployed on **Stellar Soroban**, not NEAR Protocol.
> This document maps NEP-141 (NEAR Fungible Token) and NEP-171 (NEAR NFT) requirements to their
> Stellar equivalents and documents where the implementation aligns, deviates, or is not applicable.

---

## 1. NEP-141 / SEP-41 Fungible Token Crosswalk (`tycoon-token`)

NEP-141 is the NEAR fungible token standard. The Stellar equivalent is **SEP-41**.

| NEP-141 Requirement | SEP-41 / Soroban Equivalent | Status | Notes |
|---|---|---|---|
| `ft_transfer(receiver_id, amount, memo)` | `transfer(from, to, amount)` | ✅ Implemented | `memo` not in SEP-41; omitted by design |
| `ft_transfer_call(receiver_id, amount, memo, msg)` | `transfer_from(spender, from, to, amount)` | ✅ Implemented | Cross-contract callback pattern differs; Soroban uses `require_auth()` |
| `ft_total_supply() → U128` | `total_supply() → i128` | ✅ Implemented | Type: `i128` (SEP-41) vs `U128` string (NEP-141) |
| `ft_balance_of(account_id) → U128` | `balance(id) → i128` | ✅ Implemented | — |
| `ft_metadata()` → name, symbol, decimals | `name()`, `symbol()`, `decimals()` | ✅ Implemented | Returns "Tycoon", "TYC", 18 |
| `ft_resolve_transfer` (NEP-141 callback) | Not applicable | ⚠️ Deviation | Soroban uses synchronous cross-contract calls; no async callback needed |
| Storage registration (`storage_deposit`) | Not applicable | ⚠️ Deviation | See Section 3 — Stellar uses ledger entry fees, not explicit storage deposits |
| `approve` / allowance | `approve(from, spender, amount, expiration_ledger)` | ✅ Implemented | SEP-41 extension; no NEAR equivalent |
| `burn` / `burn_from` | `burn(from, amount)`, `burn_from(spender, from, amount)` | ✅ Implemented | No NEP-141 equivalent; added as extension |

**Deviations Summary:**
- No `ft_transfer_call` receiver callback — Soroban cross-contract calls are synchronous.
- No `memo` field on transfers — not part of SEP-41.
- `expiration_ledger` on approvals has no NEP-141 analogue.

---

## 2. NEP-171 / ERC-1155-Style NFT Crosswalk (`tycoon-collectibles`)

NEP-171 is the NEAR NFT standard. Tycoon collectibles are **semi-fungible** (ERC-1155 style: multiple copies per token type), deployed on Stellar Soroban. There is no direct Stellar NEP-171 equivalent; the implementation follows ERC-1155 semantics.

| NEP-171 Requirement | Tycoon Implementation | Status | Notes |
|---|---|---|---|
| `nft_transfer(receiver_id, token_id, memo)` | `_safe_transfer(from, to, token_id, amount)` | ✅ Implemented | Semi-fungible: includes `amount`; no `memo` |
| `nft_transfer_call` | `_safe_batch_transfer` (stub) | ⚠️ Partial | Batch transfer implemented as stub; single transfer is complete |
| `nft_token(token_id) → Token` | `get_metadata(token_id)`, `get_perk`, `get_strength` | ✅ Implemented | Metadata stored on-chain per token |
| `nft_total_supply() → U128` | `get_next_token_id()` (indirect) | ⚠️ Deviation | No explicit `total_supply` view; derivable from `next_token_id - 1` |
| `nft_tokens(from_index, limit)` | `get_owned_tokens_vec(owner)` | ✅ Implemented | Per-owner enumeration via `OWNED_TOKENS_PREFIX` |
| `nft_tokens_for_owner(account_id, from_index, limit)` | `get_owned_tokens_vec(owner)` | ✅ Implemented | Full list returned; pagination not yet implemented |
| `nft_supply_for_owner(account_id)` | Derivable from `get_owned_tokens_vec` length | ⚠️ Partial | No dedicated view function |
| NEP-177 Metadata (`nft_metadata()`) | `CollectibleMetadata` struct | ✅ Implemented | See Section 2a |
| NEP-178 Approvals | Not implemented | ❌ Not Applicable | Soroban auth model differs; operator approvals not required |
| NEP-181 Enumeration | `get_owned_tokens_vec`, `get_token_index` | ✅ Implemented | — |
| NEP-199 Royalties | Not implemented | ⚠️ Pending | See Section 4 — Legal review required |

**Deviations Summary:**
- Semi-fungible model (ERC-1155) vs NEP-171 unique-per-token model.
- No operator approval system — Soroban's `require_auth()` covers authorization.
- Pagination on enumeration not yet implemented (returns full list).
- No `nft_total_supply` view function.

### 2a. Metadata Standards

`CollectibleMetadata` (in `tycoon-collectibles/src/types.rs`) follows OpenSea/ERC-721 metadata conventions:

| Field | Type | NEP-177 Equivalent | Status |
|---|---|---|---|
| `name` | `String` | `title` | ✅ |
| `description` | `String` | `description` | ✅ |
| `image` | `String` (IPFS or HTTPS) | `media` | ✅ |
| `animation_url` | `Option<String>` | `extra` (custom) | ✅ |
| `external_url` | `Option<String>` | `reference` | ✅ |
| `attributes` | `Vec<MetadataAttribute>` | `extra` (custom) | ✅ |
| `media_hash` | Not implemented | `media_hash` | ⚠️ Missing |
| `reference_hash` | Not implemented | `reference_hash` | ⚠️ Missing |
| `issued_at` / `expires_at` | Not implemented | Optional NEP-177 fields | ℹ️ Not required |

**Action items:**
- [ ] Add `media_hash` (base64-encoded sha256 of image) for content integrity verification.
- [ ] Add `reference_hash` if off-chain JSON reference is used.
- [ ] Confirm `base_uri` freeze mechanism is used before mainnet launch (`BaseURIConfig.frozen`).

---

## 3. Storage Deposit Patterns

### NEAR (NEP-145)
NEAR requires explicit `storage_deposit` calls before an account can hold tokens. Contracts refund unused deposits via `storage_withdraw`.

### Stellar Soroban (Actual Implementation)
Stellar uses **ledger entry fees** and **minimum balance requirements** instead:

| Concern | NEAR Pattern | Stellar / Soroban Pattern | Status |
|---|---|---|---|
| Account registration | `storage_deposit(account_id)` | Not required — any address can hold tokens | ✅ No action needed |
| Storage cost | Paid upfront by user | Ledger entry rent paid by contract deployer / transaction submitter | ✅ Handled by Soroban runtime |
| Persistent storage | `LookupMap` with deposit | `env.storage().persistent()` — auto-managed | ✅ Implemented |
| Instance storage | N/A | `env.storage().instance()` for contract-level config | ✅ Implemented |
| Storage reclaim | `storage_withdraw` | `env.storage().persistent().remove(&key)` on zero balance | ✅ Implemented (see `set_balance`, `set_shop_stock`) |
| TTL / expiry | N/A | Ledger entry TTL — entries expire if not extended | ⚠️ Action needed |

**Action items:**
- [ ] Implement TTL extension (`env.storage().persistent().extend_ttl(...)`) for long-lived entries (balances, metadata) to prevent ledger entry expiry on mainnet.
- [ ] Document minimum balance requirements for contract deployment in `DEPLOYMENT_RUNBOOK.md`.

---

## 4. Royalties (NEP-199 / Legal Review)

### Current Status
Royalties are **not implemented** in `tycoon-collectibles`.

### NEP-199 Requirements (for reference)
```
nft_payout(token_id, balance, max_payout) → Payout
nft_transfer_payout(receiver_id, token_id, approval_id, memo, balance, max_payout) → Payout
```

### Stellar Equivalent
No standardized royalty interface exists for Stellar Soroban. Royalty logic would need to be custom-implemented in the transfer function.

### Legal Review Checklist
- [ ] Determine if royalties apply to collectible secondary sales.
- [ ] Identify applicable jurisdictions and royalty rate requirements.
- [ ] Confirm whether royalties are owed to creators, the protocol, or both.
- [ ] Review smart contract royalty enforceability (on-chain vs. marketplace-level).
- [ ] If royalties are required: implement `royalty_info(token_id, sale_price) → (recipient, amount)` in `tycoon-collectibles`.
- [ ] Ensure royalty recipient address is updatable by admin only.

---

## 5. Checklist Summary

### Completed ✅
- [x] NEP-141 / SEP-41 FT crosswalk documented
- [x] NEP-171 NFT crosswalk documented
- [x] Metadata standards reviewed (`CollectibleMetadata` vs NEP-177)
- [x] Storage deposit pattern differences documented
- [x] Royalty gap identified and legal review items listed
- [x] All deviations from NEAR standards documented with rationale
- [x] `transfer`, `transfer_from`, `total_supply`, `balance`, `approve`, `allowance`, `burn`, `burn_from` — all implemented in `tycoon-token/src/lib.rs`
- [x] `name()` → "Tycoon", `symbol()` → "TYC", `decimals()` → 18 verified in lib.rs and test snapshots
- [x] `_mint`/`_burn` storage-branch behaviour verified by `gas_snapshot_tests.rs` (#1356)
- [x] Zero-amount mint/burn treated as no-op (not error) — confirmed in `invariant_tests.rs`

### Action Items ⚠️
- [ ] Add `media_hash` and `reference_hash` to `CollectibleMetadata`
- [ ] Implement TTL extension for persistent storage entries
- [ ] Add `nft_total_supply` view function to `tycoon-collectibles`
- [ ] Implement pagination for `get_owned_tokens_vec`
- [ ] Complete `_safe_batch_transfer` (currently a stub)
- [ ] Legal review for royalties (see Section 4)
- [ ] Document minimum balance / deployment costs in `DEPLOYMENT_RUNBOOK.md`

---

*Generated for Issue #404 — Last updated: 2026-07-23 (synced with tycoon-token lib.rs, #1358)*
