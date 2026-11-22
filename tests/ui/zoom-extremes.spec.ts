// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * T016 / analysis M2 (spec Edge Cases): LOC-90 (+40% length) labels at
 * the zoom extremes 80 and 200 wrap or truncate — the page never clips
 * silently (no horizontal overflow of the document). SC-002: the zoom
 * axis runs at every documented monitor-DPI step (100/125/150/175/200)
 * and SC-001: theme switching flips the resolved token values (a real
 * computed-style read, not just the attribute).
 */
import { expect, test } from "@playwright/test";

for (const zoom of [80, 200] as const) {
  test(`LOC-90 labels at zoom ${zoom} never clip`, async ({ page }) => {
    await page.goto(`/?harness=1&locale=en-XA&zoom=${zoom}`);
    await page.evaluate(() => document.fonts.ready);
    await expect(page.getByRole("banner")).toBeVisible();
    const overflow = await page.evaluate(
      () =>
        Math.max(
          document.documentElement.scrollWidth,
          document.body?.scrollWidth ?? 0,
        ) - window.innerWidth,
    );
    expect(overflow).toBeLessThanOrEqual(0);
  });
}

// REQ-APP-010 / SC-002: the root zoom applies at every documented
// monitor-DPI step and the layout never overflows. The applied zoom
// is asserted through the computed style on #root (round-2 M-06: the
// inline --app-zoom custom property is an implementation detail — a
// broken `zoom` declaration would leave it passing).
for (const zoom of [100, 125, 150, 175, 200] as const) {
  test(`zoom ${zoom} applies without overflow`, async ({ page }) => {
    await page.goto(`/?harness=1&locale=en&zoom=${zoom}`);
    await page.evaluate(() => document.fonts.ready);
    const applied = await page.evaluate((z) => {
      const root = document.getElementById("root");
      if (!root) return Number.NaN;
      const computed = window.getComputedStyle(root).zoom;
      return parseFloat(computed);
    }, zoom);
    expect(applied, `computed zoom on #root`).toBeCloseTo(zoom / 100, 2);
    const overflow = await page.evaluate(
      () =>
        Math.max(
          document.documentElement.scrollWidth,
          document.body?.scrollWidth ?? 0,
        ) - window.innerWidth,
    );
    expect(overflow).toBeLessThanOrEqual(0);
  });
}

// SC-001: switching the theme flips the resolved token values (the
// computed --bg-app actually changes, and High Contrast maps to the
// system Canvas color).
test("themes flip resolved token values (SC-001)", async ({ page }) => {
  const read = (name: string): Promise<string> =>
    page.evaluate(
      (tokenName) =>
        getComputedStyle(document.documentElement)
          .getPropertyValue(tokenName)
          .trim(),
      name,
    );
  await page.goto("/?harness=1&theme=dark&locale=en");
  await page.evaluate(() => document.fonts.ready);
  const darkBg = await read("--bg-app");
  await page.goto("/?harness=1&theme=light&locale=en");
  const lightBg = await read("--bg-app");
  expect(darkBg).toBeTruthy();
  expect(lightBg).toBeTruthy();
  expect(darkBg).not.toBe(lightBg);
  await page.goto("/?harness=1&theme=high-contrast&locale=en");
  const hcBg = await read("--bg-app");
  expect(hcBg.trim().toLowerCase()).toBe("canvas");
  // H-01 (round 2): the HC selection pairing resolves at runtime —
  // Highlight fills carry HighlightText labels.
  const hcOnSubtle = await read("--accent-subtle-foreground");
  expect(hcOnSubtle.trim().toLowerCase()).toBe("highlighttext");
});

// Round-3 H-01/M-02: the HC selection pairing must hold on the real
// components, not just the sheet — hovered/focused collection
// surfaces paint Highlight fills with HighlightText labels
// (07 §7.4.3); Highlight-on-High (1:1) or Highlight with inherited
// CanvasText would fail this.
test("High Contrast hovered/focused collection rows pair Highlight with HighlightText", async ({
  page,
}) => {
  // Chromium resolves the system color keywords to rgb in computed
  // style — resolve both through a probe on the HC page itself so
  // the comparison is palette-independent and still proves the
  // pairing (fill and label resolve to the two distinct system
  // colors, never High-on-High).
  await page.goto("/?harness=1&theme=high-contrast&story=collections.tree&locale=en");
  await page.evaluate(() => document.fonts.ready);
  const sys = await page.evaluate(() => {
    const probe = document.createElement("div");
    probe.style.color = "Highlight";
    probe.style.backgroundColor = "HighlightText";
    document.body.appendChild(probe);
    const style = getComputedStyle(probe);
    const resolved = {
      highlight: style.color,
      highlightText: style.backgroundColor,
    };
    probe.remove();
    return resolved;
  });
  const pairOf = (el: HTMLElement) => ({
    color: getComputedStyle(el).color,
    background: getComputedStyle(el).backgroundColor,
  });

  const treeItem = page.locator("[role=treeitem]").first();
  await treeItem.hover();
  const treePair = await treeItem.evaluate(pairOf);
  expect(treePair.background).toBe(sys.highlight);
  expect(treePair.color).toBe(sys.highlightText);

  // The `.option:hover` rule belongs to WindowedListBox's custom
  // option rows (the plain ListBox exports bare RAC items).
  await page.goto("/?harness=1&theme=high-contrast&story=collections.windowed&locale=en");
  await page.evaluate(() => document.fonts.ready);
  const option = page.locator("[role=option]").first();
  await option.hover();
  const optionPair = await option.evaluate(pairOf);
  expect(optionPair.background).toBe(sys.highlight);
  expect(optionPair.color).toBe(sys.highlightText);

  // The third rule (.table tr[data-focused]) cannot be driven yet:
  // RAC table rows are inert until selection mode ships (tracked
  // T043/T049); its token pairing is asserted at sheet level in
  // contrast.test.ts and it consumes the same two tokens as the
  // rules above.
});

// SC-001: accents flip the resolved accent tokens to the exact
// canonical fills (07 §7.4.1 / 12 §12.17 - round-2 M-06: `!== default`
// let any wrong value pass).
const ACCENT_FILLS: Readonly<Record<string, string>> = {
  "clr-07": "#8e4ec6",
  "clr-04": "#30a46c",
  "clr-02": "#f76b15",
  "clr-08": "#d6409f",
};
for (const accent of ["clr-07", "clr-04", "clr-02", "clr-08"] as const) {
  test(`accent ${accent} flips the resolved accent tokens (SC-001)`, async ({
    page,
  }) => {
    await page.goto(`/?harness=1&theme=dark&locale=en&accent=${accent}`);
    await page.evaluate(() => document.fonts.ready);
    const solid = await page.evaluate(() =>
      getComputedStyle(document.documentElement)
        .getPropertyValue("--accent-solid")
        .trim(),
    );
    expect(solid.toLowerCase()).toBe(ACCENT_FILLS[accent]);
  });
}
