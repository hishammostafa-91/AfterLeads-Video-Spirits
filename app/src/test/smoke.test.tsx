// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
// Phase-2 smoke: the app root mounts the harness shell with its
// landmarks (07 §7.13 rule 2) and live regions (rule 3).
import { render } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { App } from "../App";

describe("app root", () => {
  it("mounts the harness shell with banner, toolbar, and stories region", () => {
    const { container } = render(<App />);
    const main = container.querySelector("main.container");
    expect(main).not.toBeNull();
    expect(container.querySelector('[role="banner"]')).not.toBeNull();
    expect(container.querySelector('[role="toolbar"]')).not.toBeNull();
    expect(
      container.querySelector('[aria-live="polite"][aria-label]'),
    ).not.toBeNull();
    expect(
      container.querySelector('[aria-live="assertive"][role="alert"]'),
    ).not.toBeNull();
  });
});
