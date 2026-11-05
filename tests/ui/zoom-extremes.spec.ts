// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
// T016 / analysis M2 (spec Edge Cases): LOC-90 (+40% length) labels at
// the zoom extremes 80 and 200 wrap or truncate — the page never clips
// silently (no horizontal overflow of the document).
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
