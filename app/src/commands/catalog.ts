// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * The command catalog (12 §12.10) as typed runtime data. The doc
 * truth lives in catalog.fixture.json — generated verbatim from the
 * docs/12 §12.10 table (never hand-edited); this module validates it
 * and layers the runtime facts the doc does not own: which work
 * package implements a command. Unimplemented commands stay disabled
 * with the "Available in a later version" tooltip until their work
 * package flips them here (WP-05 card).
 */
import fixtureRaw from "./catalog.fixture.json";

export type CommandContext = "global" | "timeline" | "viewer" | "media";

/** One physical-key chord (`KeyboardEvent.code` + modifiers; REQ-I18N-005). */
export interface Shortcut {
  key: string;
  ctrl: boolean;
  shift: boolean;
  alt: boolean;
}

export interface CommandSpec {
  /** KB id from 12 §12.10 (KB-001..KB-168). */
  kb: string;
  /** Stable command id, e.g. `file.newProject`. */
  id: string;
  /** Locale key for the label (`kb.<id>`). */
  labelKey: string;
  /** Menu bar id (File…Help) or null when palette-only. */
  menu: string | null;
  /** Submenu label within the menu (12 §12.11 ▸ groups), if any. */
  submenu: string | null;
  paletteOnly: boolean;
  shortcuts: readonly Shortcut[];
  context: CommandContext;
  checkable: boolean;
  /** The work package that implements this command; null = later version. */
  implementedIn: string | null;
}

const CONTEXTS: readonly CommandContext[] = [
  "global",
  "timeline",
  "viewer",
  "media",
];

/**
 * Commands implemented by WP-05 (the shell). Every other id stays
 * null (disabled, "Available in a later version") until its own work
 * package lands and flips it here.
 */
const IMPLEMENTED_BY_WP_05: ReadonlySet<string> = new Set([
  "file.newProject",
  "file.openProject",
  "file.save",
  "file.saveAs",
  "file.closeProject",
  "file.exit",
  "view.workspaceEdit",
  "view.workspaceColor",
  "view.workspaceAudio",
  "view.workspaceCaptions",
  "view.resetWorkspace",
  "view.commandPalette",
  "view.maximizePanel",
  "view.nextPanel",
  "view.prevPanel",
  "view.focusMedia",
  "view.focusSource",
  "view.focusTimeline",
  "view.focusProgram",
  "view.focusInspector",
  "view.focusEffects",
  "view.focusCaptions",
  "view.focusMixer",
  "view.focusScopes",
  "view.uiZoomIn",
  "view.uiZoomOut",
  "view.uiZoomReset",
  "view.theme",
]);

const asShortcut = (raw: unknown, where: string): Shortcut => {
  if (typeof raw !== "object" || raw === null) {
    throw new Error(`${where}: shortcut is not an object`);
  }
  const rec = raw as Record<string, unknown>;
  for (const mod of ["ctrl", "shift", "alt"] as const) {
    if (typeof rec[mod] !== "boolean") {
      throw new Error(`${where}: shortcut.${mod} must be boolean`);
    }
  }
  if (typeof rec.key !== "string" || rec.key.length === 0) {
    throw new Error(`${where}: shortcut.key must be a non-empty string`);
  }
  return {
    key: rec.key,
    ctrl: rec.ctrl as boolean,
    shift: rec.shift as boolean,
    alt: rec.alt as boolean,
  };
};

const asRow = (raw: unknown, index: number): CommandSpec => {
  if (typeof raw !== "object" || raw === null) {
    throw new Error(`catalog row ${String(index)}: not an object`);
  }
  const rec = raw as Record<string, unknown>;
  const where = `catalog row ${String(index)}`;
  const str = (field: string): string => {
    const value = rec[field];
    if (typeof value !== "string") {
      throw new Error(`${where}: ${field} must be a string`);
    }
    return value;
  };
  const context = str("context");
  if (!CONTEXTS.includes(context as CommandContext)) {
    throw new Error(`${where}: bad context "${context}"`);
  }
  const shortcutsRaw = rec.shortcuts;
  if (!Array.isArray(shortcutsRaw)) {
    throw new Error(`${where}: shortcuts must be an array`);
  }
  const id = str("id");
  return {
    kb: str("kb"),
    id,
    labelKey: `kb.${id}`,
    menu: rec.menu === null ? null : str("menu"),
    submenu: rec.submenu === null ? null : str("submenu"),
    paletteOnly: rec.paletteOnly === true,
    shortcuts: shortcutsRaw.map((s, i) =>
      asShortcut(s, `${where} shortcut ${String(i)}`),
    ),
    context: context as CommandContext,
    checkable: rec.checkable === true,
    implementedIn: IMPLEMENTED_BY_WP_05.has(id) ? "WP-05" : null,
  };
};

const parseCatalog = (raw: unknown): readonly CommandSpec[] => {
  if (!Array.isArray(raw)) throw new Error("catalog fixture: not an array");
  return raw.map(asRow);
};

/** The full 12 §12.10 catalog, doc order (KB-001..KB-168). */
export const COMMANDS: readonly CommandSpec[] = parseCatalog(fixtureRaw);

/** Command lookup by stable id. */
export const COMMAND_BY_ID: ReadonlyMap<string, CommandSpec> = new Map(
  COMMANDS.map((command) => [command.id, command]),
);

/**
 * The contexts a command is active in: global commands are active in
 * every context; scoped commands only in their own (07 §7.9.1).
 */
export function activeContexts(
  context: CommandContext,
): readonly CommandContext[] {
  return context === "global" ? CONTEXTS : [context];
}

/** The canonical chord string of a shortcut (US labels; tests and fixtures). */
export function chord(spec: Shortcut): string {
  const parts: string[] = [];
  if (spec.ctrl) parts.push("Ctrl");
  if (spec.shift) parts.push("Shift");
  if (spec.alt) parts.push("Alt");
  parts.push(spec.key);
  return parts.join("+");
}
