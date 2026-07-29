import { describe, expect, it } from "vitest";
import {
  isShellAssetPath,
  isCacheExcludedPath,
  PWA_CACHE_NAME,
  PWA_OFFLINE_FALLBACK_URL,
  PWA_CACHE_EXCLUDED_PATTERNS,
} from ".";

describe("PWA constants", () => {
  it("uses an explicit versioned cache name", () => {
    expect(PWA_CACHE_NAME).toBe("tycoon-shell-v1");
  });

  it("matches shell-only assets and excludes dynamic state paths", () => {
    expect(isShellAssetPath("/_next/static/chunks/app.js")).toBe(true);
    expect(isShellAssetPath("/metadata/android-chrome-192x192.png")).toBe(true);
    expect(isShellAssetPath(PWA_OFFLINE_FALLBACK_URL)).toBe(true);

    expect(isShellAssetPath("/api/games/current")).toBe(false);
    expect(isShellAssetPath("/game-play")).toBe(false);
    expect(isShellAssetPath("/game-waiting")).toBe(false);
    expect(isShellAssetPath("/_next/image")).toBe(false);
  });

  describe("isCacheExcludedPath — live game state exclusion", () => {
    it("excludes /api/ routes from caching", () => {
      expect(isCacheExcludedPath("/api/games/ABC123/join")).toBe(true);
      expect(isCacheExcludedPath("/api/users/42")).toBe(true);
      expect(isCacheExcludedPath("/api/v1/games/current")).toBe(true);
    });

    it("excludes /game-* routes from caching", () => {
      expect(isCacheExcludedPath("/game-waiting?gameCode=ABC")).toBe(true);
      expect(isCacheExcludedPath("/game-room")).toBe(true);
      expect(isCacheExcludedPath("/game-play/multiplayer")).toBe(true);
    });

    it("excludes /ai-play/ routes from caching", () => {
      expect(isCacheExcludedPath("/ai-play/challenge")).toBe(true);
      expect(isCacheExcludedPath("/ai-play/ai-game")).toBe(true);
    });

    it("excludes /join-room from caching", () => {
      expect(isCacheExcludedPath("/join-room")).toBe(true);
    });

    it("allows shell and metadata assets through", () => {
      expect(isCacheExcludedPath("/_next/static/chunks/app.js")).toBe(false);
      expect(isCacheExcludedPath("/metadata/apple-touch-icon.png")).toBe(false);
      expect(isCacheExcludedPath("/manifest.json")).toBe(false);
      expect(isCacheExcludedPath("/offline")).toBe(false);
    });

    it("allows home and non-excluded routes through", () => {
      expect(isCacheExcludedPath("/")).toBe(false);
      expect(isCacheExcludedPath("/settings")).toBe(false);
      expect(isCacheExcludedPath("/profile")).toBe(false);
    });
  });

  describe("PWA_CACHE_EXCLUDED_PATTERNS constant", () => {
    it("contains correct patterns for game state paths", () => {
      expect(PWA_CACHE_EXCLUDED_PATTERNS).toContain("/api/");
      expect(PWA_CACHE_EXCLUDED_PATTERNS).toContain("/game-");
      expect(PWA_CACHE_EXCLUDED_PATTERNS).toContain("/ai-play/");
      expect(PWA_CACHE_EXCLUDED_PATTERNS).toContain("/join-room");
    });

    it("is immutable (frozen)", () => {
      expect(Object.isFrozen(PWA_CACHE_EXCLUDED_PATTERNS)).toBe(true);
    });
  });
});
