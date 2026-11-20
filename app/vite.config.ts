// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/// <reference types="vitest/config" />
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import process from "node:process";
const host = process.env.TAURI_DEV_HOST;

// https://vite.dev/config/
export default defineConfig(() => ({
  plugins: [react()],

  // CSS Modules: the components access kebab-case classes through
  // their camelized names (styles.menuBarTrigger from
  // ".menu-bar-trigger"). Vite's default exports the written name
  // only, which silently dropped every kebab-named class in the
  // served build while vitest's pipeline masked it — export the
  // camelized names (camelCaseOnly also fails loudly on typos).
  css: {
    modules: {
      localsConvention: "camelCaseOnly",
    },
  },

  // Vitest (docs/02 §2.14 tests; jsdom for component tests).
  test: {
    environment: "jsdom",
    setupFiles: ["src/test/setup.ts"],
  },

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent Vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      // 3. tell Vite to ignore watching `src-tauri`
      ignored: ["**/src-tauri/**"],
    },
  },
}));
