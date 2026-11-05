// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
// Phase-1 harness smoke: the Playwright pipeline captures the app root
// in every project mode. T008 re-targets this spec at the harness shell.
import { expect, test } from "@playwright/test";

for (const theme of ["dark", "light"] as const) {
  for (const locale of ["en", "ar"] as const) {
    test(`harness smoke (${theme}/${locale})`, async ({ page }, testInfo) => {
      await page.goto(`/?theme=${theme}&locale=${locale}`);
      // Fonts must be settled before any capture (research D7).
      await page.evaluate(() => document.fonts.ready);
      const main = page.getByRole("main");
      await expect(main).toBeVisible();
      // One golden per project mode (dark/light × en/ar).
      await expect(page).toHaveScreenshot(
        `harness-smoke-${testInfo.project.name}.png`,
      );
    });
  }
}
