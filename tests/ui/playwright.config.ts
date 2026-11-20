// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
// Playwright harness config (docs/07 §7.6; research D7): deterministic
// snapshots — fixed viewport 1280×720, reduced motion, four projects
// enumerating dark/light × en/ar. The mode rides the URL
// (?theme=&locale=) so one build serves every project.
import { defineConfig, devices } from "@playwright/test";

const themes = ["dark", "light"] as const;
const locales = ["en", "ar"] as const;

export default defineConfig({
  testDir: ".",
  snapshotDir: "__snapshots__",
  timeout: 30_000,
  forbidOnly: !!process.env.CI,
  workers: 2,
  use: {
    baseURL: "http://localhost:1420",
    ...devices["Desktop Chrome"],
    viewport: { width: 1280, height: 720 },
    reducedMotion: "reduce",
  },
  webServer: {
    // Repo-root cwd (the harness script runs from app/ via pnpm -C).
    command: "pnpm -C app dev",
    url: "http://localhost:1420",
    reuseExistingServer: true,
    cwd: "../..",
    timeout: 120_000,
  },
  projects: themes.flatMap((theme) =>
    locales.map((locale) => ({
      name: `${theme}-${locale}`,
      use: {
        colorScheme: theme === "dark" ? ("dark" as const) : ("light" as const),
      },
      metadata: { theme, locale },
    })),
  ),
});
