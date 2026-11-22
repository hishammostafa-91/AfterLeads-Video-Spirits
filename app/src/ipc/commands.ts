// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * The typed command wrappers (04 s4.2 rule 1): the UI calls IPC only
 * through this module (`app.info()`, never a raw `invoke`). Types
 * come from the ts-rs bindings (ADR-010); rejections are the error
 * envelope (04 s4.9).
 */
import type { AppInfo } from "./bindings/AppInfo";
import { invokeNormalized } from "./transport";

/** The application commands (04 s4.3.1). */
export const app = {
  /** Boot report; rejects with the envelope. */
  info(): Promise<AppInfo> {
    return invokeNormalized<AppInfo>("app_info");
  },
  /** The UI mounted; the core emits current state events. */
  ready(): Promise<void> {
    return invokeNormalized("app_ready");
  },
  /**
   * Run the shutdown sequence (02 s2.5). `force` is true once the UI
   * resolved the dirty gate (DLG-20).
   */
  quit(request: { force: boolean }): Promise<void> {
    return invokeNormalized("app_quit", request);
  },
};
