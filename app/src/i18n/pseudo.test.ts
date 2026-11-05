// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * T-I18N-004 (pseudo-locales; 12 §12.14): LOC-90 is accented, ≥1.4×
 * the source length (07 §7.12 rule 5 budget), and bracketed; LOC-91
 * carries RTL bidi marks (mirrored rendering comes from `dir="rtl"`).
 * Generated from the real catalogs.
 */
import { describe, expect, it } from "vitest";
import { MESSAGES } from "./runtime";
import { pseudoBase, pseudoCatalog } from "./pseudo";

describe("t_i18n_004 pseudo-locales (12 s12.14)", () => {
  it("LOC-90 (en-XA) meets the +40% expansion budget and brackets", () => {
    const pseudo = pseudoCatalog("en-XA", MESSAGES.en);
    for (const [key, value] of Object.entries(MESSAGES.en)) {
      const transformed = pseudo[key] ?? "";
      expect(transformed.startsWith("["), key).toBe(true);
      expect(transformed.endsWith("]"), key).toBe(true);
      expect(
        transformed.length,
        `${key}: ${String(transformed.length)} vs ${String(value.length)}`,
      ).toBeGreaterThanOrEqual(Math.ceil(value.length * 1.4));
      expect(transformed).toMatch(/[àéîôû]/);
    }
  });

  it("LOC-90 keeps ICU expressions intact", () => {
    const pseudo = pseudoCatalog("en-XA", MESSAGES.en);
    expect(pseudo["harness.toolbar.zoom"]).toContain("{value}");
    expect(pseudo["harness.stories.count"]).toContain("{count, plural,");
  });

  it("LOC-91 (ar-XB) wraps every message in bidi marks", () => {
    const pseudo = pseudoCatalog("ar-XB", MESSAGES.ar);
    for (const [key, value] of Object.entries(MESSAGES.ar)) {
      const transformed = pseudo[key] ?? "";
      expect(transformed, key).toContain(value);
      expect(transformed).toMatch(/⁧[\s\S]*⁩/);
      expect(transformed.startsWith("‏")).toBe(true);
    }
  });

  it("pseudo bases map to the shipped catalogs", () => {
    expect(pseudoBase("en-XA")).toBe("en");
    expect(pseudoBase("ar-XB")).toBe("ar");
  });
});
