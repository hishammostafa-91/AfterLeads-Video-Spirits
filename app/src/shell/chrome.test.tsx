// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * T-APP-001 chrome share: the title bar shows the project name and
 * the dirty marker (REQ-APP-012), the toolbar enables exactly the
 * shell's implemented commands and disables later-version ones with
 * their tooltip labels (REQ-HLP-003), and the status bar renders its
 * five fields with the neutral dash until data exists (REQ-APP-007).
 */
import { act, render, screen } from "@testing-library/react";
import type { JSX, ReactNode } from "react";
import { IntlProvider } from "react-intl";
import { describe, expect, it } from "vitest";
import en from "../locales/en.json";
import { resetStatusStoreForTests, useStatusStore } from "./status";
import { StatusBar } from "./StatusBar";
import { TitleBar } from "./TitleBar";
import { Toolbar } from "./Toolbar";

const flags = { focusedContext: "global", projectOpen: true } as const;

function Wrapper({ children }: { children: ReactNode }): JSX.Element {
  return (
    <IntlProvider locale="en" messages={en}>
      {children}
    </IntlProvider>
  );
}

describe("t_app_001 — shell chrome", () => {
  it("title bar hosts the seven menus, the project name, and the dirty marker", () => {
    render(
      <Wrapper>
        <TitleBar flags={flags} projectName="Interviews" isDirty />
      </Wrapper>,
    );
    const menubar = screen.getByRole("menubar");
    expect(menubar.textContent).toContain("File");
    expect(menubar.textContent).toContain("Help");
    expect(screen.getByText("Interviews")).toBeTruthy();
    expect(screen.getByRole("img", { name: "Unsaved changes" })).toBeTruthy();
  });

  it("title bar without a project shows the neutral title and no dirty marker", () => {
    render(
      <Wrapper>
        <TitleBar flags={flags} />
      </Wrapper>,
    );
    expect(screen.getByText("No project open")).toBeTruthy();
    expect(screen.queryByRole("img", { name: "Unsaved changes" })).toBeNull();
  });

  it("toolbar: workspace switcher enabled; tools and export disabled with tooltips", () => {
    render(
      <Wrapper>
        <Toolbar flags={flags} activeWorkspace="WS-01" />
      </Wrapper>,
    );
    const toolbar = screen.getByRole("toolbar", { name: "Toolbar" });
    expect(toolbar.textContent).toContain("Workspace: Workspace › Edit");
    // Later-version controls are disabled buttons with their label as
    // the accessible name (tooltip text).
    const exportButton = screen.getByRole("button", { name: "Export…" });
    expect((exportButton as HTMLButtonElement).disabled).toBe(true);
    const blade = screen.getByRole("button", { name: "Blade Tool" });
    expect((blade as HTMLButtonElement).disabled).toBe(true);
  });

  it("status bar: neutral dashes, then live values", () => {
    resetStatusStoreForTests();
    render(
      <Wrapper>
        <StatusBar />
      </Wrapper>,
    );
    const bar = screen.getByRole("contentinfo");
    // Five fields render the neutral dash; split() adds one more part.
    expect(bar.textContent.split("—").length).toBe(6);
    act(() => {
      useStatusStore.getState().setJobs({ running: 2 });
      useStatusStore.getState().setAutosave({
        state: "saved",
        timeUtc: "2026-01-01T12:30:00Z",
      });
      useStatusStore.setState({ renderer: "NVIDIA GeForce RTX 3060" });
    });
    expect(bar.textContent).toContain("2 jobs running");
    expect(bar.textContent).toContain("Saved");
    expect(bar.textContent).toContain("NVIDIA GeForce RTX 3060");
  });
});
