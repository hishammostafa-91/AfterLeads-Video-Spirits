// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
// Vitest setup (research D1): Testing Library cleanup between tests.
import { afterEach } from "vitest";
import { cleanup } from "@testing-library/react";

// jsdom lacks ResizeObserver; dockview's auto-resize needs it (WP-05
// dock). A no-op stub keeps component tests mounting in jsdom - real
// geometry only exists in the Playwright harness.
class ResizeObserverStub {
  observe(): void {}
  unobserve(): void {}
  disconnect(): void {}
}
if (typeof globalThis.ResizeObserver !== "function") {
  globalThis.ResizeObserver = ResizeObserverStub;
}

afterEach(() => {
  cleanup();
});
