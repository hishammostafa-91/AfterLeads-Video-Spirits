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
// @ts-expect-error no @types/node in the baseline (tokens.test.ts
// precedent).
import { readFileSync } from "node:fs";
// @ts-expect-error no @types/node in the baseline.
import { dirname, join } from "node:path";
// @ts-expect-error no @types/node in the baseline.
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";

const SHEET = readFileSync(
  join(dirname(fileURLToPath(import.meta.url)), "tokens.css"),
  "utf8",
) as string;

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
  // Component/large-text threshold (3:1): the accent edge. The
  // canonical border-strong values (07 §7.4) are decorative
  // boundaries — component identification rides on the focus ring
  // and accent, so no 3:1 border assertion is imposed on them.
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
          `${theme}: ${pair.label} = ${value.toFixed(2)} (min ${pair.min})`,
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
});
