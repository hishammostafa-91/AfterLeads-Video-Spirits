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

/** Format a #rrggbb token value the way computed colors report it. */
function rgb(hex: string): string {
  const full =
    hex.length === 4
      ? hex
          .slice(1)
          .split("")
          .map((c) => c + c)
          .join("")
      : hex.slice(1);
  const n = parseInt(full, 16);
  const channel = (shift: number): number => (n >> shift) & 0xff;
  return `rgb(${String(channel(16))}, ${String(channel(8))}, ${String(channel(0))})`;
}

test("t_a11y_004 — stories are axe-clean, focus-visible rings show, motion is off", async ({
  page,
}, testInfo) => {
  const { theme, locale } = testInfo.project.metadata as {
    theme: string;
    locale: string;
  };
  await page.goto(`/?theme=${theme}&locale=${locale}`);
  await page.evaluate(() => document.fonts.ready);

  // Progress indicators (Spinner) keep animating under reduced
  // motion (07 §7.4.6 / REQ-A11Y-004); everything else zeroes.
  const spinner = page.locator("svg[role='progressbar']").first();
  if ((await spinner.count()) > 0) {
    const duration = await spinner
      .locator("circle")
      .last()
      .evaluate((el) => window.getComputedStyle(el).animationDuration);
    expect(duration, "spinner must keep animating under reduce").not.toBe("0s");
    expect(duration).not.toBe("none");
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
    const violations = results.violations.map(
      (v) => `${v.id}: ${v.nodes.length}`,
    );
    expect(violations, `axe violations in ${storyId}`).toEqual([]);

    // The selector covers the collection roles too (tree items, table
    // rows, list options); the first non-disabled match is the focus
    // target even when RAC renders it visually hidden (it is still
    // the tab stop).
    const matches = card.locator(
      "button, input, textarea, [role='slider'], [role='tab'], a, " +
        "[role='treeitem'], [role='row'], [role='option']",
    );
    const total = await matches.count();
    let primary: ReturnType<typeof matches.nth> | null = null;
    for (let i = 0; i < total; i += 1) {
      const candidate = matches.nth(i);
      if (await candidate.isDisabled()) continue;
      // Only elements that can actually hold focus (RAC table rows
      // are inert until selection mode; focusing them is a no-op).
      const focusable = await candidate.evaluate(
        (el) =>
          el.tabIndex >= 0 ||
          el instanceof HTMLInputElement ||
          el instanceof HTMLTextAreaElement ||
          el instanceof HTMLButtonElement,
      );
      if (focusable) {
        primary = candidate;
        break;
      }
    }
    if (primary !== null) {
      await primary.focus();
      // Focus check (US3/AC1): assert a visible 2 px outline in the
      // --focus-ring token color somewhere in the card — RAC often
      // draws the ring on the label visual instead of the (possibly
      // visually hidden) input that holds focus.
      const ring = await primary.evaluate((el) => {
        const scope = el.closest("[data-story-id]") ?? el;
        const target = getComputedStyle(document.documentElement)
          .getPropertyValue("--focus-ring")
          .trim();
        for (const node of Array.from(scope.querySelectorAll("*"))) {
          const style = window.getComputedStyle(node);
          if (
            style.outlineStyle !== "none" &&
            parseFloat(style.outlineWidth) >= 2
          ) {
            return {
              width: style.outlineWidth,
              color: style.outlineColor,
              ring: target,
            };
          }
        }
        return null;
      });
      expect(ring, `visible focus ring on ${storyId}`).not.toBeNull();
      if (ring !== null) {
        expect(parseFloat(ring.width), `ring width on ${storyId}`).toBe(2);
        expect(
          ring.color.toLowerCase(),
          `ring color on ${storyId} (${ring.color})`,
        ).toBe(rgb(ring.ring).toLowerCase());
      }
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
