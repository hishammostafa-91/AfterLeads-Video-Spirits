// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * The application shell frame (07 s7.2): the Welcome surface when no
 * project is open (REQ-APP-005), otherwise title bar, toolbar, dock,
 * status bar. Owns the file-command flows (FR-013): DLG-01, the open
 * picker with the E1007 lock-conflict recovery, save/save-as, and
 * the DLG-20 three-way gate on close/exit/open-other; `model:dirty`
 * drives the title-bar dot and the read-only state the banner.
 */
import { useCallback, useEffect, useMemo, useState } from "react";
import { useIntl } from "react-intl";
import type { JSX } from "react";
import {
  open as openDialog,
  save as saveDialog,
} from "@tauri-apps/plugin-dialog";
import { TitleBar } from "./TitleBar";
import { Toolbar } from "./Toolbar";
import { StatusBar } from "./StatusBar";
import { Welcome } from "./Welcome";
import { Dock } from "./dock/Dock";
import { CommandPalette } from "../commands/palette/CommandPalette";
import { installDispatcher } from "../commands/dispatcher";
import {
  registerHandler,
  unregisterHandler,
  type CommandRuntimeFlags,
} from "../commands/registry";
import { useWorkspaceStore } from "./dock/workspaces";
import { NewProjectDialog } from "../dialogs/NewProject";
import { LockConflictDialog } from "../dialogs/LockConflict";
import {
  UnsavedChangesDialog,
  type UnsavedChoice,
} from "../dialogs/UnsavedChanges";
import { app } from "../ipc/commands";
import { onEvent } from "../ipc/events";
import { useProjectStore } from "../projects/store";
import { useStatusStore } from "./status";
import { useNotificationStore } from "./notifications/store";
import styles from "./AppShell.module.css";

/** What waits behind the DLG-20 gate (FR-013 AC2's three triggers). */
type GatedAction =
  | { kind: "close" }
  | { kind: "exit" }
  | { kind: "open"; path: string }
  | { kind: "new" };

export function AppShell(): JSX.Element {
  const { formatMessage } = useIntl();
  const workspace = useWorkspaceStore((state) => state.active);
  const snapshot = useProjectStore((state) => state.snapshot);
  const dirty = useProjectStore((state) => state.dirty);
  const lockConflict = useProjectStore((state) => state.lockConflict);
  const lastError = useProjectStore((state) => state.lastError);
  const [paletteOpen, setPaletteOpen] = useState(false);
  const [newOpen, setNewOpen] = useState(false);
  const [gate, setGate] = useState<GatedAction | null>(null);

  const flags: CommandRuntimeFlags = useMemo(
    () => ({ focusedContext: "global", projectOpen: snapshot !== null }),
    [snapshot],
  );

  const openPicker = useCallback((): void => {
    void openDialog({
      multiple: false,
      filters: [{ name: "AfterLeads-Video-Spirits", extensions: ["alvs"] }],
    }).then((picked) => {
      if (typeof picked === "string") {
        void useProjectStore.getState().openPath(picked);
      }
    });
  }, []);

  /** Continue a gated action (the dirty case is already resolved). */
  const runAction = useCallback((action: GatedAction): void => {
    switch (action.kind) {
      case "close":
        void useProjectStore.getState().close(true);
        break;
      case "exit":
        void app.quit({ force: false }).catch(() => undefined);
        break;
      case "open":
        void useProjectStore.getState().openPath(action.path);
        break;
      case "new":
        setNewOpen(true);
        break;
    }
  }, []);

  /** The gated flows: DLG-20 first when dirty (REQ-APP-012). */
  const runGated = useCallback(
    (action: GatedAction): void => {
      if (useProjectStore.getState().dirty) {
        setGate(action);
        return;
      }
      runAction(action);
    },
    [runAction],
  );

  /** DLG-20's three-way outcome (07 s7.14). */
  const resolveGate = useCallback(
    (choice: UnsavedChoice): void => {
      const action = gate;
      setGate(null);
      if (choice === "cancel" || action === null) return;
      if (choice === "save") {
        void useProjectStore
          .getState()
          .save()
          .then((saved) => {
            if (saved) runAction(action);
          });
        return;
      }
      // "Don't save": drop the project, then continue the action.
      void useProjectStore
        .getState()
        .close(true)
        .then(() => {
          runAction(action);
        });
    },
    [gate, runAction],
  );

  // The command handlers (registry ids; FR-013).
  useEffect(() => {
    const handlers: Array<[string, () => void]> = [
      [
        "file.newProject",
        () => {
          runGated({ kind: "new" });
        },
      ],
      [
        "file.openProject",
        () => {
          openPicker();
        },
      ],
      [
        "file.save",
        () => {
          void useProjectStore.getState().save();
        },
      ],
      [
        "file.saveAs",
        () => {
          void saveDialog({
            filters: [
              { name: "AfterLeads-Video-Spirits", extensions: ["alvs"] },
            ],
          }).then((picked) => {
            if (typeof picked === "string") {
              void useProjectStore.getState().saveAs(picked);
            }
          });
        },
      ],
      [
        "file.closeProject",
        () => {
          runGated({ kind: "close" });
        },
      ],
      [
        "file.exit",
        () => {
          runGated({ kind: "exit" });
        },
      ],
      [
        "view.commandPalette",
        () => {
          setPaletteOpen((open) => !open);
        },
      ],
    ];
    for (const [id, handler] of handlers) registerHandler(id, handler);
    return () => {
      for (const [id] of handlers) unregisterHandler(id);
    };
  }, [openPicker, runGated]);

  // Keyboard dispatcher + boot signal + core events.
  useEffect(() => {
    const teardownDispatcher = installDispatcher(window, () => ({
      focusedContext: "global",
      projectOpen: useProjectStore.getState().snapshot !== null,
    }));
    void app.ready().catch(() => undefined);
    const offDirty = onEvent<{ dirty: boolean }>("model:dirty", (payload) => {
      useProjectStore.getState().setDirty(payload.dirty);
    });
    const offOpenFile = onEvent<{ path: string }>(
      "app:open-file",
      (payload) => {
        // Dirty + forwarded open takes the same DLG-20 gate (FR-001 AC3).
        if (useProjectStore.getState().dirty) {
          setGate({ kind: "open", path: payload.path });
        } else {
          void useProjectStore.getState().openPath(payload.path);
        }
      },
    );
    // The renderer chip from app_info (FR-005 AC2).
    void app
      .info()
      .then((info) => {
        useStatusStore.getState().setRendererFromInfo(info);
      })
      .catch(() => undefined);
    // The notification fabric (FR-014): app:notify feeds history +
    // toasts; autosave:status feeds the chip (FR-005 AC2).
    const offNotify = onEvent<Record<string, unknown>>(
      "app:notify",
      (payload) => {
        useNotificationStore.getState().notify(payload as never);
      },
    );
    const offAutosave = onEvent<{
      state: "idle" | "saving" | "saved" | "failed";
      timeUtc: string | null;
    }>("autosave:status", (payload) => {
      useStatusStore.getState().setAutosave(payload);
    });
    return () => {
      unregisterHandler("view.commandPalette");
      teardownDispatcher();
      for (const off of [offDirty, offOpenFile, offNotify, offAutosave]) {
        void off.then((unsubscribe) => {
          unsubscribe();
        });
      }
    };
  }, []);

  const readOnly = snapshot?.readOnly ?? null;

  return (
    <div className={styles.shell}>
      {snapshot === null ? (
        <Welcome
          onNewProject={() => {
            setNewOpen(true);
          }}
          onOpenProject={openPicker}
          onOpenRecent={(path) => {
            runGated({ kind: "open", path });
          }}
        />
      ) : (
        <>
          <TitleBar
            flags={flags}
            projectName={snapshot.project.name}
            isDirty={dirty}
          />
          <Toolbar flags={flags} activeWorkspace={workspace} />
          {readOnly !== null ? (
            <div className={styles.readOnlyBanner} role="status">
              {formatMessage({ id: "banner.readOnly" })}
            </div>
          ) : null}
          <Dock />
          <StatusBar />
        </>
      )}
      {/* The settings-driven default location arrives with the
         settings IPC (WP-24); DLG-01 starts from the picker. */}
      <NewProjectDialog
        isOpen={newOpen}
        defaultFolder=""
        error={lastError}
        onClose={() => {
          useProjectStore.getState().clearError();
          setNewOpen(false);
        }}
        onCreate={async (request) => {
          return useProjectStore.getState().newProject(request);
        }}
      />
      <LockConflictDialog
        conflict={lockConflict}
        onOpenReadonly={(path) => {
          void useProjectStore.getState().openReadonly(path);
        }}
        onTakeOver={(path) => {
          void useProjectStore.getState().takeOverLock(path);
        }}
        onCancel={() => {
          useProjectStore.getState().dismissLockConflict();
        }}
      />
      <UnsavedChangesDialog
        isOpen={gate !== null}
        projectName={snapshot?.project.name ?? ""}
        onChoose={resolveGate}
      />
      <CommandPalette
        isOpen={paletteOpen}
        onClose={() => {
          setPaletteOpen(false);
        }}
        flags={flags}
      />
    </div>
  );
}
