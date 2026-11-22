// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
// Phase-1 harness smoke: the Playwright pipeline captures the app root
// in every project mode (dark/light × en/ar). T008 re-targets this
// spec at the harness shell.
import { expect, test } from "@playwright/test";

test("harness smoke (shell)", async ({ page }, testInfo) => {
  const { theme, locale } = testInfo.project.metadata as {
    theme: string;
    locale: string;
  };
  await page.goto(`/?harness=1&theme=${theme}&locale=${locale}`);
  // Fonts must be settled before any capture (research D7).
  await page.evaluate(() => document.fonts.ready);
  await expect(page.getByRole("banner")).toBeVisible();
  // One golden per project mode (dark/light × en/ar).
  await expect(page).toHaveScreenshot(
    `harness-smoke-${testInfo.project.name}.png`,
  );
});
