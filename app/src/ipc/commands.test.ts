// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * IPC layer tests (T022; FR-016 AC2): wrappers route through the
 * transport seam, rejections normalize to the envelope, the mock
 * covers every implemented command, and nothing outside `ipc/`
 * issues raw invokes.
 */
import { readdirSync, readFileSync, statSync } from "node:fs";
import { join } from "node:path";
import { beforeEach, describe, expect, it } from "vitest";
import { app } from "./commands";
import {
  invokeNormalized,
  isEnvelope,
  resetIpcTransportForTests,
  setIpcTransport,
} from "./transport";
import { ipcMockTransport } from "../harness/mock/ipcMock";

describe("t_app_006 — typed wrappers (04 s4.2 rule 1)", () => {
  beforeEach(() => {
    resetIpcTransportForTests();
  });

  it("routes through the bound transport", async () => {
    const mock = ipcMockTransport();
    setIpcTransport(mock);
    const info = await app.info();
    expect(info.identifier).toBe("com.afterleads.videospirits");
    expect(mock.calls.app_info).toBe(1);
    await app.ready();
    expect(mock.calls.app_ready).toBe(1);
    await app.quit({ force: true });
    expect(mock.calls.app_quit).toEqual([{ force: true }]);
  });

  it("normalizes a framework rejection to an E0001 envelope", async () => {
    setIpcTransport({
      invoke: () => Promise.reject(new Error("no __TAURI_INTERNALS__")),
    });
    await expect(app.info()).rejects.toMatchObject({
      code: "E0001",
      messageKey: "error.E0001",
    });
  });

  it("passes backend envelopes through untouched", async () => {
    const envelope = {
      code: "E1008",
      severity: "error",
      messageKey: "error.E1008",
      params: {},
      retryable: false,
      recovery: [],
    } as const;
    expect(isEnvelope(envelope)).toBe(true);
    setIpcTransport({
      // The backend rejects with the raw envelope object (Tauri
      // serializes Result errors as plain JSON) - the foreign shape
      // `isEnvelope` must recognize; not an Error by design.
      // eslint-disable-next-line @typescript-eslint/prefer-promise-reject-errors
      invoke: () => Promise.reject(envelope),
    });
    await expect(app.quit({ force: false })).rejects.toBe(envelope);
  });

  it("invokeNormalized resolves values from the transport", async () => {
    setIpcTransport({
      invoke: <T>() => Promise.resolve(7 as T),
    });
    await expect(invokeNormalized<number>("cmd")).resolves.toBe(7);
  });
});

describe("t_app_006 — no raw invokes outside the IPC layer (FR-016 AC2)", () => {
  const srcRoot = join(import.meta.dirname, "..");
  const offenders: string[] = [];
  const walk = (dir: string): void => {
    for (const entry of readdirSync(dir)) {
      const full = join(dir, entry);
      if (statSync(full).isDirectory()) {
        if (entry === "bindings") continue;
        walk(full);
      } else if (/\.(ts|tsx)$/.test(entry)) {
        const source = readFileSync(full, "utf8");
        const insideIpc = full
          .replace(/\//g, "\\")
          .includes(join("src", "ipc"));
        if (
          !insideIpc &&
          /from "@tauri-apps\/api\/core"|[^a-zA-Z]invoke\s*</.test(source)
        ) {
          offenders.push(full.substring(srcRoot.length + 1));
        }
      }
    }
  };
  walk(srcRoot);

  it("has no offenders", () => {
    expect(offenders).toEqual([]);
  });
});
