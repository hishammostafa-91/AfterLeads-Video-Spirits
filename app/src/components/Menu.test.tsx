// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * Menu family tests (07 §7.6): menus render their items with
 * end-aligned shortcut hints and fire onAction (keyboard-driven),
 * MenuBar implements the APG menubar keyboard pattern (arrow cycling
 * with wrap), ContextMenu opens on right-click, Submenu nests.
 * Open state is controlled/defaulted and items are driven through
 * keyboard events for jsdom determinism.
 */
import { fireEvent, render, screen } from "@testing-library/react";
import type { JSX, ReactNode } from "react";
import { FormattedMessage, IntlProvider } from "react-intl";
import { MenuTrigger } from "react-aria-components";
import { Button } from "./Button";
import {
  ContextMenu,
  Menu,
  MenuBar,
  MenuBarItem,
  MenuItem,
  MenuItemCheckbox,
  Submenu,
} from "./Menu";
import styles from "./Menu.module.css";
import { describe, expect, it, vi } from "vitest";
import en from "../locales/en.json";

function Wrapper({ children }: { children: ReactNode }): JSX.Element {
  return (
    <IntlProvider locale="en" messages={en}>
      {children}
    </IntlProvider>
  );
}

describe("Menu (07 s7.6)", () => {
  it("renders its items with end-aligned shortcut hints", () => {
    render(
      <Wrapper>
        <MenuTest />
      </Wrapper>,
    );
    expect(screen.getByRole("menu")).toBeTruthy();
    const items = screen.getAllByRole("menuitem");
    expect(items.length).toBe(2);
    const shortcut = items[0].querySelector("kbd");
    expect(shortcut?.textContent).toBe("Ctrl+Z");
  });

  it("fires onAction when an item is chosen with the keyboard", () => {
    const onAction = vi.fn();
    render(
      <Wrapper>
        <MenuTest onAction={onAction} />
      </Wrapper>,
    );
    fireEvent.keyDown(screen.getAllByRole("menuitem")[0], { key: "Enter" });
    expect(onAction).toHaveBeenCalled();
    expect(onAction.mock.calls[0]?.[0]).toBe("undo");
  });
});

describe("MenuItemCheckbox (07 s7.6 checkable items)", () => {
  function renderCheckable(isChecked: boolean): void {
    render(
      <Wrapper>
        <MenuTrigger defaultOpen>
          <Button>
            <FormattedMessage id="components.menu.edit" />
          </Button>
          <Menu>
            <MenuItemCheckbox id="waveform" isChecked={isChecked}>
              <FormattedMessage id="components.menu.waveform" />
            </MenuItemCheckbox>
          </Menu>
        </MenuTrigger>
      </Wrapper>,
    );
  }

  it("exposes role=menuitemcheckbox with aria-checked reflecting state", () => {
    renderCheckable(true);
    const item = screen.getByRole("menuitemcheckbox");
    expect(item.getAttribute("aria-checked")).toBe("true");
  });

  it("shows aria-checked=false when unchecked", () => {
    renderCheckable(false);
    const item = screen.getByRole("menuitemcheckbox");
    expect(item.getAttribute("aria-checked")).toBe("false");
  });
});

describe("MenuBar (APG menubar pattern)", () => {
  function renderMenuBar(): HTMLButtonElement[] {
    render(
      <Wrapper>
        <MenuBar labelMessageId="components.menu.barLabel">
          <MenuBarItem label={<FormattedMessage id="components.menu.file" />}>
            <MenuItem id="new">
              <FormattedMessage id="components.menu.newProject" />
            </MenuItem>
          </MenuBarItem>
          <MenuBarItem label={<FormattedMessage id="components.menu.edit" />}>
            <MenuItem id="undo">
              <FormattedMessage id="components.menu.undo" />
            </MenuItem>
          </MenuBarItem>
        </MenuBar>
      </Wrapper>,
    );
    return Array.from(
      screen.getByRole("menubar").querySelectorAll(`.${styles.menuBarTrigger}`),
    );
  }

  it("exposes role=menubar and cycles focus with Arrow keys (wrap)", () => {
    const triggers = renderMenuBar();
    expect(triggers.length).toBe(2);
    // No trigger focused yet: navigation is a no-op until one has focus.
    fireEvent.keyDown(screen.getByRole("menubar"), { key: "ArrowRight" });
    expect(document.activeElement).not.toBe(triggers[1]);
    triggers[0].focus();
    fireEvent.keyDown(triggers[0], { key: "ArrowRight" });
    expect(document.activeElement).toBe(triggers[1]);
    // Wraps from the last item back to the first.
    fireEvent.keyDown(triggers[1], { key: "ArrowRight" });
    expect(document.activeElement).toBe(triggers[0]);
  });

  it("jumps to the first trigger with Home", () => {
    const triggers = renderMenuBar();
    triggers[1].focus();
    fireEvent.keyDown(triggers[1], { key: "Home" });
    expect(document.activeElement).toBe(triggers[0]);
  });
});

describe("ContextMenu", () => {
  it("opens on right-click", () => {
    const { container } = render(
      <Wrapper>
        <ContextMenu
          items={
            <MenuItem id="rename">
              <FormattedMessage id="components.menu.rename" />
            </MenuItem>
          }
        >
          <div className="context-target">
            <FormattedMessage id="components.menu.contextArea" />
          </div>
        </ContextMenu>
      </Wrapper>,
    );
    const target = container.querySelector(".context-target") as HTMLElement;
    expect(target).not.toBeNull();
    fireEvent.contextMenu(target);
    expect(screen.getByRole("menu")).toBeTruthy();
  });
});

describe("Submenu", () => {
  it("renders the nested trigger item inside the parent menu", () => {
    render(
      <Wrapper>
        <MenuTest withSubmenu />
      </Wrapper>,
    );
    expect(screen.getByText("New")).toBeTruthy();
    // The parent's two items, the submenu trigger, and the nested item.
    expect(screen.getAllByRole("menuitem").length).toBeGreaterThanOrEqual(3);
  });
});

function MenuTest({
  onAction,
  withSubmenu = false,
}: {
  onAction?: (key: string | number) => void;
  withSubmenu?: boolean;
}): JSX.Element {
  return (
    <MenuTrigger defaultOpen>
      <Button>
        <FormattedMessage id="components.menu.edit" />
      </Button>
      <Menu onAction={onAction}>
        <MenuItem id="undo" shortcut="Ctrl+Z">
          <FormattedMessage id="components.menu.undo" />
        </MenuItem>
        <MenuItem id="redo" shortcut="Ctrl+Y">
          <FormattedMessage id="components.menu.redo" />
        </MenuItem>
        {withSubmenu && (
          <Submenu label="New">
            <MenuItem id="project">
              <FormattedMessage id="components.menu.newProject" />
            </MenuItem>
          </Submenu>
        )}
      </Menu>
    </MenuTrigger>
  );
}
