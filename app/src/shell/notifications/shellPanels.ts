// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * Shell-owned panel state (FR-004's anatomy controls): the bell's
 * notification center and the jobs access are panel controls, not
 * KB commands - this store is the seam they use to ask the dock to
 * open/focus their panel.
 */
import { create } from "zustand";

/** Shell panels that are not PNL placeholders. */
export type ShellPanelId = "notifications";

/** Locale key of a shell panel's title (tabs render these). */
export const SHELL_PANEL_TITLE_KEY: Readonly<Record<ShellPanelId, string>> = {
  notifications: "notifications.title",
};

interface ShellPanelsState {
  /** Panels currently requested open. */
  open: Set<ShellPanelId>;
  /** Ask the dock to open (or, when open, focus) a shell panel. */
  openPanel: (id: ShellPanelId) => void;
  /** A monotonically increasing request counter so repeated opens
   * of the same panel retrigger the dock effect. */
  requestSeq: number;
}

export const useShellPanelsStore = create<ShellPanelsState>((set) => ({
  open: new Set<ShellPanelId>(),
  requestSeq: 0,
  openPanel: (id) => {
    set((state) => {
      const open = new Set(state.open);
      open.add(id);
      return { open, requestSeq: state.requestSeq + 1 };
    });
  },
}));

/** Test helper. */
export function resetShellPanelsForTests(): void {
  useShellPanelsStore.setState({ open: new Set(), requestSeq: 0 });
}
