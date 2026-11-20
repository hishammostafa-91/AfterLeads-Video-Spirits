// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * T-APP-003 (enablement) and T-I18N-005 (dispatch) unit coverage:
 * the dispatcher matches physical codes regardless of layout labels,
 * applies the text-field exception list and IME suppression exactly
 * per 07 §7.9.2 / 12 §12.10, and the registry only runs implemented,
 * context-active commands.
 */
import { afterEach, describe, expect, it, vi } from "vitest";
import { COMMAND_BY_ID, type CommandSpec } from "./catalog";
import {
  eventChord,
  installDispatcher,
  isEditableTextTarget,
  resolveCommand,
  type KeyEventLike,
} from "./dispatcher";
import {
  commandRow,
  isEnabled,
  isChecked,
  registerHandler,
  resetCommandRuntime,
  runCommand,
  setChecked,
  type CommandRuntimeFlags,
} from "./registry";

/** Fetch a spec or fail loudly (tests reference catalog ids). */
const spec = (id: string): CommandSpec => {
  const found = COMMAND_BY_ID.get(id);
  if (!found) throw new Error(`test setup: unknown command "${id}"`);
  return found;
};

/** A physical key event (no layout concept — codes only). */
const key = (overrides: Partial<KeyEventLike> = {}): KeyEventLike => ({
  code: "KeyS",
  ctrlKey: true,
  shiftKey: false,
  altKey: false,
  isComposing: false,
  defaultPrevented: false,
  target: document.body,
  preventDefault: () => {
    return undefined;
  },
  ...overrides,
});

const shellFlags: CommandRuntimeFlags = {
  focusedContext: "global",
  projectOpen: true,
};
const withSave: CommandRuntimeFlags = shellFlags;
const withoutProject: CommandRuntimeFlags = {
  focusedContext: "global",
  projectOpen: false,
};

afterEach(() => {
  resetCommandRuntime();
});

describe("t_app_003 — registry enablement (REQ-APP-004)", () => {
  it("later-version commands are never enabled and carry the marker", () => {
    const exportSpec = spec("file.export");
    expect(exportSpec.implementedIn).toBeNull();
    expect(isEnabled(exportSpec, withSave)).toBe(false);
    expect(commandRow(exportSpec, withSave).laterVersion).toBe(true);
  });

  it("an implemented command with a handler is enabled", () => {
    const saveSpec = spec("file.save");
    registerHandler("file.save", () => {
      return undefined;
    });
    expect(isEnabled(saveSpec, withSave)).toBe(true);
  });

  it("save/saveAs/close need an open project", () => {
    registerHandler("file.save", () => {
      return undefined;
    });
    registerHandler("file.saveAs", () => {
      return undefined;
    });
    registerHandler("file.closeProject", () => {
      return undefined;
    });
    registerHandler("file.newProject", () => {
      return undefined;
    });
    expect(isEnabled(spec("file.save"), withoutProject)).toBe(false);
    expect(isEnabled(spec("file.saveAs"), withoutProject)).toBe(false);
    expect(isEnabled(spec("file.closeProject"), withoutProject)).toBe(false);
    expect(isEnabled(spec("file.newProject"), withoutProject)).toBe(true);
  });

  it("handlers are refused for unknown and unimplemented commands", () => {
    const handler = () => {
      return Promise.resolve();
    };
    expect(() => {
      registerHandler("nope.nope", handler);
    }).toThrow(/unknown/);
    expect(() => {
      registerHandler("file.export", handler);
    }).toThrow(/not implemented/);
  });

  it("runCommand executes only enabled commands", async () => {
    const run = vi.fn();
    registerHandler("view.commandPalette", run);
    expect(await runCommand(spec("view.commandPalette"), shellFlags)).toBe(
      true,
    );
    expect(run).toHaveBeenCalled();
    expect(await runCommand(spec("file.export"), shellFlags)).toBe(false);
  });

  it("checked state only on checkable commands", () => {
    expect(() => {
      setChecked("timeline.toggleSnapping", true);
    }).not.toThrow();
    expect(isChecked("timeline.toggleSnapping")).toBe(true);
    expect(() => {
      setChecked("file.save", true);
    }).toThrow(/checkable/);
  });
});

const registerStub = (id: string): boolean => {
  try {
    registerHandler(id, () => {
      return undefined;
    });
    return true;
  } catch {
    return false;
  }
};

describe("t_i18n_005 — physical-key dispatch (07 s7.9.2)", () => {
  it("matches by physical code, not layout label", () => {
    registerHandler("file.save", () => {
      return undefined;
    });
    registerHandler("view.workspaceEdit", () => {
      return undefined;
    });
    const timelineFlags: CommandRuntimeFlags = {
      focusedContext: "timeline",
      projectOpen: true,
    };
    // The same physical position: KeyS with Ctrl on ANY layout.
    expect(resolveCommand(key(), timelineFlags)?.id).toBe("file.save");
    // Alt+Digit1 is workspace Edit — physical Digit1, whatever glyph
    // the layout prints on it.
    expect(
      resolveCommand(
        key({ code: "Digit1", ctrlKey: false, altKey: true }),
        timelineFlags,
      )?.id,
    ).toBe("view.workspaceEdit");
  });

  it("global commands fire in every context; scoped later-version ones never fire", () => {
    registerHandler("file.save", () => {
      return undefined;
    });
    const mediaFlags: CommandRuntimeFlags = {
      focusedContext: "media",
      projectOpen: true,
    };
    expect(resolveCommand(key(), mediaFlags)?.id).toBe("file.save");
    // clip.split is timeline-scoped AND a later-version command: no
    // handler can exist for it (registry refuses), so it never fires —
    // not even with the timeline focused.
    const timelineFlags: CommandRuntimeFlags = {
      focusedContext: "timeline",
      projectOpen: true,
    };
    expect(
      resolveCommand(key({ code: "KeyK", ctrlKey: true }), timelineFlags),
    ).toBeNull();
    expect(registerStub("clip.split")).toBe(false);
  });

  it("text fields receive every key except the documented still-fire list", () => {
    registerHandler("file.save", () => {
      return undefined;
    });
    registerHandler("file.saveAs", () => {
      return undefined;
    });
    registerHandler("view.commandPalette", () => {
      return undefined;
    });
    const field = document.createElement("input");
    field.type = "text";
    const inField = (e: Partial<KeyEventLike>) => key({ target: field, ...e });

    // Ctrl+C in a field is copy — no command may grab it.
    expect(resolveCommand(inField({ code: "KeyC" }), withSave)).toBeNull();
    // The still-fire list passes through even in a field (enabled ones match).
    expect(
      resolveCommand(inField({ code: "KeyS", ctrlKey: true }), withSave)?.id,
    ).toBe("file.save");
    expect(
      resolveCommand(
        inField({ code: "KeyS", ctrlKey: true, shiftKey: true }),
        withSave,
      )?.id,
    ).toBe("file.saveAs");
    expect(
      resolveCommand(
        inField({ code: "KeyP", ctrlKey: true, shiftKey: true }),
        withSave,
      )?.id,
    ).toBe("view.commandPalette");
    // Ctrl+E is on the still-fire list (reserved); file.export is a
    // later-version command, so the key stays with the field.
    expect(
      resolveCommand(inField({ code: "KeyE", ctrlKey: true }), withSave),
    ).toBeNull();
  });

  it("isEditableTextTarget: text inputs and textareas yes; buttons, radios, contenteditable", () => {
    const text = document.createElement("input");
    text.type = "text";
    const radio = document.createElement("input");
    radio.type = "radio";
    // jsdom does not compute isContentEditable from the attribute;
    // assert it directly like a real browser would report it.
    const editable = document.createElement("div");
    Object.defineProperty(editable, "isContentEditable", { value: true });
    expect(isEditableTextTarget(text)).toBe(true);
    expect(isEditableTextTarget(document.createElement("textarea"))).toBe(true);
    expect(isEditableTextTarget(editable)).toBe(true);
    expect(isEditableTextTarget(radio)).toBe(false);
    expect(isEditableTextTarget(document.createElement("button"))).toBe(false);
  });

  it("single-key shortcuts are suppressed during IME composition; chords still fire", () => {
    registerHandler("view.maximizePanel", () => {
      return undefined;
    });
    registerHandler("file.save", () => {
      return undefined;
    });
    const composing = { isComposing: true };
    // ` (Backquote) single key while composing → suppressed.
    expect(
      resolveCommand(
        key({ code: "Backquote", ctrlKey: false, ...composing }),
        withSave,
      ),
    ).toBeNull();
    // Ctrl+S is a modified chord → still fires during composition.
    expect(resolveCommand(key(composing), withSave)?.id).toBe("file.save");
  });

  it("already-handled events and disabled commands never consume keys", () => {
    registerHandler("file.save", () => {
      return undefined;
    });
    expect(
      resolveCommand(key({ defaultPrevented: true }), withSave),
    ).toBeNull();
    // No handler registered → disabled → not consumed.
    resetCommandRuntime();
    expect(resolveCommand(key(), withSave)).toBeNull();
  });

  it("eventChord renders code-based chords", () => {
    expect(eventChord(key())).toBe("Ctrl+KeyS");
    expect(
      eventChord(key({ ctrlKey: false, shiftKey: true, code: "F6" })),
    ).toBe("Shift+F6");
  });
});

describe("installDispatcher", () => {
  it("dispatches enabled matches with preventDefault, ignores the rest", () => {
    const run = vi.fn();
    registerHandler("view.commandPalette", run);
    const added: Array<(event: KeyboardEvent) => void> = [];
    const removed: Array<(event: KeyboardEvent) => void> = [];
    const fakeWindow = {
      addEventListener: (
        _type: "keydown",
        listener: (event: KeyboardEvent) => void,
      ) => {
        added.push(listener);
      },
      removeEventListener: (
        _type: "keydown",
        listener: (event: KeyboardEvent) => void,
      ) => {
        removed.push(listener);
      },
    };
    const teardown = installDispatcher(fakeWindow, () => shellFlags);
    expect(added).toHaveLength(1);
    const listener = added[0];

    const matched = new KeyboardEvent("keydown", {
      code: "KeyP",
      ctrlKey: true,
      shiftKey: true,
      cancelable: true,
    });
    Object.defineProperty(matched, "isComposing", { value: false });
    listener(matched);
    expect(matched.defaultPrevented).toBe(true);
    // The handler call itself is synchronous inside the async wrapper.
    expect(run).toHaveBeenCalledTimes(1);

    const unmatched = new KeyboardEvent("keydown", {
      code: "KeyX",
      ctrlKey: true,
      cancelable: true,
    });
    listener(unmatched);
    expect(unmatched.defaultPrevented).toBe(false);
    expect(run).toHaveBeenCalledTimes(1);

    teardown();
    expect(removed).toEqual([listener]);
  });
});
