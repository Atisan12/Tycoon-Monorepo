import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";

const __dirname = dirname(fileURLToPath(import.meta.url));
const LOCALES = join(__dirname, "../public/locales");
const REQUIRED = ["title","room_code_label","room_code_placeholder","submit",
  "invalid_code","network_error","timeout","service_unavailable","success"];

function load(lang: string) {
  return JSON.parse(readFileSync(join(LOCALES, lang, "common.json"), "utf-8")) as
    Record<string, Record<string, string>>;
}

(["en", "es"] as const).forEach((lang) => {
  describe(`${lang}/common.json — join_room i18n`, () => {
    const data = load(lang);
    it("has join_room namespace", () => { expect(data.join_room).toBeDefined(); });
    REQUIRED.forEach((key) => {
      it(`has non-empty key: join_room.${key}`, () => {
        expect(typeof data.join_room[key]).toBe("string");
        expect(data.join_room[key].trim().length).toBeGreaterThan(0);
      });
    });
    it("network_error mentions network or connection", () => {
      const m = data.join_room.network_error.toLowerCase();
      expect(m.includes("network") || m.includes("connection") || m.includes("red") || m.includes("conexi")).toBe(true);
    });
    it("timeout mentions timeout or expiry", () => {
      const m = data.join_room.timeout.toLowerCase();
      expect(m.includes("timeout") || m.includes("timed") || m.includes("expir")).toBe(true);
    });
  });
});
