// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * The typed command wrappers (04 s4.2 rule 1): the UI calls IPC only
 * through this module (`app.info()`, never a raw `invoke`). Types
 * come from the ts-rs bindings (ADR-010); rejections are the error
 * envelope (04 s4.9).
 */
import type { AppInfo } from "./bindings/AppInfo";
import type { NewProjectRequest } from "./bindings/NewProjectRequest";
import type { ProjectSnapshot } from "./bindings/ProjectSnapshot";
import type { RecentProject } from "./bindings/RecentProject";
import type { SaveResult } from "./bindings/SaveResult";
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

/** The project commands (04 s4.3.2). */
export const project = {
  /** Create `<folder>\<name>\<name>.alvs` and open it (DLG-01). */
  new(request: NewProjectRequest): Promise<ProjectSnapshot> {
    return invokeNormalized<ProjectSnapshot>("project_new", request);
  },
  /**
   * Open writable. Rejects with E1007 when another holder keeps the
   * lock - the recovery actions carry `project.openReadOnly` /
   * `project.takeOverLock`.
   */
  open(path: string): Promise<ProjectSnapshot> {
    return invokeNormalized<ProjectSnapshot>("project_open", { path });
  },
  /** Accept a held lock (used after E1007). */
  openReadonly(path: string): Promise<ProjectSnapshot> {
    return invokeNormalized<ProjectSnapshot>("project_open_readonly", { path });
  },
  /** The confirmed take-over (03 s3.9.4). */
  takeOverLock(path: string): Promise<ProjectSnapshot> {
    return invokeNormalized<ProjectSnapshot>("project_take_over_lock", {
      path,
    });
  },
  /** Save in place; rejects E1008 on read-only projects. */
  save(): Promise<SaveResult> {
    return invokeNormalized<SaveResult>("project_save");
  },
  /** Save to a new path and continue there. */
  saveAs(path: string): Promise<SaveResult> {
    return invokeNormalized<SaveResult>("project_save_as", { path });
  },
  /** Drop the open project (the UI gates dirty closes with DLG-20). */
  close(discardChanges: boolean): Promise<void> {
    return invokeNormalized("project_close", { discardChanges });
  },
  /** The newest-first recents (03 s3.7). */
  recent(): Promise<RecentProject[]> {
    return invokeNormalized<RecentProject[]>("project_recent");
  },
};
