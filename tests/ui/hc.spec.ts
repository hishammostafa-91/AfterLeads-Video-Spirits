// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * High-Contrast captures through the Windows contrast-theme trigger
 * (docs/07 §7.4.3; research D7; T033): Playwright's `colorScheme`
 * covers only light/dark/no-preference, so `forced-colors: active`
 * is emulated through a CDP session (Emulation.setEmulatedMedia) —
 * the environment Windows contrast themes actually produce — and the
 * story grid is captured under the system-color mapping.
 */
import { expect, test, type CDPSession } from "@playwright/test";

test("forced-colors story captures", async ({ page, browser }, testInfo) => {
  test.setTimeout(120_000);
  test.skip(
    !browser.browserType().name().toLowerCase().includes("chromium"),
    "CDP forced-colors emulation is Chromium-only (research D7).",
  );
  const { locale } = testInfo.project.metadata as { locale: string };

  const cdp: CDPSession = await page.context().newCDPSession(page);
  await cdp.send("Emulation.setEmulatedMedia", {
    features: [{ name: "forced-colors", value: "active" }],
  });
  await page.goto(`/?theme=dark&locale=${locale}`);
  await page.evaluate(() => document.fonts.ready);

  // The mapping must actually engage: forced-colors flips the canvas
  // onto system colors, so the resolved background changes.
  const active = await page.evaluate(
    () => window.matchMedia("(forced-colors: active)").matches,
  );
  expect(active).toBe(true);

  const cards = page.locator("[data-story-id]");
  const count = await cards.count();
  expect(count).toBeGreaterThan(0);
  for (let i = 0; i < count; i += 1) {
    const card = cards.nth(i);
    const storyId = (await card.getAttribute("data-story-id")) ?? `card-${i}`;
    await expect(card).toHaveScreenshot(
      `${storyId}-${testInfo.project.name}-forced-colors.png`,
    );
  }
});
