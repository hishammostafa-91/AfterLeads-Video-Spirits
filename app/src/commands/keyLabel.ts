// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * Shortcut display labels (REQ-I18N-005; 07 §7.9.2): shortcuts bind
 * to physical `KeyboardEvent.code` values; the displayed label uses
 * the current keyboard layout's characters when
 * `navigator.keyboard.getLayoutMap()` is available, else the US
 * label (12 §12.10).
 */
import type { Shortcut } from "./catalog";

/** US-layout labels per physical code (12 §12.10: "the key names below are the US-layout labels"). */
const US_LABELS: Readonly<Record<string, string>> = {
  Space: "Space",
  Enter: "Enter",
  Tab: "Tab",
  Delete: "Delete",
  Home: "Home",
  End: "End",
  ArrowLeft: "Left",
  ArrowRight: "Right",
  ArrowUp: "Up",
  ArrowDown: "Down",
  Comma: ",",
  Period: ".",
  Semicolon: ";",
  Quote: "'",
  Backquote: "`",
  BracketLeft: "[",
  BracketRight: "]",
  Equal: "=",
  Minus: "-",
  Slash: "/",
};

/** The US-layout label for a physical key code. */
export function usLabel(code: string): string {
  const known = US_LABELS[code];
  if (known !== undefined) return known;
  if (/^Key[A-Z]$/.test(code)) return code.slice(3);
  if (/^Digit\d$/.test(code)) return code.slice(5);
  if (/^F\d{1,2}$/.test(code)) return code;
  return code;
}

/** A layout map: physical code → the current layout's character. */
export type LayoutMapLike = { get(code: string): string | undefined };

/** The displayed label for one key, honoring a layout map when present. */
export function keyLabel(
  code: string,
  layoutMap?: LayoutMapLike | null,
): string {
  const mapped = layoutMap?.get(code);
  return mapped && mapped.length > 0 ? mapped : usLabel(code);
}

/** The displayed chord, e.g. "Ctrl+Shift+N" (modifiers first, plus-separated). */
export function chordLabel(
  shortcut: Shortcut,
  layoutMap?: LayoutMapLike | null,
): string {
  const parts: string[] = [];
  if (shortcut.ctrl) parts.push("Ctrl");
  if (shortcut.shift) parts.push("Shift");
  if (shortcut.alt) parts.push("Alt");
  parts.push(keyLabel(shortcut.key, layoutMap));
  return parts.join("+");
}

/** True when the running browser exposes the keyboard layout map. */
export function layoutMapSupported(): boolean {
  return (
    typeof navigator !== "undefined" &&
    "keyboard" in navigator &&
    typeof (
      navigator as Navigator & {
        keyboard?: { getLayoutMap?: unknown };
      }
    ).keyboard?.getLayoutMap === "function"
  );
}
