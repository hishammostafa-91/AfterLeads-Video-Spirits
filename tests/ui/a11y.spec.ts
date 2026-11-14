// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * The per-story accessibility suite (T-A11Y-004, SC-006; T034): an
 * axe scan over every story card in every project mode with zero
 * violations, a visible focus ring check, and a reduced-motion
 * zero-duration assertion (07 §7.4.6). Badge text+icon pairing
 * (never color alone) is asserted structurally; color contrast
 * ratios from the token values are covered by the Vitest companion
 * `app/src/design/contrast.test.ts`.
 */
import AxeBuilder from "@axe-core/playwright";
import { expect, test } from "@playwright/test";

test("t_a11y_004 — stories are axe-clean, focus-visible rings show, motion is off", async ({
  page,
}, testInfo) => {
  const { theme, locale } = testInfo.project.metadata as {
    theme: string;
    locale: string;
  };
  await page.goto(`/?theme=${theme}&locale=${locale}`);
  await page.evaluate(() => document.fonts.ready);

  // Reduced motion is zeroed at the sheet level (07 §7.4.6): under
  // the reduce emulation the animated element computes 0s.
  const spinner = page.locator("svg[role='progressbar']").first();
  if ((await spinner.count()) > 0) {
    const duration = await spinner
      .locator("circle")
      .last()
      .evaluate((el) => window.getComputedStyle(el).animationDuration);
    expect(duration).toBe("0s");
  }

  const cards = page.locator("[data-story-id]");
  const count = await cards.count();
  expect(count).toBeGreaterThan(0);

  for (let i = 0; i < count; i += 1) {
    const card = cards.nth(i);
    const storyId = (await card.getAttribute("data-story-id")) ?? `card-${i}`;

    // Per-story axe scan: zero violations (SC-006).
    const results = await new AxeBuilder({ page })
      .include(`[data-story-id='${storyId}']`)
      .withTags(["wcag2a", "wcag2aa", "wcag21a", "wcag21aa"])
      .analyze();
    const violations = results.violations.map((v) => `${v.id}: ${v.nodes.length}`);
    expect(violations, `axe violations in ${storyId}`).toEqual([]);

    // Focus-visible: the first focusable shows a non-none outline.
    // Disabled controls are correctly not focusable, so they are
    // skipped (the disabled state has its own story). Visible-only:
    // RAC hides the native inputs of Checkbox/Switch/Radio; the ring
    // to assert is on the visible control.
    const primary = card
      .locator("button, input, textarea, [role='slider'], [role='tab'], a")
      .filter({ visible: true })
      .first();
    if ((await primary.count()) > 0 && !(await primary.isDisabled())) {
      await primary.focus();
      const outline = await primary.evaluate((el) => {
        return window.getComputedStyle(el).outlineStyle;
      });
      expect(
        outline,
        `focus outline on ${storyId} (${outline})`,
      ).not.toBe("none");
    }
  }

  // Badges never carry state by color alone: every chip pairs its
  // icon (svg) with text (07 §7.6).
  const badgeCard = page.locator("[data-story-id='badges.all']");
  if ((await badgeCard.count()) > 0) {
    const chips = badgeCard.locator("span[data-kind]");
    const chipCount = await chips.count();
    expect(chipCount).toBe(5);
    for (let i = 0; i < chipCount; i += 1) {
      expect(await chips.nth(i).locator("svg").count()).toBe(1);
      expect((await chips.nth(i).textContent()) ?? "").not.toBe("");
    }
  }
});
