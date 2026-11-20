// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * T-APP-003 (catalog completeness) + the 12 §12.10 guarantees: the
 * registry is structurally identical to the doc-derived fixture, the
 * default shortcut map has zero conflicts within any context, and the
 * menu placement matches 12 §12.11's shape.
 */
import { describe, expect, it } from "vitest";
import fixtureRaw from "./catalog.fixture.json";
import {
  activeContexts,
  chord,
  COMMAND_BY_ID,
  COMMANDS,
  type CommandSpec,
} from "./catalog";

const fixture = fixtureRaw as unknown as Array<Record<string, unknown>>;

describe("t_app_003 — command catalog completeness (12 s12.10)", () => {
  it("contains exactly the 168 catalog rows in doc order", () => {
    expect(COMMANDS).toHaveLength(168);
    expect(COMMANDS[0]?.kb).toBe("KB-001");
    expect(COMMANDS[167]?.kb).toBe("KB-168");
    for (const [index, command] of COMMANDS.entries()) {
      expect(command.kb, `row ${String(index)} kb`).toBe(
        fixture[index]?.kb as string,
      );
      expect(command.id).toBe(fixture[index]?.id as string);
      expect(command.menu).toBe(
        (fixture[index]?.menu ?? null) as string | null,
      );
      expect(command.submenu).toBe(
        (fixture[index]?.submenu ?? null) as string | null,
      );
      expect(command.paletteOnly).toBe(fixture[index]?.paletteOnly === true);
      expect(command.context).toBe(fixture[index]?.context as string);
      expect(command.checkable).toBe(fixture[index]?.checkable === true);
      expect(command.shortcuts).toEqual(fixture[index]?.shortcuts);
    }
  });

  it("ids are unique and lookable up", () => {
    expect(new Set(COMMANDS.map((c) => c.id)).size).toBe(168);
    for (const command of COMMANDS) {
      expect(COMMAND_BY_ID.get(command.id)).toBe(command);
    }
  });

  it("labels keys are kb.<id> and every menu is one of the seven or palette-only", () => {
    const menus = [
      "File",
      "Edit",
      "Clip",
      "Timeline",
      "Playback",
      "View",
      "Help",
    ];
    for (const command of COMMANDS) {
      expect(command.labelKey).toBe(`kb.${command.id}`);
      if (command.paletteOnly) {
        expect(command.menu).toBeNull();
      } else {
        expect(menus, `${command.id} menu`).toContain(command.menu);
      }
    }
  });

  it("exactly KB-168 is palette-only and exactly the five documented commands are checkable", () => {
    expect(COMMANDS.filter((c) => c.paletteOnly).map((c) => c.kb)).toEqual([
      "KB-168",
    ]);
    expect(
      COMMANDS.filter((c) => c.checkable)
        .map((c) => c.id)
        .sort(),
    ).toEqual([
      "playback.playEveryFrame",
      "playback.toggleLoop",
      "timeline.listView",
      "timeline.toggleLinkedSelection",
      "timeline.toggleSnapping",
    ]);
  });

  it("the default shortcut map has zero conflicts within any context", () => {
    // 12 s12.10 closing note; grounds T-SET-002. Global commands are
    // active in every context, so they conflict with scoped commands
    // sharing the same chord in that context.
    for (const context of ["global", "timeline", "viewer", "media"] as const) {
      const taken = new Map<string, string>();
      for (const command of COMMANDS) {
        if (!activeContexts(command.context).includes(context)) continue;
        for (const shortcut of command.shortcuts) {
          const key = chord(shortcut);
          const holder = taken.get(key);
          expect(
            holder,
            `conflict in ${context}: ${key} used by ${String(holder)} and ${command.id}`,
          ).toBeUndefined();
          taken.set(key, command.id);
        }
      }
      expect(taken.size, `${context} has shortcuts`).toBeGreaterThan(0);
    }
  });

  it("physical codes: every shortcut key is a KeyboardEvent.code value", () => {
    const codePattern =
      /^(Key[A-Z]|Digit\d|F\d{1,2}|Space|Enter|Tab|Delete|Home|End|Arrow(Left|Right|Up|Down)|Comma|Period|Semicolon|Quote|Backquote|Bracket(Left|Right)|Equal|Minus|Slash)$/;
    for (const command of COMMANDS) {
      for (const shortcut of command.shortcuts) {
        expect(
          codePattern.test(shortcut.key),
          `${command.id} key "${shortcut.key}"`,
        ).toBe(true);
      }
    }
  });

  it("WP-05 implements exactly the shell command set; the rest are later-version", () => {
    const implemented = COMMANDS.filter((c) => c.implementedIn === "WP-05");
    expect(implemented).toHaveLength(28);
    expect(
      implemented.every(
        (c) => c.id.startsWith("file.") || c.id.startsWith("view."),
      ),
    ).toBe(true);
    const later = COMMANDS.filter((c) => c.implementedIn === null);
    expect(later).toHaveLength(140);
    expect(later.map((c) => c.id)).toContain("file.export");
  });
});

describe("activeContexts (07 s7.9.1)", () => {
  it("global commands are active everywhere; scoped ones only in their context", () => {
    const global: CommandSpec | undefined = COMMAND_BY_ID.get("file.save");
    expect(global?.context).toBe("global");
    expect(activeContexts("global")).toHaveLength(4);
    expect(activeContexts("timeline")).toEqual(["timeline"]);
  });
});
