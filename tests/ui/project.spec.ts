// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * T-APP-004 e2e-lite flows over the mock backend: the Welcome ->
 * DLG-01 -> shell lifecycle and the recents open path (FR-011/012
 * ACs), driven on the project.flow story exactly like the product
 * flows run behind the typed wrappers.
 */
import { expect, test } from "@playwright/test";

const FLOW = "/?harness=1&theme=dark&locale=en&story=project.flow";

test.beforeEach(async ({ page }) => {
  await page.goto(FLOW);
  await page.evaluate(() => document.fonts.ready);
});

test("t_app_004 - welcome shows recents from project_recent", async ({
  page,
}) => {
  await expect(page.getByText("Welcome")).toBeVisible();
  await expect(page.getByRole("button", { name: "New Project" })).toBeVisible();
  await expect(
    page.getByRole("button", { name: /Interviews/ }).first(),
  ).toBeVisible();
});

test("t_app_004 - DLG-01 creates and opens the project", async ({ page }) => {
  await page.getByRole("button", { name: "New Project" }).click();
  await expect(page.getByText("New Project").first()).toBeVisible();
  // The name rule blocks Create until a safe name is typed.
  const create = page.getByRole("button", { name: "Create", exact: true });
  await expect(create).toBeDisabled();
  await page.getByLabel("Name").fill("Beach Cut");
  await expect(create).toBeEnabled();
  await create.click();
  // The shell replaces Welcome; the title bar carries the name.
  await expect(page.getByRole("menubar")).toBeVisible();
  await expect(page.getByText("Beach Cut")).toBeVisible();
  await expect(page.getByRole("main", { name: "Panels" })).toBeVisible();
});

test("t_app_004 - DLG-01 rejects an unsafe name inline", async ({ page }) => {
  await page.getByRole("button", { name: "New Project" }).click();
  const name = page.getByLabel("Name");
  await name.fill("bad:name");
  // The inline rule appears after leaving the field (07 s7.9.2).
  await name.press("Tab");
  await expect(
    page.getByText(
      "The name contains characters Windows file names forbid.",
    ),
  ).toBeVisible();
  await expect(
    page.getByRole("button", { name: "Create", exact: true }),
  ).toBeDisabled();
});

test("t_app_004 - opening a recent swaps Welcome for the shell", async ({
  page,
}) => {
  await page.getByRole("button", { name: "Interviews" }).click();
  await expect(page.getByRole("menubar")).toBeVisible();
  await expect(page.getByRole("main", { name: "Panels" })).toBeVisible();
});

test("t_app_007 - closing a clean project returns to Welcome", async ({
  page,
}) => {
  await page.getByRole("button", { name: "Interviews" }).click();
  await expect(page.getByRole("menubar")).toBeVisible();
  await page.getByRole("menuitem", { name: "File" }).click();
  await page.getByRole("menuitem", { name: "Close Project" }).click();
  // Clean project: no DLG-20; Welcome returns.
  await expect(page.getByText("Welcome")).toBeVisible();
});
