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
    await page.goto(`/?locale=en-XA&zoom=${zoom}`);
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
// monitor-DPI step and the layout never overflows.
for (const zoom of [100, 125, 150, 175, 200] as const) {
  test(`zoom ${zoom} applies without overflow`, async ({ page }) => {
    await page.goto(`/?locale=en&zoom=${zoom}`);
    await page.evaluate(() => document.fonts.ready);
    const applied = await page.evaluate(
      (z) =>
        document.documentElement.style.getPropertyValue("--app-zoom") ===
        String(z),
      zoom,
    );
    expect(applied).toBe(true);
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
  await page.goto("/?theme=dark&locale=en");
  await page.evaluate(() => document.fonts.ready);
  const darkBg = await read("--bg-app");
  await page.goto("/?theme=light&locale=en");
  const lightBg = await read("--bg-app");
  expect(darkBg).toBeTruthy();
  expect(lightBg).toBeTruthy();
  expect(darkBg).not.toBe(lightBg);
  await page.goto("/?theme=high-contrast&locale=en");
  const hcBg = await read("--bg-app");
  expect(hcBg.trim().toLowerCase()).toBe("canvas");
});

// SC-001: accents flip the resolved accent tokens.
for (const accent of ["clr-07", "clr-04", "clr-02", "clr-08"] as const) {
  test(`accent ${accent} flips the resolved accent tokens (SC-001)`, async ({
    page,
  }) => {
    await page.goto(`/?theme=dark&locale=en&accent=${accent}`);
    await page.evaluate(() => document.fonts.ready);
    const solid = await page.evaluate(() =>
      getComputedStyle(document.documentElement)
        .getPropertyValue("--accent-solid")
        .trim(),
    );
    expect(solid).toBeTruthy();
    expect(solid).not.toBe("#3e63dd");
  });
}
