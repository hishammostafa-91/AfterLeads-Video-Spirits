// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * The harness workspace persistence port: a localStorage-backed
 * implementation of the `alvs.workspaces.*` document (03 s3.6) so
 * restart round-trips are observable in the mock backend
 * (FR-006 AC2). The packaged app binds the settings IPC port over
 * the same seam instead (WP-24).
 */
import type {
  WorkspaceDoc,
  WorkspacePersistence,
} from "../../shell/dock/workspaces";

const STORAGE_KEY = "alvs.harness.workspaces";

function safeParse(raw: string | null): WorkspaceDoc | null {
  if (raw === null) return null;
  try {
    return JSON.parse(raw) as WorkspaceDoc;
  } catch {
    return null;
  }
}

export function localStorageWorkspacePersistence(
  storage: Storage,
): WorkspacePersistence {
  return {
    read: () => Promise.resolve(safeParse(storage.getItem(STORAGE_KEY))),
    write: (doc) => {
      try {
        storage.setItem(STORAGE_KEY, JSON.stringify(doc));
      } catch {
        // Private-mode storage: the harness degrades to in-memory.
      }
      return Promise.resolve();
    },
  };
}

/** Test helper: clear the harness persistence slot. */
export function clearHarnessWorkspaces(storage: Storage): void {
  storage.removeItem(STORAGE_KEY);
}
