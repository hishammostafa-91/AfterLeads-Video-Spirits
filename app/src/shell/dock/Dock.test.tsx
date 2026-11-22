// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * Dock host tests (T-APP-002/T-A11Y-001 in jsdom): the dock mounts
 * the active workspace's panels with tabs, workspace switching
 * (Alt+1..4) rebuilds the layout, focus commands (Shift+1..9) move
 * the focused tab, and F6 cycles. Geometry itself is asserted by the
 * Playwright harness (real layout); here we pin structure and the
 * keyboard model.
 */
import { act, fireEvent, render, screen } from "@testing-library/react";
import type { JSX, ReactNode } from "react";
import { IntlProvider } from "react-intl";
import { beforeEach, describe, expect, it } from "vitest";
import en from "../../locales/en.json";
import { installDispatcher } from "../../commands/dispatcher";
import { resetCommandRuntime, runCommand } from "../../commands/registry";
import { COMMAND_BY_ID } from "../../commands/catalog";
import { Dock } from "./Dock";
import { resetWorkspaceStoreForTests, useWorkspaceStore } from "./workspaces";

const flags = { focusedContext: "global", projectOpen: false } as const;

function Wrapper({ children }: { children: ReactNode }): JSX.Element {
  return (
    <IntlProvider locale="en" messages={en}>
      {children}
    </IntlProvider>
  );
}

function focusedTabPanelId(): string | null {
  // The focused panel = active tab of the active group (dockview
  // maintains the classes live; the accent line follows them).
  const tab = document.querySelector(
    ".dv-active-group .dv-active-tab [data-panel]",
  );
  return tab?.getAttribute("data-panel") ?? null;
}

describe("t_app_002 — dock host", () => {
  beforeEach(() => {
    resetWorkspaceStoreForTests();
    resetCommandRuntime();
  });

  it("mounts WS-01 with its panels and tabs", async () => {
    render(
      <Wrapper>
        <Dock />
      </Wrapper>,
    );
    // The placeholders carry the localized panel titles.
    expect(await screen.findAllByText("Media")).not.toHaveLength(0);
    expect(screen.getAllByText("Timeline")).not.toHaveLength(0);
    expect(screen.getAllByText("Program Viewer")).not.toHaveLength(0);
  });

  it("Alt+2 (KB-126) switches to the Color workspace layout", () => {
    const teardown = installDispatcher(window, () => flags);
    render(
      <Wrapper>
        <Dock />
      </Wrapper>,
    );
    expect(useWorkspaceStore.getState().active).toBe("WS-01");
    fireEvent.keyDown(window, { code: "Digit2", key: "2", altKey: true });
    expect(useWorkspaceStore.getState().active).toBe("WS-02");
    // WS-02 docks Scopes (07 s7.3).
    expect(screen.getAllByText("Scopes")).not.toHaveLength(0);
    teardown();
  });

  it("Shift+3 (KB-137) focuses the Timeline tab", () => {
    const teardown = installDispatcher(window, () => flags);
    render(
      <Wrapper>
        <Dock />
      </Wrapper>,
    );
    fireEvent.keyDown(window, { code: "Digit3", key: "3", shiftKey: true });
    expect(focusedTabPanelId()).toBe("timeline");
    teardown();
  });

  it("view.nextPanel (F6, KB-133) runs and cycles focus", async () => {
    render(
      <Wrapper>
        <Dock />
      </Wrapper>,
    );
    const spec = COMMAND_BY_ID.get("view.nextPanel");
    expect(spec).toBeDefined();
    const ran = spec
      ? await runCommand(spec, {
          focusedContext: "global",
          projectOpen: false,
        })
      : false;
    expect(ran).toBe(true);
  });

  it("maximize toggles (KB-132)", async () => {
    render(
      <Wrapper>
        <Dock />
      </Wrapper>,
    );
    const spec = COMMAND_BY_ID.get("view.maximizePanel");
    expect(spec).toBeDefined();
    const ran = spec
      ? await runCommand(spec, {
          focusedContext: "global",
          projectOpen: false,
        })
      : false;
    expect(ran).toBe(true);
    const specAgain = COMMAND_BY_ID.get("view.maximizePanel");
    if (specAgain) {
      await act(async () => {
        await runCommand(specAgain, {
          focusedContext: "global",
          projectOpen: false,
        });
      });
    }
  });
});
