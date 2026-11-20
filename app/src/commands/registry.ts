// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * The command runtime (REQ-APP-004; 07 §7.9.1): enable state,
 * handlers, and checked state over the static catalog. Unimplemented
 * commands are never executable — they render disabled with the
 * localized "Available in a later version" tooltip until their work
 * package flips implementedIn in catalog.ts (WP-05 card).
 */
import {
  COMMAND_BY_ID,
  type CommandContext,
  type CommandSpec,
} from "./catalog";

/** Dynamic facts the enable state depends on. */
export interface CommandRuntimeFlags {
  /** The context of the focused panel; null before the shell mounts. */
  focusedContext: CommandContext | null;
  /** True while a project is open. */
  projectOpen: boolean;
}

export const INITIAL_RUNTIME_FLAGS: CommandRuntimeFlags = {
  focusedContext: null,
  projectOpen: false,
};

/** Commands that need an open project to make sense (File › Save…). */
const REQUIRES_PROJECT: ReadonlySet<string> = new Set([
  "file.save",
  "file.saveAs",
  "file.closeProject",
]);

export type CommandHandler = () => void | Promise<void>;

const handlers = new Map<string, CommandHandler>();
const checked = new Map<string, boolean>();

/** Register the handler for an implemented command (its owning WP). */
export function registerHandler(id: string, handler: CommandHandler): void {
  const spec = COMMAND_BY_ID.get(id);
  if (!spec) throw new Error(`registerHandler: unknown command "${id}"`);
  if (spec.implementedIn === null) {
    throw new Error(
      `registerHandler: "${id}" is not implemented by any shipped work package`,
    );
  }
  handlers.set(id, handler);
}

/** Remove a handler (component teardown). */
export function unregisterHandler(id: string): void {
  handlers.delete(id);
}

/** Set the checked state of a checkable command (menu ✓ marks). */
export function setChecked(id: string, value: boolean): void {
  const spec = COMMAND_BY_ID.get(id);
  if (!spec?.checkable) {
    throw new Error(`setChecked: "${id}" is not a checkable command`);
  }
  checked.set(id, value);
}

/** Read the checked state of a checkable command. */
export function isChecked(id: string): boolean {
  return checked.get(id) === true;
}

/** True when the command is implemented and its context is active (07 §7.9.1). */
export function isEnabled(
  spec: CommandSpec,
  flags: CommandRuntimeFlags,
): boolean {
  if (spec.implementedIn === null) return false;
  if (spec.context !== "global" && flags.focusedContext !== spec.context) {
    return false;
  }
  if (REQUIRES_PROJECT.has(spec.id) && !flags.projectOpen) return false;
  return handlers.has(spec.id);
}

/** True when the command exists but ships in a later version (tooltip marker). */
export function isLaterVersion(spec: CommandSpec): boolean {
  return spec.implementedIn === null;
}

/**
 * The runtime view a menu/palette row renders from: the spec plus its
 * effective enable state and checked state.
 */
export interface CommandRow {
  spec: CommandSpec;
  enabled: boolean;
  laterVersion: boolean;
  checked: boolean;
}

/** Snapshot for rendering (menus, palette). */
export function commandRow(
  spec: CommandSpec,
  flags: CommandRuntimeFlags,
): CommandRow {
  return {
    spec,
    enabled: isEnabled(spec, flags),
    laterVersion: isLaterVersion(spec),
    checked: spec.checkable && isChecked(spec.id),
  };
}

/** Run a command if it is currently enabled; returns whether it ran. */
export async function runCommand(
  spec: CommandSpec,
  flags: CommandRuntimeFlags,
): Promise<boolean> {
  if (!isEnabled(spec, flags)) return false;
  const handler = handlers.get(spec.id);
  if (!handler) return false;
  await handler();
  return true;
}

/** Test-only: clear all runtime registrations. */
export function resetCommandRuntime(): void {
  handlers.clear();
  checked.clear();
}
