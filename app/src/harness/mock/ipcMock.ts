// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * The mock IPC transport (ADR-020's mock backend): answers every
 * implemented command (04 s4.3.1 app commands) with fixture data and
 * records the calls - story and test data only, no Tauri shell.
 */
import type { IpcTransport } from "../../ipc/transport";
import type { AppInfo } from "../../ipc/bindings/AppInfo";

/** Fixture `app_info` - mock values, the honest unknown renderer. */
export function appInfoFixture(): AppInfo {
  return {
    version: "0.1.0",
    commit: "mock",
    buildDate: "2026-01-01",
    identifier: "com.afterleads.videospirits",
    renderer: { name: "unknown", isWarp: false },
    safeMode: false,
    portable: false,
    locale: "en",
  };
}

/** The recorded calls of a mock transport. */
export interface MockCalls {
  app_info: number;
  app_ready: number;
  app_quit: Array<{ force: boolean }>;
}

/** A mock transport over the implemented commands. */
export function ipcMockTransport(): IpcTransport & { calls: MockCalls } {
  const calls: MockCalls = {
    app_info: 0,
    app_ready: 0,
    app_quit: [],
  };
  return {
    calls,
    invoke: <T>(command: string, payload?: unknown): Promise<T> => {
      switch (command) {
        case "app_info":
          calls.app_info += 1;
          return Promise.resolve(appInfoFixture() as T);
        case "app_ready":
          calls.app_ready += 1;
          return Promise.resolve(undefined as T);
        case "app_quit": {
          const request = (payload ?? {}) as { force?: boolean };
          calls.app_quit.push({ force: request.force === true });
          return Promise.resolve(undefined as T);
        }
        default:
          return Promise.reject(
            new Error(`mock backend: command "${command}" is not implemented`),
          );
      }
    },
  };
}
