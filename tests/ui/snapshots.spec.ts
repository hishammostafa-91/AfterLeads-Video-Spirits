// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * The snapshot matrix (docs/07 §7.6; research D7; T033): every story
 * card captured in the four projects (dark/light × en/ar) across the
 * page-level states — default, compact (URL), and High Contrast
 * (theme param; hc.spec.ts adds the forced-colors CDP pass) — plus
 * the interaction states (hover, focus-visible, pressed) driven on
 * each card's primary focusable element. Every capture waits for
 * document.fonts.ready, and one card per project is captured twice to
 * assert repeat-run determinism.
 */
import { expect, test } from "@playwright/test";

const MODES = [
  { id: "default", url: "" },
  { id: "compact", url: "&density=compact" },
  { id: "hc", url: "&theme=high-contrast" },
] as const;

test("story matrix", async ({ page }, testInfo) => {
  // 44 cards × 3 modes per project; first-write runs are slower.
  test.setTimeout(420_000);
  const { theme, locale } = testInfo.project.metadata as {
    theme: string;
    locale: string;
  };
  for (const mode of MODES) {
    await page.goto(
      `/?theme=${theme}&locale=${locale}${mode.url === "" ? "" : mode.url}`,
    );
    await page.evaluate(() => document.fonts.ready);
    const cards = page.locator("[data-story-id]");
    const count = await cards.count();
    expect(count).toBeGreaterThan(0);

    // Repeat-run determinism: the first card captures identically
    // twice under the fixed viewport + reduced motion (research D7).
    const first = cards.first();
    const shot1 = await first.screenshot();
    const shot2 = await first.screenshot();
    expect(shot1.equals(shot2)).toBe(true);

    for (let i = 0; i < count; i += 1) {
      const card = cards.nth(i);
      const storyId = (await card.getAttribute("data-story-id")) ?? `card-${i}`;
      const tag = `${testInfo.project.name}-${mode.id}`;
      await expect(card).toHaveScreenshot(`${storyId}-${tag}.png`);
    }
  }
});

test("interaction states", async ({ page }, testInfo) => {
  // 35 stories × (hover/focus/pressed) captures per project.
  test.setTimeout(180_000);
  const { theme, locale } = testInfo.project.metadata as {
    theme: string;
    locale: string;
  };

  // Collect the story ids from the full grid once…
  await page.goto(`/?theme=${theme}&locale=${locale}`);
  await page.evaluate(() => document.fonts.ready);
  const storyIds = await page
    .locator("[data-story-id]")
    .evaluateAll((cards) =>
      cards.map((card) =>
        card instanceof HTMLElement ? (card.dataset.storyId ?? "") : "",
      ),
    );

  for (const storyId of storyIds) {
    const tag = testInfo.project.name;
    // …then capture each story on its OWN page (?story=). Pressing a
    // dropdown trigger opens a RAC popover that would bleed into the
    // next card's captures on a shared page; per-story pages make the
    // states hermetic.
    await page.goto(`/?theme=${theme}&locale=${locale}&story=${storyId}`);
    await page.evaluate(() => document.fonts.ready);
    const card = page.locator(`[data-story-id='${storyId}']`);
    await expect(card).toBeVisible();
    // Visible-only: RAC renders visually hidden native inputs inside
    // Checkbox/Switch/Radio — hover/press never become actionable on
    // them, and the ring lives on the visible control.
    const primary = card
      .locator("button, input, textarea, [role='slider'], [role='tab'], a")
      .filter({ visible: true })
      .first();

    // Static stories (menu.context's right-click area, progress,
    // badges, empty states) have no primary control — the page-mode
    // matrix covers them; there is no interaction state to capture.
    if ((await primary.count()) === 0) continue;

    // Hover. force: RAC Checkbox/Switch/Radio overlay the native
    // input with its own <label> (the press target), so the input is
    // never hover-actionable — the wrapped control's hover state is
    // what the card shows either way.
    await primary.hover({ force: true });
    await expect(card).toHaveScreenshot(`${storyId}-${tag}-hover.png`);

    // Focus-visible (keyboard modality marks the ring).
    await primary.focus();
    await expect(card).toHaveScreenshot(`${storyId}-${tag}-focus.png`);

    // Pressed (held pointer on the primary control).
    const box = await primary.boundingBox();
    if (box) {
      await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
      await page.mouse.down();
      await expect(card).toHaveScreenshot(`${storyId}-${tag}-pressed.png`);
      await page.mouse.up();
    }
  }
});
