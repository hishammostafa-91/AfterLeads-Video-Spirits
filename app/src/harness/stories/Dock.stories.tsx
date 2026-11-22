// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * Dock stories (FR-006/FR-007): the workspace-driven dock surface -
 * toolbar (workspace switcher), the dockview region, status bar -
 * with the keyboard dispatcher installed, so the Playwright flows
 * drive docking, workspace switching (Alt+1..4, KB-125..128), panel
 * focus (Shift+1..9, KB-135..143), and restart round-trips
 * (T-APP-002, T-A11Y-001) exactly like the product shell.
 */
import { useEffect, useState } from "react";
import { Dock } from "../../shell/dock/Dock";
import { StatusBar } from "../../shell/StatusBar";
import { TitleBar } from "../../shell/TitleBar";
import { Toolbar } from "../../shell/Toolbar";
import { installDispatcher } from "../../commands/dispatcher";
import {
  INITIAL_RUNTIME_FLAGS,
  type CommandRuntimeFlags,
} from "../../commands/registry";
import { useWorkspaceStore } from "../../shell/dock/workspaces";
import styles from "./Dock.stories.module.css";
import type { StoryFile } from "./types";

function DockFrame(): React.JSX.Element {
  const active = useWorkspaceStore((state) => state.active);
  const [flags] = useState<CommandRuntimeFlags>(INITIAL_RUNTIME_FLAGS);
  useEffect(() => installDispatcher(window, () => flags), [flags]);
  return (
    <div className={styles.frame} data-active-workspace={active}>
      <TitleBar flags={flags} />
      <Toolbar flags={flags} activeWorkspace={active} />
      <Dock />
      <StatusBar />
    </div>
  );
}

export const dockStories: StoryFile = {
  id: "dock",
  stories: [
    {
      id: "shell.dock",
      render: () => <DockFrame />,
      wide: true,
    },
  ],
};
