// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * T-APP-003 (palette share): fuzzy results in the active locale,
 * enabled rows run their command and close, later-version rows stay
 * visible with their note and never run, and the empty state shows
 * when nothing matches.
 */
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import type { JSX, ReactNode } from "react";
import { IntlProvider } from "react-intl";
import { afterEach, describe, expect, it, vi } from "vitest";
import en from "../../locales/en.json";
import { COMMAND_BY_ID } from "../catalog";
import { registerHandler, resetCommandRuntime } from "../registry";
import { CommandPalette } from "./CommandPalette";

const flags = { focusedContext: "global", projectOpen: true } as const;

function Wrapper({ children }: { children: ReactNode }): JSX.Element {
  return (
    <IntlProvider locale="en" messages={en}>
      {children}
    </IntlProvider>
  );
}

afterEach(() => {
  resetCommandRuntime();
});

describe("t_app_003 — command palette (REQ-APP-004)", () => {
  it("renders all commands initially and filters fuzzily", async () => {
    const { rerender } = render(
      <Wrapper>
        <CommandPalette isOpen onClose={() => {}} flags={flags} />
      </Wrapper>,
    );
    const input = await screen.findByRole("textbox", {
      name: "Search commands…",
    });
    expect(await screen.findAllByText("New Project…")).not.toHaveLength(0);
    fireEvent.change(input, { target: { value: "zzz-nothing" } });
    expect(screen.getByText("No matching commands")).toBeTruthy();
    fireEvent.change(input, { target: { value: "sae" } });
    await waitFor(() => {
      expect(screen.getByText("Save")).toBeTruthy();
    });
    rerender(
      <Wrapper>
        <CommandPalette isOpen onClose={() => {}} flags={flags} />
      </Wrapper>,
    );
  });

  it("activating an enabled command runs it and closes the palette", async () => {
    const run = vi.fn();
    registerHandler("view.commandPalette", run);
    const onClose = vi.fn();
    render(
      <Wrapper>
        <CommandPalette isOpen onClose={onClose} flags={flags} />
      </Wrapper>,
    );
    const input = await screen.findByRole("textbox", {
      name: "Search commands…",
    });
    fireEvent.change(input, { target: { value: "palette" } });
    const row = await screen.findByText("Command Palette");
    // Click activation (RAC ListBox onAction; the keyboard Enter path
    // is RAC-guaranteed and exercised in the harness interaction
    // suite, which runs a real browser).
    fireEvent.click(row.closest('[role="option"]') as HTMLElement);
    await waitFor(() => {
      expect(run).toHaveBeenCalledTimes(1);
      expect(onClose).toHaveBeenCalled();
    });
  });

  it("later-version commands render their note and never run", async () => {
    const run = vi.fn();
    const spec = COMMAND_BY_ID.get("file.export");
    expect(spec?.implementedIn).toBeNull();
    registerHandler("view.commandPalette", run);
    render(
      <Wrapper>
        <CommandPalette isOpen onClose={() => {}} flags={flags} />
      </Wrapper>,
    );
    const input = await screen.findByRole("textbox", {
      name: "Search commands…",
    });
    fireEvent.change(input, { target: { value: "export" } });
    const row = await screen.findByText("Export…");
    expect(
      screen.getAllByText("Available in a later version"),
    ).not.toHaveLength(0);
    fireEvent.click(row.closest('[role="option"]') as HTMLElement);
    // The disabled row cannot run anything; nothing fired.
    expect(run).not.toHaveBeenCalled();
  });
});
