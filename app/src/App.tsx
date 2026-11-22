// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * The app entry. The default route is the product shell (the Tauri
 * window loads `/`); the UI harness mounts with `?harness=1` so one
 * build serves the Playwright projects (ADR-020).
 *
 * Workspace persistence: the harness binds its localStorage port
 * (mock backend). The packaged app keeps the in-memory seam until
 * the settings IPC ships (WP-24 binds `settings_get`/`settings_set`
 * over the same interface; 03 s3.6 keys) - never a private
 * stand-in storage in the real app.
 */
import { AppShell } from "./shell/AppShell";
import { HarnessApp } from "./harness/HarnessApp";

export function App(): React.JSX.Element {
  const harness =
    new URLSearchParams(window.location.search).get("harness") === "1";
  return harness ? <HarnessApp /> : <AppShell />;
}
