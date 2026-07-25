# Operational Runbook: Shop & Purchases

## Overview
This runbook covers the operational procedures for managing the Tycoon Shop and Purchases module, including troubleshooting failed transactions, managing coupons, and auditing financial activity.

## Common Issues & Troubleshooting

### 1. Failed Purchases
If a user reports a failed purchase but claims they were charged (or vice versa):
1.  **Check Audit Logs**:
    ```sql
    SELECT * FROM audit_trails 
    WHERE action = 'PURCHASE_CREATED' 
    AND user_id = <USER_ID> 
    ORDER BY created_at DESC;
    ```
2.  **Verify Ledger**: Check the `ledger_reconciliation` module/logs to see if the transaction was recorded in the internal ledger.
3.  **Inventory Check**: Verify if the item exists in the user's inventory:
    ```sql
    SELECT * FROM user_inventories 
    WHERE user_id = <USER_ID> 
    AND shop_item_id = <ITEM_ID>;
    ```

### 2. Invalid Coupon Errors
If coupons are not working as expected:
-   **Expiry**: Check `valid_until` in the `coupons` table.
-   **Usage Limit**: Check if `usage_count` has reached `max_usages`.
-   **Scope**: Ensure the coupon is valid for the specific `shop_item_id`.

### 3. Inventory Out of Sync
If a user cannot see their purchased items:
-   The cache might be stale. Invalidate the shop cache for the user:
    -   Redis Key: `shop:inventory:<USER_ID>`
    -   Action: `DEL shop:inventory:<USER_ID>`

### 4. Duplicate Purchase Reports (Idempotency)
`POST /shop/purchase` accepts an `Idempotency-Key` header and is wrapped with
`IdempotencyInterceptor` (`src/modules/redis/idempotency.interceptor.ts`), matching the
claim → complete → fail lifecycle used by `shop-api`'s `IdempotencyService.claimKey`:
-   **Claim**: on a new key, the request is marked `processing` in Redis (`idempotency:<key>`, 24h TTL) before the handler runs.
-   **Complete**: on success, the response is cached and replayed (with `X-Idempotency-Replayed: true`) for any repeat request using the same key.
-   **Fail**: if the handler throws, the key is deleted so the client can safely retry with the same key.
-   A second request while the first is still `processing` receives `409 Conflict`.
-   Requests without an `Idempotency-Key` header are not deduplicated — each is processed independently.

If a user reports being charged twice for what they believe was one click, check whether
the client sent the same `Idempotency-Key` on both requests; if not, this is expected
behavior and should be treated as two independent purchases (see Section 1).

## Operational Procedures

### Deactivating a Malfunctioning Shop Item
If an item is causing issues (e.g., incorrect pricing), deactivate it immediately:
```sql
UPDATE shop_items SET active = false WHERE id = <ITEM_ID>;
```
This is preferred over deletion to preserve historical purchase records.

### Refunding a Purchase
Currently, refunds are handled manually by:
1.  Removing the item from `user_inventories`.
2.  Crediting the user's balance (if applicable).
3.  Logging the action in `audit_trails` with a reason.

## Monitoring & Metrics
-   **Metric**: `tycoon_purchases_total` - Track successful vs failed purchases.
-   **Metric**: `tycoon_coupon_usage_total` - Monitor marketing campaign effectiveness.

## Support Contacts
-   Backend Team: #team-backend
-   Finance/Operations: #ops-billing
