"use client";

import { useCallback, useEffect, useState } from "react";
import { ShopGrid } from "@/components/game/ShopGrid";
import { track } from "@/lib/analytics";
import { apiClient, type ShopItemResponse } from "@/lib/api";

// ─── Adapter ──────────────────────────────────────────────────────────────────
// ShopGrid expects ShopItemData (string ids, numeric price).
// The backend returns ShopItemResponse (numeric ids, string price).

interface ShopItemData {
  id: string;
  name: string;
  description: string | null;
  price: number;
  type?: string;
  currency?: string;
  rarity?: string;
  active?: boolean;
}

function adaptItem(item: ShopItemResponse): ShopItemData {
  return {
    id: String(item.id),
    name: item.name,
    description: item.description,
    price: parseFloat(item.price) || 0,
    type: item.type,
    currency: item.currency,
    rarity: item.rarity,
    active: item.active,
  };
}

// ─── Page ─────────────────────────────────────────────────────────────────────

export default function ShopPage(): React.JSX.Element {
  const [items, setItems] = useState<ShopItemData[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const fetchCatalog = useCallback(async (): Promise<void> => {
    setIsLoading(true);
    setError(null);

    try {
      type ShopListResponse = { data: ShopItemResponse[] } | ShopItemResponse[];
      const result = await apiClient.get<ShopListResponse>("/shop/items");
      const raw: ShopItemResponse[] = Array.isArray(result)
        ? result
        : (result as { data: ShopItemResponse[] }).data ?? [];
      setItems(raw.map(adaptItem));
    } catch (err: unknown) {
      // 401 → surface a login-prompting message; other errors → generic
      if (
        err instanceof Error &&
        (err.message.includes("401") || err.message.toLowerCase().includes("unauthorized"))
      ) {
        setError("Please log in to view the shop.");
      } else {
        setError("Could not load shop items. Please try again.");
      }
    } finally {
      setIsLoading(false);
    }
  }, []);

  // Emit view_shop telemetry once on mount (preserves SW-FE-006 contract)
  useEffect(() => {
    track("view_shop", { route: "/shop" });
  }, []);

  useEffect(() => {
    void fetchCatalog();
  }, [fetchCatalog]);

  const handlePurchase = useCallback((itemId: string): void => {
    const item = items.find((i) => i.id === itemId);
    if (!item) return;

    try {
      track("purchase_click", {
        route: "/shop",
        item_id: item.id,
        item_name: item.name,
        item_category: item.type,
        currency: item.currency ?? "USD",
        value: item.price,
      });
    } catch {
      // Analytics must never block the purchase flow
    }
  }, [items]);

  return (
    <main
      aria-labelledby="shop-page-title"
      className="relative min-h-screen bg-[#010F10] px-6 py-16 text-[#F0F7F7]"
    >
      <a
        href="#shop-content"
        className="sr-only focus:not-sr-only focus:absolute focus:left-4 focus:top-4 focus:z-50 focus:rounded focus:bg-[#00F0FF] focus:px-4 focus:py-2 focus:text-[#010F10] focus:outline-none focus:ring-2 focus:ring-[#00F0FF] focus:ring-offset-2 focus:ring-offset-[#010F10]"
      >
        Skip to shop items
      </a>

      <div className="mx-auto flex max-w-5xl flex-col gap-10">
        <header className="space-y-4">
          <p className="font-orbitron text-sm uppercase tracking-[0.3em] text-[#00F0FF]">
            In-Game Shop
          </p>
          <h1
            id="shop-page-title"
            className="font-orbitron text-4xl font-[800] uppercase text-[#F0F7F7]"
          >
            Shop
          </h1>
          <p className="max-w-2xl font-dmSans text-base text-[#F0F7F7]/75">
            Browse and purchase items to use in your Tycoon games.
          </p>
        </header>

        <section
          id="shop-content"
          aria-label="Shop catalog"
          tabIndex={-1}
          className="focus:outline-none"
        >
          <ShopGrid
            items={items}
            isLoading={isLoading}
            error={error}
            onRetry={fetchCatalog}
            onPurchase={handlePurchase}
            telemetrySource="shop_page"
            columns={3}
          />
        </section>
      </div>
    </main>
  );
}
