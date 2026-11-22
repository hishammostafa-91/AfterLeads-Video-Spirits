// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * Panel registry tests (T016/D3): the 15 documented panels (07 s7.7)
 * with unique ids and focus-command bindings that exist in the
 * command catalog, and placeholders that label their owning WP.
 */
import { render, screen } from "@testing-library/react";
import type { JSX, ReactNode } from "react";
import { IntlProvider } from "react-intl";
import { describe, expect, it } from "vitest";
import en from "../locales/en.json";
import { COMMAND_BY_ID } from "../commands/catalog";
import { PanelPlaceholder } from "./PanelPlaceholder";
import { PANELS, PANEL_BY_FOCUS_COMMAND } from "./registry";

function Wrapper({ children }: { children: ReactNode }): JSX.Element {
  return (
    <IntlProvider locale="en" messages={en}>
      {children}
    </IntlProvider>
  );
}

describe("t_app_002 — panel registry (07 s7.7)", () => {
  it("holds exactly the 15 documented panels with unique ids", () => {
    expect(PANELS.length).toBe(15);
    expect(new Set(PANELS.map((panel) => panel.id)).size).toBe(15);
    expect(new Set(PANELS.map((panel) => panel.pnlId)).size).toBe(15);
  });

  it("every focus command binding exists in the 12 s12.10 catalog", () => {
    for (const panel of PANELS) {
      if (panel.focusCommandId !== null) {
        expect(
          COMMAND_BY_ID.has(panel.focusCommandId),
          panel.focusCommandId,
        ).toBe(true);
      }
    }
    // The nine KB-135..143 bindings.
    expect(PANEL_BY_FOCUS_COMMAND.size).toBe(9);
  });

  it("every title key is seeded in the catalogs", () => {
    for (const panel of PANELS) {
      expect(
        en[panel.titleKey as keyof typeof en],
        panel.titleKey,
      ).toBeTruthy();
    }
  });
});

describe("t_app_002 — placeholder body (D3)", () => {
  it("labels the panel and names the owning work package", () => {
    render(
      <Wrapper>
        <PanelPlaceholder id="media" />
      </Wrapper>,
    );
    expect(screen.getByText("Media")).toBeTruthy();
    expect(
      screen.getByText("This panel arrives with WP-08 in a later version."),
    ).toBeTruthy();
  });

  it("throws on unknown panels (registry is the boundary)", () => {
    expect(() => render(<PanelPlaceholder id="nope" />)).toThrow(
      /unknown panel/,
    );
  });
});
