// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
// WP-05 smoke: the app root routes - `?harness=1` mounts the UI
// harness, the default route mounts the application shell (title
// bar, toolbar, dock, status bar; 07 s7.2).
import { render, screen } from "@testing-library/react";
import type { JSX, ReactNode } from "react";
import { IntlProvider } from "react-intl";
import { afterEach, describe, expect, it } from "vitest";
import en from "../locales/en.json";
import { App } from "../App";

function Wrapper({ children }: { children: ReactNode }): JSX.Element {
  return (
    <IntlProvider locale="en" messages={en}>
      {children}
    </IntlProvider>
  );
}

const originalHref = window.location.href;
afterEach(() => {
  window.history.replaceState(null, "", originalHref);
});

describe("app root routing", () => {
  it("default route mounts the shell with the Welcome surface", () => {
    const { container } = render(
      <Wrapper>
        <App />
      </Wrapper>,
    );
    // REQ-APP-005: no restored project -> Welcome (New / Open /
    // recents / Getting Started), not the editing chrome.
    expect(screen.getByText("Welcome")).toBeTruthy();
    expect(screen.getByRole("button", { name: "New Project" })).toBeTruthy();
    expect(screen.getByRole("button", { name: "Open Project" })).toBeTruthy();
    expect(screen.getByText("Recent projects")).toBeTruthy();
    expect(container.querySelector('[role="menubar"]')).toBeNull();
  });

  it("?harness=1 mounts the UI harness", () => {
    window.history.replaceState(null, "", "/?harness=1");
    const { container } = render(
      <Wrapper>
        <App />
      </Wrapper>,
    );
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
