// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * T-UI-001 (09 §9.10): themes, accents, and High Contrast apply
 * without restart (REQ-APP-009). T-UI-002: zoom clamps to 80–200 and
 * drives the CSS zoom var (REQ-APP-010; research D3).
 */
import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  clampZoom,
  forcedColorsActive,
  resetThemeStoreForTests,
  useThemeStore,
} from "./theme";

const root = () => document.documentElement;

beforeEach(() => {
  resetThemeStoreForTests();
  root().removeAttribute("data-theme");
  root().removeAttribute("data-accent");
  root().style.removeProperty("--app-zoom");
});

describe("t_ui_001 themes and accents (REQ-APP-009)", () => {
  it("switches dark, light, and high-contrast without reload", () => {
    useThemeStore.getState().setTheme("dark");
    expect(root().dataset.theme).toBe("dark");
    useThemeStore.getState().setTheme("light");
    expect(root().dataset.theme).toBe("light");
    useThemeStore.getState().setTheme("high-contrast");
    expect(root().dataset.theme).toBe("high-contrast");
  });

  it("defaults to dark with the clr-06 accent", () => {
    useThemeStore.getState().setTheme("dark");
    expect(root().dataset.theme).toBe("dark");
    expect(useThemeStore.getState().accent).toBe("clr-06");
    expect(root().dataset.accent).toBe("clr-06");
  });

  it.each(["clr-07", "clr-04", "clr-02", "clr-08"] as const)(
    "accent %s swaps the accent attribute",
    (accent) => {
      useThemeStore.getState().setAccent(accent);
      expect(root().dataset.accent).toBe(accent);
      expect(useThemeStore.getState().accent).toBe(accent);
    },
  );

  it("forcedColorsActive follows the forced-colors media query", () => {
    const stub = {
      matches: true,
      addEventListener: () => {},
      removeEventListener: () => {},
      dispatchEvent: () => false,
    };
    vi.stubGlobal(
      "matchMedia",
      (query: string): MediaQueryList =>
        ({
          ...stub,
          matches: query.includes("forced-colors"),
        }) as unknown as MediaQueryList,
    );
    expect(forcedColorsActive()).toBe(true);
    vi.unstubAllGlobals();
  });
});

describe("t_ui_002 zoom (REQ-APP-010)", () => {
  it("clamps zoom into the 80-200 range", () => {
    expect(clampZoom(50)).toBe(80);
    expect(clampZoom(80)).toBe(80);
    expect(clampZoom(125)).toBe(125);
    expect(clampZoom(200)).toBe(200);
    expect(clampZoom(300)).toBe(200);
  });

  it("applies the zoom integer as the --app-zoom document var", () => {
    useThemeStore.getState().setZoom(150);
    expect(root().style.getPropertyValue("--app-zoom")).toBe("150");
    useThemeStore.getState().setZoom(400);
    expect(root().style.getPropertyValue("--app-zoom")).toBe("200");
  });
});
