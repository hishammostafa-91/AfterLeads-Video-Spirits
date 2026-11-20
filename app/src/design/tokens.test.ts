// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
// @vitest-environment node
// Token inventory (FR-001): both themes define the identical
// documented token set (07 §7.4) — no token may exist in one theme and
// be missing from the other. Pure file parsing, no DOM — the node
// environment gives import.meta.url a real file: URL (vitest's CSS
// handling returns empty strings for ?raw imports).
import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

const css = readFileSync(new URL("./tokens.css", import.meta.url), "utf8");

function block(selector: string): string {
  const start = css.indexOf(selector);
  if (start < 0) throw new Error(`selector not found: ${selector}`);
  const open = css.indexOf("{", start);
  let depth = 0;
  for (let i = open; i < css.length; i += 1) {
    if (css[i] === "{") depth += 1;
    if (css[i] === "}") {
      depth -= 1;
      if (depth === 0) return css.slice(open + 1, i);
    }
  }
  throw new Error(`unbalanced block: ${selector}`);
}

function tokenNames(body: string): Set<string> {
  return new Set([...body.matchAll(/(--[a-z0-9-]+)\s*:/g)].map((m) => m[1]));
}

const dark = tokenNames(block('[data-theme="dark"]'));
const light = tokenNames(block('[data-theme="light"]'));

describe("token inventory (07 s7.4)", () => {
  it("dark and light define identical token sets", () => {
    const onlyDark = [...dark].filter((t) => !light.has(t));
    const onlyLight = [...light].filter((t) => !dark.has(t));
    expect(onlyDark, `only in dark: ${onlyDark.join(", ")}`).toEqual([]);
    expect(onlyLight, `only in light: ${onlyLight.join(", ")}`).toEqual([]);
  });

  it("every documented s7.4 color token exists in both themes", () => {
    const documented = [
      "--bg-app",
      "--bg-panel",
      "--bg-panel-header",
      "--bg-raised",
      "--bg-hover",
      "--bg-active",
      "--border-subtle",
      "--border-strong",
      "--text-primary",
      "--text-secondary",
      "--text-tertiary",
      "--text-disabled",
      "--accent-solid",
      "--accent-text",
      "--accent-subtle",
      "--focus-ring",
      "--danger",
      "--danger-text",
      "--warning",
      "--warning-text",
      "--success",
      "--success-text",
      "--info",
      "--shadow-popover",
      "--shadow-dialog",
      "--tl-bg",
      "--tl-track-alt",
      "--tl-video",
      "--tl-video-border",
      "--tl-audio",
      "--tl-audio-border",
      "--tl-title",
      "--tl-title-border",
      "--tl-generator",
      "--tl-generator-border",
      "--tl-nested",
      "--tl-nested-border",
      "--tl-caption",
      "--tl-caption-border",
      "--tl-playhead",
      "--tl-inout",
      "--tl-selected",
      "--tl-offline",
    ] as const;
    for (const token of documented) {
      expect(dark.has(token), `dark missing ${token}`).toBe(true);
      expect(light.has(token), `light missing ${token}`).toBe(true);
    }
  });

  it.each(["clr-06", "clr-07", "clr-04", "clr-02", "clr-08"] as const)(
    "accent %s swaps the three accent tokens (12 s12.17)",
    (accent) => {
      const body = block(`[data-accent="${accent}"]`);
      const names = tokenNames(body);
      expect(names.has("--accent-solid")).toBe(true);
      expect(names.has("--accent-text")).toBe(true);
      expect(names.has("--accent-subtle")).toBe(true);
    },
  );

  // Round-3 L-04: the forced-colors block and the manual HC block
  // are two hand-maintained copies of one shared mapping (07 s7.4.3;
  // tokens.css header) — they must define identical name→value maps.
  it("forced-colors and manual high-contrast blocks map every token identically", () => {
    const tokens = (body: string): Map<string, string> =>
      new Map(
        [...body.matchAll(/(--[a-z0-9-]+)\s*:\s*([^;]+);/g)].map((m) => [
          m[1],
          m[2].trim(),
        ]),
      );
    const media = tokens(block("@media (forced-colors: active)"));
    // Anchor with the brace: the file header comment mentions the
    // manual selector verbatim (contrast.test.ts hit the same trap).
    const manual = tokens(block('[data-theme="high-contrast"] {'));
    expect(media.size).toBeGreaterThan(0);
    const onlyMedia = [...media].filter(([k, v]) => manual.get(k) !== v);
    const onlyManual = [...manual].filter(([k, v]) => media.get(k) !== v);
    expect(onlyMedia.map(([k]) => k)).toEqual([]);
    expect(onlyManual.map(([k]) => k)).toEqual([]);
  });
});
