// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * The keyboard dispatcher (REQ-I18N-005; 12 §12.10 preamble;
 * 07 §7.9.2): one window-level keydown listener matching physical
 * `KeyboardEvent.code` chords against the enabled commands for the
 * active context. Text-field exception list and IME suppression are
 * exactly the documented rules.
 */
import { chord, COMMANDS, type CommandSpec } from "./catalog";
import { isEnabled, runCommand, type CommandRuntimeFlags } from "./registry";

/** The minimal event surface the matcher needs (tests construct these). */
export interface KeyEventLike {
  code: string;
  ctrlKey: boolean;
  shiftKey: boolean;
  altKey: boolean;
  isComposing: boolean;
  defaultPrevented: boolean;
  target: EventTarget | null;
  preventDefault(): void;
}

/**
 * Chords that still fire while an editable text field has focus
 * (07 §7.9.2, verbatim list): Ctrl+S, Ctrl+Shift+S, Ctrl+E,
 * Ctrl+Shift+P, F1, F6/Shift+F6, F11.
 */
const TEXT_FIELD_CHORDS: ReadonlySet<string> = new Set([
  "Ctrl+KeyS",
  "Ctrl+Shift+KeyS",
  "Ctrl+KeyE",
  "Ctrl+Shift+KeyP",
  "F1",
  "F6",
  "Shift+F6",
  "F11",
]);

/** Editable text fields: text-like inputs, textareas, contenteditable. */
export function isEditableTextTarget(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) return false;
  if (target.isContentEditable) return true;
  if (target instanceof HTMLTextAreaElement) return true;
  if (target instanceof HTMLInputElement) {
    return ![
      "button",
      "checkbox",
      "color",
      "file",
      "image",
      "radio",
      "range",
      "reset",
      "submit",
    ].includes(target.type);
  }
  return false;
}

/** The chord string of an event (physical code; matches catalog chords). */
export function eventChord(event: KeyEventLike): string {
  const parts: string[] = [];
  if (event.ctrlKey) parts.push("Ctrl");
  if (event.shiftKey) parts.push("Shift");
  if (event.altKey) parts.push("Alt");
  parts.push(event.code);
  return parts.join("+");
}

/**
 * Find the command an event triggers under the current flags, or null
 * when no enabled command matches — applying the documented rules:
 * already-handled events are ignored (deeper UI layers win), text
 * fields receive every key except the still-fire list, and single-key
 * shortcuts are suppressed during IME composition.
 */
export function resolveCommand(
  event: KeyEventLike,
  flags: CommandRuntimeFlags,
): CommandSpec | null {
  if (event.defaultPrevented) return null;
  const chordString = eventChord(event);

  // Text-field exception (07 §7.9.2): the field receives every key
  // except the documented still-fire chords.
  if (
    isEditableTextTarget(event.target) &&
    !TEXT_FIELD_CHORDS.has(chordString)
  ) {
    return null;
  }

  // IME (12 §12.10): single-key shortcuts are suppressed during
  // composition; modified chords still fire.
  const isSingleKey = !event.ctrlKey && !event.shiftKey && !event.altKey;
  if (event.isComposing && isSingleKey) return null;

  // Resolve against the focused context: a command is a candidate when
  // its context is global or equals the focused panel's context.
  const contextOk = (spec: CommandSpec): boolean =>
    spec.context === "global" || spec.context === flags.focusedContext;
  let match: CommandSpec | null = null;
  for (const command of COMMANDS) {
    if (!contextOk(command)) continue;
    if (!command.shortcuts.some((s) => chord(s) === chordString)) continue;
    // Only enabled commands consume the key; a disabled (including
    // later-version) command leaves the key to the page.
    if (!isEnabled(command, flags)) continue;
    if (match !== null) {
      // The catalog guarantees no conflicts within any context; a
      // second match means the guarantee was violated.
      throw new Error(
        `keyboard conflict: ${chordString} matched ${match.id} and ${command.id}`,
      );
    }
    match = command;
  }
  return match;
}

/**
 * The window surface the dispatcher needs — structural so tests can
 * install a fake without casts and the real Window satisfies it.
 */
export interface DispatcherWindow {
  addEventListener(
    type: "keydown",
    listener: (event: KeyboardEvent) => void,
  ): void;
  removeEventListener(
    type: "keydown",
    listener: (event: KeyboardEvent) => void,
  ): void;
}

/**
 * Install the window-level listener. Returns the teardown function.
 * The shell calls this once; menus/dialogs that handle keys first win
 * through the defaultPrevented check.
 */
export function installDispatcher(
  win: DispatcherWindow,
  getFlags: () => CommandRuntimeFlags,
): () => void {
  const onKeyDown = (keyEvent: KeyboardEvent): void => {
    const match = resolveCommand(
      {
        code: keyEvent.code,
        ctrlKey: keyEvent.ctrlKey,
        shiftKey: keyEvent.shiftKey,
        altKey: keyEvent.altKey,
        isComposing: keyEvent.isComposing,
        defaultPrevented: keyEvent.defaultPrevented,
        target: keyEvent.target,
        preventDefault: () => {
          keyEvent.preventDefault();
        },
      },
      getFlags(),
    );
    if (match !== null) {
      keyEvent.preventDefault();
      void runCommand(match, getFlags());
    }
  };
  win.addEventListener("keydown", onKeyDown);
  return () => {
    win.removeEventListener("keydown", onKeyDown);
  };
}
