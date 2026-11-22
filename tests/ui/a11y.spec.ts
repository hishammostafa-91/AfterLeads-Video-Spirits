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
  await page.goto(`/?harness=1&theme=${theme}&locale=${locale}`);
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

  // H-02 (round 2): the transitioned non-progress controls must
  // compute a 0s duration under the suite's reduced-motion emulation
  // — the switch thumb and the tree chevron animate through
  // --dur-fast, which the reduced-motion media query zeroes.
  const zeroedCards = ["selection.checkbox-switch", "collections.tree"];
  for (const storyId of zeroedCards) {
    const card = page.locator(`[data-story-id='${storyId}']`);
    if ((await card.count()) === 0) continue;
    const animated = await card.first().evaluate((el) =>
      [el, ...Array.from(el.querySelectorAll("*"))]
        .filter(
          (node) => getComputedStyle(node).transitionDuration !== "0s",
        )
        .map(
          (node) =>
            `${(node as Element).tagName}.${String(
              (node as Element).className,
            )}: ${getComputedStyle(node).transitionDuration}`,
        ),
    );
    expect(
      animated,
      `${storyId} must have no non-zero transition durations under reduce`,
    ).toEqual([]);
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
      .withTags([
        "wcag2a",
        "wcag2aa",
        "wcag21a",
        "wcag21aa",
        // WCAG 2.2 is the requirement (FR-013); 2.2 rules include the
        // 24x24 target size documented in 07 §7.4.6 (round-2 M-07).
        "wcag22a",
        "wcag22aa",
      ])
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
      // Prime the keyboard interaction modality: the rings are
      // :focus-visible-gated (correct — rings belong to keyboard
      // focus), and Chromium only applies :focus-visible to script
      // focus when the last interaction was keyboard.
      await page.keyboard.press("Tab");
      await primary.focus();
      // Focus check (US3/AC1): the ring must belong to the focused
      // control — its own outline, or the label visual associated
      // with it (RAC draws the ring on the label while the input
      // stays visually hidden). Scanning the whole card would pass on
      // any persistently outlined element (round-2 M-05).
      const ring = await primary.evaluate((el) => {
        const ringToken = getComputedStyle(document.documentElement)
          .getPropertyValue("--focus-ring")
          .trim();
        // Ownership: the focused control's own outline; its label
        // visuals (RAC composes inputs inside their label —
        // RadioButton renders <label class=segment><input/></label>,
        // the checkbox ring sits on an indicator span inside the
        // label); and immediate wrappers, which components ring via
        // :has(input:focus-visible). Wrappers are checked by own
        // outline only so the scan can never widen to the card
        // (round-2 M-05).
        const ownOnly: Element[] = [el];
        let wrapper = el.parentElement;
        for (let depth = 0; wrapper && depth < 3; depth += 1) {
          ownOnly.push(wrapper);
          wrapper = wrapper.parentElement;
        }
        const labelScopes: Element[] = [];
        const wrappingLabel = el.closest("label");
        if (wrappingLabel) labelScopes.push(wrappingLabel);
        if (el.id) {
          labelScopes.push(
            ...Array.from(
              document.querySelectorAll(`label[for="${el.id}"]`),
            ),
          );
        }
        const labelledBy = el.getAttribute("aria-labelledby");
        if (labelledBy) {
          for (const id of labelledBy.split(/\s+/)) {
            labelScopes.push(
              ...Array.from(document.querySelectorAll(`#${CSS.escape(id)}`)),
            );
          }
        }
        const nodes = [
          ...ownOnly,
          ...labelScopes,
          ...labelScopes.flatMap((scope) =>
            Array.from(scope.querySelectorAll("*")),
          ),
        ];
        for (const node of nodes) {
          const style = window.getComputedStyle(node);
          if (
            style.outlineStyle !== "none" &&
            parseFloat(style.outlineWidth) >= 2
          ) {
            return {
              width: style.outlineWidth,
              color: style.outlineColor,
              ring: ringToken,
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
