// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * Contrast from the token values (REQ-A11Y-003; T034 companion to
 * the axe suite): the token sheet is parsed per theme and the WCAG
 * 2.2 contrast ratios of the canonical text/component pairs are
 * asserted — 4.5:1 for body text, 3:1 for UI components and large
 * text — in both the dark and light themes.
 */
// @vitest-environment node
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";

const SHEET = readFileSync(
  join(dirname(fileURLToPath(import.meta.url)), "tokens.css"),
  "utf8",
);

function themeBlock(theme: string): string {
  const start = SHEET.indexOf(`[data-theme="${theme}"]`);
  const end = SHEET.indexOf("}", start);
  return SHEET.slice(start, end);
}

function accentBlock(accent: string): string {
  const start = SHEET.indexOf(`[data-accent="${accent}"]`);
  const end = SHEET.indexOf("}", start);
  return SHEET.slice(start, end);
}

function lightAccentBlock(accent: string): string {
  const start = SHEET.indexOf(`[data-theme="light"][data-accent="${accent}"]`);
  const end = SHEET.indexOf("}", start);
  return SHEET.slice(start, end);
}

function rootBlock(): string {
  const start = SHEET.indexOf(":root");
  const end = SHEET.indexOf("}", start);
  return SHEET.slice(start, end);
}

function token(block: string, name: string): string {
  const match = new RegExp(`--${name}:\\s*([^;]+);`).exec(block);
  if (!match) throw new Error(`token --${name} not found`);
  return match[1].trim();
}

/** WCAG relative luminance (https://www.w3.org/TR/WCAG22/#dfn-relative-luminance). */
function luminance(hex: string): number {
  const h = hex.replace("#", "").trim();
  const full =
    h.length === 3
      ? h
          .split("")
          .map((c) => c + c)
          .join("")
      : h;
  const channels = [0, 2, 4].map(
    (i) => parseInt(full.slice(i, i + 2), 16) / 255,
  );
  const [r, g, b] = channels.map((c) =>
    c <= 0.03928 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4,
  );
  return 0.2126 * r + 0.7152 * g + 0.0722 * b;
}

function ratio(fg: string, bg: string): number {
  const l1 = luminance(fg);
  const l2 = luminance(bg);
  const [hi, lo] = l1 > l2 ? [l1, l2] : [l2, l1];
  return (hi + 0.05) / (lo + 0.05);
}

const TEXT_PAIRS: ReadonlyArray<{
  label: string;
  fg: string;
  bg: string;
  min: number;
}> = [
  {
    label: "text-primary on bg-app",
    fg: "text-primary",
    bg: "bg-app",
    min: 4.5,
  },
  {
    label: "text-primary on bg-panel",
    fg: "text-primary",
    bg: "bg-panel",
    min: 4.5,
  },
  {
    label: "text-primary on bg-raised",
    fg: "text-primary",
    bg: "bg-raised",
    min: 4.5,
  },
  {
    label: "text-secondary on bg-app",
    fg: "text-secondary",
    bg: "bg-app",
    min: 4.5,
  },
  {
    label: "text-secondary on bg-panel",
    fg: "text-secondary",
    bg: "bg-panel",
    min: 4.5,
  },
  {
    label: "text-tertiary on bg-app",
    fg: "text-tertiary",
    bg: "bg-app",
    min: 4.5,
  },
  {
    label: "danger-text on bg-panel",
    fg: "danger-text",
    bg: "bg-panel",
    min: 4.5,
  },
  {
    label: "warning-text on bg-panel",
    fg: "warning-text",
    bg: "bg-panel",
    min: 4.5,
  },
  {
    label: "success-text on bg-panel",
    fg: "success-text",
    bg: "bg-panel",
    min: 4.5,
  },
  // Input boundaries are UI components (round-2 H-03, owner ruling
  // A): --border-strong must hold ≥3:1 against both surfaces it
  // outlines, in both themes.
  {
    label: "border-strong on bg-panel",
    fg: "border-strong",
    bg: "bg-panel",
    min: 3,
  },
  {
    label: "border-strong on bg-raised",
    fg: "border-strong",
    bg: "bg-raised",
    min: 3,
  },
  { label: "accent-solid on bg-app", fg: "accent-solid", bg: "bg-app", min: 3 },
  {
    label: "accent-solid on bg-raised",
    fg: "accent-solid",
    bg: "bg-raised",
    min: 3,
  },
];

describe("token contrast ratios (REQ-A11Y-003)", () => {
  for (const theme of ["dark", "light"]) {
    it(`${theme} theme meets text and component thresholds`, () => {
      const block = themeBlock(theme);
      for (const pair of TEXT_PAIRS) {
        const value = ratio(token(block, pair.fg), token(block, pair.bg));
        expect(
          value,
          `${theme}: ${pair.label} = ${value.toFixed(2)} (min ${String(pair.min)})`,
        ).toBeGreaterThanOrEqual(pair.min);
      }
      // On-danger text (CLR-01 amended, owner ruling 2026-11-14):
      // --danger is theme-constant, --danger-foreground lives in :root.
      const onDanger = ratio(
        token(rootBlock(), "danger-foreground"),
        token(block, "danger"),
      );
      expect(
        onDanger,
        `${theme}: danger-foreground on danger = ${onDanger.toFixed(2)}`,
      ).toBeGreaterThanOrEqual(4.5);
    });
  }

  // Text on every accent fill (07 §7.4.1: --accent-foreground swaps
  // with the accent; green/orange take black — CLR-04/CLR-02).
  for (const accent of ["clr-06", "clr-07", "clr-04", "clr-02", "clr-08"]) {
    it(`${accent}: accent-foreground on accent-solid ≥ 4.5:1`, () => {
      const block = accentBlock(accent);
      const value = ratio(
        token(block, "accent-foreground"),
        token(block, "accent-solid"),
      );
      expect(value, `${accent} = ${value.toFixed(2)}`).toBeGreaterThanOrEqual(
        4.5,
      );
    });
  }

  // Links/selected text (07 §7.4.1 --accent-text) must read on every
  // theme surface and on the subtle selection background, for the
  // default accent AND every alternative in both themes (H5: the
  // light theme overrides text+subtle per accent).
  for (const theme of ["dark", "light"]) {
    const surface = themeBlock(theme);
    for (const accent of ["clr-06", "clr-07", "clr-04", "clr-02", "clr-08"]) {
      it(`${theme}/${accent}: accent-text reads on surfaces and subtle`, () => {
        const accentScope =
          theme === "light" ? lightAccentBlock(accent) : accentBlock(accent);
        const accentText = token(accentScope, "accent-text");
        const accentSubtle = token(accentScope, "accent-subtle");
        const onApp = ratio(accentText, token(surface, "bg-app"));
        expect(onApp, `on bg-app = ${onApp.toFixed(2)}`).toBeGreaterThanOrEqual(
          4.5,
        );
        const onPanel = ratio(accentText, token(surface, "bg-panel"));
        expect(
          onPanel,
          `on bg-panel = ${onPanel.toFixed(2)}`,
        ).toBeGreaterThanOrEqual(4.5);
        const onSubtle = ratio(accentText, accentSubtle);
        expect(
          onSubtle,
          `on accent-subtle = ${onSubtle.toFixed(2)}`,
        ).toBeGreaterThanOrEqual(4.5);
      });
    }
  }

  // Focus indicators are UI components: ≥ 3:1 against the app
  // background in both themes (FR-013).
  for (const theme of ["dark", "light"]) {
    it(`${theme}: focus-ring ≥ 3:1 on bg-app`, () => {
      const block = themeBlock(theme);
      const value = ratio(token(block, "focus-ring"), token(block, "bg-app"));
      expect(value, `${theme} = ${value.toFixed(2)}`).toBeGreaterThanOrEqual(3);
    });
  }

  // Round-2 H-01: the on-subtle token must pair the Highlight
  // selection fill with HighlightText labels in High Contrast — for
  // both triggers (manual theme and forced-colors) — and follow
  // --accent-text in the regular themes.
  it("root: accent-subtle-foreground follows accent-text", () => {
    expect(token(rootBlock(), "accent-subtle-foreground")).toBe(
      "var(--accent-text)",
    );
  });

  for (const label of ["manual high-contrast theme", "forced-colors block"]) {
    it(`${label}: Highlight fill pairs with HighlightText`, () => {
      // Anchor on the selector with its brace — the header comment
      // also mentions [data-theme="high-contrast"] verbatim.
      const block =
        label === "manual high-contrast theme"
          ? SHEET.slice(
              SHEET.indexOf('[data-theme="high-contrast"] {'),
              SHEET.indexOf(
                "}",
                SHEET.indexOf('[data-theme="high-contrast"] {'),
              ),
            )
          : SHEET.slice(
              SHEET.indexOf("@media (forced-colors: active)"),
              SHEET.indexOf(
                "}",
                SHEET.indexOf("@media (forced-colors: active)"),
              ),
            );
      expect(token(block, "accent-subtle")).toBe("Highlight");
      expect(token(block, "accent-subtle-foreground")).toBe("HighlightText");
      expect(token(block, "accent-foreground")).toBe("HighlightText");
    });
  }
});
