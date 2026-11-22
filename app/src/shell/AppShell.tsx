// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * The application shell frame (07 s7.2): title bar, toolbar, the dock
 * region, and the status bar, plus the keyboard dispatcher and the
 * command palette. The workspace persistence seam must be bound
 * before mount (the app binds the settings IPC port; the harness
 * binds its localStorage port).
 */
import { useEffect, useState } from "react";
import type { JSX } from "react";
import { TitleBar } from "./TitleBar";
import { Toolbar } from "./Toolbar";
import { StatusBar } from "./StatusBar";
import { Dock } from "./dock/Dock";
import { CommandPalette } from "../commands/palette/CommandPalette";
import { installDispatcher } from "../commands/dispatcher";
import {
  INITIAL_RUNTIME_FLAGS,
  registerHandler,
  unregisterHandler,
  type CommandRuntimeFlags,
} from "../commands/registry";
import { useWorkspaceStore } from "./dock/workspaces";
import styles from "./AppShell.module.css";

export function AppShell(): JSX.Element {
  const active = useWorkspaceStore((state) => state.active);
  const [paletteOpen, setPaletteOpen] = useState(false);
  const [flags] = useState<CommandRuntimeFlags>(INITIAL_RUNTIME_FLAGS);

  useEffect(() => {
    const teardownDispatcher = installDispatcher(window, () => flags);
    registerHandler("view.commandPalette", () => {
      setPaletteOpen((open) => !open);
    });
    return () => {
      unregisterHandler("view.commandPalette");
      teardownDispatcher();
    };
  }, [flags]);

  return (
    <div className={styles.shell}>
      <TitleBar flags={flags} />
      <Toolbar flags={flags} activeWorkspace={active} />
      <Dock />
      <StatusBar />
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
