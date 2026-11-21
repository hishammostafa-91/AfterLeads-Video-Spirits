// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * Shell stories: the chrome surfaces (title bar with the menu bar,
 * toolbar, status bar) in their neutral state and with live-looking
 * values (stories are fixtures — product code never fakes data).
 */
import { StatusBar } from "../../shell/StatusBar";
import { TitleBar } from "../../shell/TitleBar";
import { Toolbar } from "../../shell/Toolbar";
import type { CommandRuntimeFlags } from "../../commands/registry";
import type { StoryFile } from "./types";

const flags: CommandRuntimeFlags = {
  focusedContext: "global",
  projectOpen: true,
};

function TitleBarDemo(): React.JSX.Element {
  return <TitleBar flags={flags} projectName="Interviews" isDirty />;
}

function ToolbarDemo(): React.JSX.Element {
  return <Toolbar flags={flags} activeWorkspace="WS-01" />;
}

function StatusBarNeutral(): React.JSX.Element {
  return <StatusBar />;
}

function StatusBarLive(): React.JSX.Element {
  return (
    <StatusBar
      jobsSummary="2 jobs running"
      autosaveState="saved"
      lastSavedAt="12:30"
      fps={30}
      droppedFrames={0}
      proxyMode
      rendererName="NVIDIA GeForce RTX 3060"
    />
  );
}

export const shellStories: StoryFile = {
  id: "shell",
  stories: [
    { id: "shell.title-bar", render: () => <TitleBarDemo /> },
    { id: "shell.toolbar", render: () => <ToolbarDemo /> },
    { id: "shell.status-bar", render: () => <StatusBarNeutral /> },
    { id: "shell.status-bar-live", render: () => <StatusBarLive /> },
  ],
};
