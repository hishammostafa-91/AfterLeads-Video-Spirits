// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * The project store (FR-013): the open snapshot, dirty state (from
 * `model:dirty`), recents, the E1007 lock conflict pending a user
 * choice, and the last envelope for DLG-level messages. All IPC goes
 * through the typed wrappers (FR-016 AC1).
 */
import { create } from "zustand";
import type { ErrorEnvelope } from "../ipc/bindings/ErrorEnvelope";
import type { NewProjectRequest } from "../ipc/bindings/NewProjectRequest";
import type { ProjectSnapshot } from "../ipc/bindings/ProjectSnapshot";
import type { RecentProject } from "../ipc/bindings/RecentProject";
import { project } from "../ipc/commands";
import { isEnvelope } from "../ipc/transport";

/** A lock conflict awaiting the user's choice (E1007 recovery). */
export interface LockConflict {
  path: string;
  takeoverAvailable: boolean;
}

interface ProjectState {
  snapshot: ProjectSnapshot | null;
  /** From `model:dirty` (04 s4.10). */
  dirty: boolean;
  recents: RecentProject[];
  /** Recents whose file was not found (E1001): the missing
   * indicator, never an error toast (FR-011 AC2). */
  missingPaths: Set<string>;
  /** The E1007 flow's pending choice. */
  lockConflict: LockConflict | null;
  /** The last envelope for DLG-level messages (cleared on render). */
  lastError: ErrorEnvelope | null;
  /** Install the open snapshot (IPC flows + open-file events). */
  setSnapshot: (snapshot: ProjectSnapshot | null) => void;
  /** The `model:dirty` event payload. */
  setDirty: (dirty: boolean) => void;
  /** Record a rejection envelope (typed or normalized). */
  recordError: (rejection: unknown) => void;
  /** Record a missing recent file (E1001): the missing indicator. */
  recordMissing: (path: string) => void;
  clearError: () => void;
  refreshRecents: () => Promise<void>;
  /** DLG-01's Create: project_new + install. */
  newProject: (request: NewProjectRequest) => Promise<boolean>;
  /** file.openProject: project_open with the E1007 branch. */
  openPath: (path: string) => Promise<boolean>;
  /** E1007 recovery: open accepting the lock. */
  openReadonly: (path: string) => Promise<boolean>;
  /** E1007 recovery: the confirmed take-over. */
  takeOverLock: (path: string) => Promise<boolean>;
  save: () => Promise<boolean>;
  saveAs: (path: string) => Promise<boolean>;
  /** After DLG-20 resolved. */
  close: (discardChanges: boolean) => Promise<boolean>;
  dismissLockConflict: () => void;
}

function envelopeOf(rejection: unknown): ErrorEnvelope | null {
  return isEnvelope(rejection) ? rejection : null;
}

export const useProjectStore = create<ProjectState>((set, get) => ({
  snapshot: null,
  dirty: false,
  recents: [],
  missingPaths: new Set<string>(),
  lockConflict: null,
  lastError: null,
  setSnapshot: (snapshot) => {
    set({ snapshot, dirty: false, lockConflict: null });
  },
  setDirty: (dirty) => {
    set({ dirty });
  },
  recordError: (rejection) => {
    const envelope = envelopeOf(rejection);
    if (envelope) set({ lastError: envelope });
  },
  /** Record a missing recent (E1001) on any open attempt. */
  recordMissing: (path: string) => {
    set((state) => ({
      missingPaths: new Set(state.missingPaths).add(path),
    }));
  },
  clearError: () => {
    set({ lastError: null });
  },
  refreshRecents: async () => {
    try {
      const recents = await project.recent();
      set({ recents });
    } catch (rejection) {
      get().recordError(rejection);
    }
  },
  newProject: async (request) => {
    try {
      const snapshot = await project.new(request);
      get().setSnapshot(snapshot);
      void get().refreshRecents();
      return true;
    } catch (rejection) {
      get().recordError(rejection);
      return false;
    }
  },
  openPath: async (path) => {
    try {
      const snapshot = await project.open(path);
      get().setSnapshot(snapshot);
      void get().refreshRecents();
      return true;
    } catch (rejection) {
      const envelope = envelopeOf(rejection);
      if (envelope !== null && envelope.code === "E1007") {
        set({
          lockConflict: {
            path,
            takeoverAvailable: envelope.params.takeoverAvailable === "true",
          },
        });
        return false;
      }
      if (envelope !== null && envelope.code === "E1001") {
        get().recordMissing(path);
        return false;
      }
      get().recordError(rejection);
      return false;
    }
  },
  openReadonly: async (path) => {
    try {
      const snapshot = await project.openReadonly(path);
      get().setSnapshot(snapshot);
      void get().refreshRecents();
      return true;
    } catch (rejection) {
      get().recordError(rejection);
      return false;
    }
  },
  takeOverLock: async (path) => {
    try {
      const snapshot = await project.takeOverLock(path);
      get().setSnapshot(snapshot);
      void get().refreshRecents();
      return true;
    } catch (rejection) {
      get().recordError(rejection);
      return false;
    }
  },
  save: async () => {
    try {
      await project.save();
      set({ dirty: false });
      void get().refreshRecents();
      return true;
    } catch (rejection) {
      get().recordError(rejection);
      return false;
    }
  },
  saveAs: async (path) => {
    try {
      await project.saveAs(path);
      const snapshot = get().snapshot;
      if (snapshot) {
        get().setSnapshot({ ...snapshot, path, readOnly: null });
      }
      set({ dirty: false });
      void get().refreshRecents();
      return true;
    } catch (rejection) {
      get().recordError(rejection);
      return false;
    }
  },
  close: async (discardChanges) => {
    try {
      await project.close(discardChanges);
      set({ snapshot: null, dirty: false, lockConflict: null });
      void get().refreshRecents();
      return true;
    } catch (rejection) {
      get().recordError(rejection);
      return false;
    }
  },
  dismissLockConflict: () => {
    set({ lockConflict: null });
  },
}));

/** Test helper: reset to the empty state. */
export function resetProjectStoreForTests(): void {
  useProjectStore.setState({
    snapshot: null,
    dirty: false,
    recents: [],
    missingPaths: new Set<string>(),
    lockConflict: null,
    lastError: null,
  });
}
