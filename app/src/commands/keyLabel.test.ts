// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * T-I18N-005 (label half): displayed shortcut labels use the current
 * layout's characters when a layout map is available, else the US
 * label (REQ-I18N-005; 07 s7.9.2; 12 s12.10).
 */
import { describe, expect, it } from "vitest";
import type { Shortcut } from "./catalog";
import { chordLabel, keyLabel, layoutMapSupported, usLabel } from "./keyLabel";

const chordOf = (partial: Partial<Shortcut>): Shortcut => ({
  key: "KeyN",
  ctrl: false,
  shift: false,
  alt: false,
  ...partial,
});

describe("t_i18n_005 — shortcut display labels", () => {
  it("US labels for letters, digits, and named keys", () => {
    expect(usLabel("KeyA")).toBe("A");
    expect(usLabel("Digit1")).toBe("1");
    expect(usLabel("F11")).toBe("F11");
    expect(usLabel("ArrowLeft")).toBe("Left");
    expect(usLabel("Comma")).toBe(",");
    expect(usLabel("BracketLeft")).toBe("[");
  });

  it("layout map characters win over US labels", () => {
    // AZERTY: the physical KeyQ position prints "a", KeyW prints "z".
    const azerty = { get: (code: string) => ({ KeyQ: "a", KeyW: "z" })[code] };
    expect(keyLabel("KeyQ", azerty)).toBe("a");
    expect(keyLabel("KeyW", azerty)).toBe("z");
    expect(keyLabel("KeyY", azerty)).toBe("Y"); // unmapped → US
  });

  it("chords render modifiers first, plus-separated", () => {
    expect(chordLabel(chordOf({ ctrl: true }))).toBe("Ctrl+N");
    expect(chordLabel(chordOf({ ctrl: true, shift: true, key: "KeyS" }))).toBe(
      "Ctrl+Shift+S",
    );
    expect(chordLabel(chordOf({ alt: true, key: "Digit1" }))).toBe("Alt+1");
  });

  it("layout-aware chords keep modifiers, translate the key", () => {
    const arabic = { get: () => "ش" };
    expect(chordLabel(chordOf({ ctrl: true, key: "KeyK" }), arabic)).toBe(
      "Ctrl+ش",
    );
  });

  it("layoutMapSupported reflects navigator.keyboard availability", () => {
    // jsdom: no keyboard layout API → false (the US fallback path).
    expect(layoutMapSupported()).toBe(false);
  });
});
