// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * T-APP-001/003 (menu structure share): the §12.11 data is complete
 * against the catalog — every menu's groups flatten to exactly the
 * catalog commands of that menu, in KB order, with no duplicates —
 * and the group sets match the §12.11 separator table.
 */
import { describe, expect, it } from "vitest";
import { COMMANDS } from "../commands/catalog";
import { MENUS, menuItems, MENU_ORDER } from "./menus";

describe("t_app_001 — menu bar structure (12 s12.11)", () => {
  it("the seven menus appear in bar order", () => {
    expect(MENU_ORDER).toEqual([
      "File",
      "Edit",
      "Clip",
      "Timeline",
      "Playback",
      "View",
      "Help",
    ]);
  });

  it("every menu's items are exactly the catalog commands of that menu, in KB order", () => {
    for (const menu of MENUS) {
      const flat = [...menuItems(menu)];
      const inMenu = COMMANDS.filter((command) => command.menu === menu.id);
      expect(new Set(flat).size, `${menu.id}: duplicate items`).toBe(
        flat.length,
      );
      expect(
        [...new Set(flat)].sort((a, b) => flat.indexOf(a) - flat.indexOf(b)),
        `${menu.id}: catalog completeness`,
      ).toHaveLength(inMenu.length);
      for (const command of inMenu) {
        expect(flat, `${menu.id}: ${command.id} missing`).toContain(command.id);
      }
      // KB-ascending across the whole menu.
      const numbers = flat.map((id) => {
        const spec = COMMANDS.find((command) => command.id === id);
        return Number(spec?.kb.slice(3));
      });
      const sorted = [...numbers].sort((a, b) => a - b);
      expect(numbers, `${menu.id}: KB order`).toEqual(sorted);
    }
  });

  it("every non-palette command is placed in exactly one menu", () => {
    const placed = MENUS.flatMap((menu) => menuItems(menu));
    const catalogPlaced = COMMANDS.filter(
      (command) => command.menu !== null,
    ).map((command) => command.id);
    expect(new Set(placed)).toEqual(new Set(catalogPlaced));
    expect(placed.length).toBe(new Set(placed).size);
  });

  it("the documented separator groups exist (spot checks from the s12.11 table)", () => {
    const file = MENUS[0];
    expect(file.groups).toHaveLength(7);
    const edit = MENUS[1];
    expect(edit.groups).toHaveLength(6);
    const help = MENUS[6];
    expect(help.groups).toHaveLength(4);
    // File's first group: New/Open (Open Recent ▸ renders after open).
    expect(file.groups[0]?.items).toEqual([
      "file.newProject",
      "file.openProject",
    ]);
    // View's Panels group is the nine focus commands (KB-135..143).
    const view = MENUS[5];
    const panels = view.groups.find((group) =>
      group.items.includes("view.focusScopes"),
    );
    expect(panels?.items).toHaveLength(9);
    // Timeline's Tools group is KB-091..097.
    const timeline = MENUS[3];
    const tools = timeline.groups.find((group) =>
      group.items.includes("tool.select"),
    );
    expect(tools?.items[0]).toBe("tool.select");
    const lastTool = tools?.items[tools.items.length - 1];
    expect(lastTool).toBe("tool.zoom");
  });
});
