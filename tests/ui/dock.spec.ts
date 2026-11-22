// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * T-APP-002 (docking operations, workspace switching, reset,
 * persistence) and the keyboard share of T-A11Y-001, driven on the
 * shell.dock story with the harness persistence port (localStorage)
 * standing in for the settings IPC (03 s3.6 keys; WP-24 binds the
 * real backend over the same seam).
 */
import { expect, test } from "@playwright/test";

const STORY = "/?harness=1&theme=dark&locale=en&story=shell.dock";

test.beforeEach(async ({ page }) => {
  await page.goto(STORY);
  await page.evaluate(() => document.fonts.ready);
});

test("t_app_002 - WS-01 docks the 07 s7.3 arrangement", async ({ page }) => {
  await expect(page.getByRole("tab", { name: "Media" })).toBeVisible();
  await expect(page.getByRole("tab", { name: "Titles" })).toBeVisible();
  await expect(page.getByRole("tab", { name: "Source Viewer" })).toBeVisible();
  await expect(page.getByRole("tab", { name: "Program Viewer" })).toBeVisible();
  await expect(page.getByRole("tab", { name: "Inspector" })).toBeVisible();
  await expect(page.getByRole("tab", { name: "Timeline" })).toBeVisible();
});

test("t_app_002 - Alt+1..4 switch workspaces (KB-125..128)", async ({
  page,
}) => {
  const frame = page.getByRole("main", { name: "Panels" });
  await expect(frame).toHaveAttribute("data-active-workspace", "WS-01");
  await page.keyboard.press("Alt+Digit2");
  await expect(frame).toHaveAttribute("data-active-workspace", "WS-02");
  // WS-02 docks Scopes instead of the media dock (07 s7.3).
  await expect(page.getByRole("tab", { name: "Scopes" })).toBeVisible();
  await page.keyboard.press("Alt+Digit3");
  await expect(frame).toHaveAttribute("data-active-workspace", "WS-03");
  await expect(page.getByRole("tab", { name: "Mixer" })).toBeVisible();
  await page.keyboard.press("Alt+Digit4");
  await expect(frame).toHaveAttribute("data-active-workspace", "WS-04");
  await expect(page.getByRole("tab", { name: "Transcript" })).toBeVisible();
  await page.keyboard.press("Alt+Digit1");
  await expect(frame).toHaveAttribute("data-active-workspace", "WS-01");
});

test("t_app_002 - close (pointer x) and reopen from View > Panels (Shift+1..9)", async ({
  page,
}) => {
  const mediaTab = page.getByRole("tab", { name: "Media" });
  await expect(mediaTab).toBeVisible();
  await mediaTab.locator("[data-close='true']").click();
  await expect(mediaTab).toHaveCount(0);
  // Reopen via the View > Panels command (KB-135).
  await page.keyboard.press("Shift+Digit1");
  await expect(mediaTab).toBeVisible();
});

test("t_app_002 - Delete closes the focused tab (APG deletable tabs)", async ({
  page,
}) => {
  // Shift+3 focuses the Timeline (KB-137); focus lands on its tab.
  await page.keyboard.press("Shift+Digit3");
  const focused = page.locator(".dv-active-group .dv-active-tab [data-panel]");
  await expect(focused).toHaveAttribute("data-panel", "timeline");
  await page.keyboard.press("Delete");
  await expect(page.getByRole("tab", { name: "Timeline" })).toHaveCount(0);
});

test("t_app_002 - maximize toggles with ` (KB-132)", async ({ page }) => {
  const dock = page.getByRole("main", { name: "Panels" });
  await expect(dock).toHaveAttribute("data-maximized", "false");
  await page.keyboard.press("Backquote");
  await expect(dock).toHaveAttribute("data-maximized", "true");
  await page.keyboard.press("Backquote");
  await expect(dock).toHaveAttribute("data-maximized", "false");
});

test("t_app_002 - layout and active workspace survive a reload (FR-006 AC2)", async ({
  page,
}) => {
  // Edit the layout: close a panel (persists debounced, 03 s3.6).
  await page
    .getByRole("tab", { name: "History" })
    .locator("[data-close='true']")
    .click();
  await page.keyboard.press("Alt+Digit2");
  await page.waitForTimeout(800);
  await page.reload();
  await page.evaluate(() => document.fonts.ready);
  const frame = page.getByRole("main", { name: "Panels" });
  await expect(frame).toHaveAttribute("data-active-workspace", "WS-02");
  // The WS-01 override (History closed) round-trips too.
  await page.keyboard.press("Alt+Digit1");
  await expect(page.getByRole("tab", { name: "History" })).toHaveCount(0);
});

test("t_app_002 - Reset workspace restores the 07 s7.3 table (KB-130)", async ({
  page,
}) => {
  await page
    .getByRole("tab", { name: "History" })
    .locator("[data-close='true']")
    .click();
  await expect(page.getByRole("tab", { name: "History" })).toHaveCount(0);
  await page.getByRole("menuitem", { name: "View" }).click();
  await page.getByRole("menuitem", { name: /Reset Workspace/ }).click();
  await expect(page.getByRole("tab", { name: "History" })).toBeVisible();
});

test("t_a11y_001 - F6/Shift+F6 cycle and focus moves are announced", async ({
  page,
}) => {
  await page.keyboard.press("F6");
  const focused = page.locator(".dv-active-group .dv-active-tab [data-panel]");
  const first = await focused.getAttribute("data-panel");
  await page.keyboard.press("F6");
  const second = await focused.getAttribute("data-panel");
  expect(first).not.toBe(second);
  // The polite status region carries the focused-panel announcement.
  const status = page.locator('[aria-live="polite"][aria-label]');
  await expect(status).toContainText(/focused/u);
});
