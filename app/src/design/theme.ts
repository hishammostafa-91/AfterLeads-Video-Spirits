// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * Theme, accent, and zoom state (REQ-APP-009/010; 07 §7.4; research
 * D2/D3). Switching applies `data-theme` / `data-accent` /
 * `--app-zoom` on the document root immediately — no reload. High
 * Contrast is both the automatic forced-colors trigger and a manually
 * selectable theme value (owner ruling F05; one shared system-color
 * mapping in tokens.css). The persistence seam (research D12) lets the
 * harness bind URL params now and WP-05 bind `alvs.appearance.*`
 * settings keys later.
 */
import { create } from "zustand";

export type ThemeName = "dark" | "light" | "high-contrast";
export type Accent = "clr-06" | "clr-07" | "clr-04" | "clr-02" | "clr-08";

export const MIN_ZOOM = 80;
export const MAX_ZOOM = 200;

/** Clamp an application zoom into the documented 80–200 range. */
export function clampZoom(zoom: number): number {
  return Math.min(MAX_ZOOM, Math.max(MIN_ZOOM, Math.round(zoom)));
}

/** Whether Windows forced colors (contrast themes) are active (§7.4.3). */
export function forcedColorsActive(): boolean {
  return (
    typeof window !== "undefined" &&
    typeof window.matchMedia === "function" &&
    window.matchMedia("(forced-colors: active)").matches
  );
}

interface ThemeState {
  theme: ThemeName;
  accent: Accent;
  zoom: number;
  setTheme: (theme: ThemeName) => void;
  setAccent: (accent: Accent) => void;
  setZoom: (zoom: number) => void;
}

function applyDocument(state: ThemeState): void {
  const root = document.documentElement;
  root.dataset.theme = state.theme;
  root.dataset.accent = state.accent;
  root.style.setProperty("--app-zoom", String(state.zoom));
}

export const useThemeStore = create<ThemeState>((set) => ({
  theme: "dark",
  accent: "clr-06",
  zoom: 100,
  setTheme: (theme) => {
    set({ theme });
    const state = useThemeStore.getState();
    applyDocument({ ...state, theme });
    themePersistence.write({ theme });
  },
  setAccent: (accent) => {
    set({ accent });
    const state = useThemeStore.getState();
    applyDocument({ ...state, accent });
    themePersistence.write({ accent });
  },
  setZoom: (zoom) => {
    const clamped = clampZoom(zoom);
    set({ zoom: clamped });
    const state = useThemeStore.getState();
    applyDocument({ ...state, zoom: clamped });
    themePersistence.write({ zoom: clamped });
  },
}));

/** The persistence seam (research D12). */
export interface ThemePersistence {
  read(): Partial<{ theme: ThemeName; accent: Accent; zoom: number }>;
  write(
    state: Partial<{ theme: ThemeName; accent: Accent; zoom: number }>,
  ): void;
}

const noopPersistence: ThemePersistence = {
  read: () => ({}),
  write: () => {},
};

let themePersistence: ThemePersistence = noopPersistence;

/** Bind a persistence backend (harness: URL params; WP-05: settings). */
export function setThemePersistence(persistence: ThemePersistence): void {
  themePersistence = persistence;
  const restored = persistence.read();
  if (restored.theme !== undefined) {
    useThemeStore.getState().setTheme(restored.theme);
  }
  if (restored.accent !== undefined) {
    useThemeStore.getState().setAccent(restored.accent);
  }
  if (restored.zoom !== undefined) {
    useThemeStore.getState().setZoom(restored.zoom);
  }
}

/** Test helper: detach persistence and reset to the defaults. */
export function resetThemeStoreForTests(): void {
  themePersistence = noopPersistence;
  useThemeStore.setState({ theme: "dark", accent: "clr-06", zoom: 100 });
}
