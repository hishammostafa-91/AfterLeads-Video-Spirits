// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * Shell stories: the chrome surfaces (title bar with the menu bar,
 * toolbar, status bar) in their neutral state and with live-looking
 * values (stories are fixtures — product code never fakes data).
 */
import { useEffect } from "react";
import { StatusBar } from "../../shell/StatusBar";
import { TitleBar } from "../../shell/TitleBar";
import { Toolbar } from "../../shell/Toolbar";
import type { CommandRuntimeFlags } from "../../commands/registry";
import { useStatusStore, resetStatusStoreForTests } from "../../shell/status";
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
  useEffect(() => {
    resetStatusStoreForTests();
  }, []);
  return <StatusBar />;
}

function StatusBarLive(): React.JSX.Element {
  return (
    <StatusLive
      jobs={{ running: 2 }}
      autosave={{ state: "saved", timeUtc: "2026-01-01T12:30:00Z" }}
      renderer="NVIDIA GeForce RTX 3060"
    />
  );
}

/** Story-only seam: seed the status store for the live card. */
function StatusLive(props: {
  jobs: { running: number };
  autosave: {
    state: "idle" | "saving" | "saved" | "failed";
    timeUtc: string | null;
  };
  renderer: string;
}): React.JSX.Element {
  useEffect(() => {
    useStatusStore.getState().setJobs(props.jobs);
    useStatusStore.getState().setAutosave(props.autosave);
    useStatusStore.setState({ renderer: props.renderer });
  }, [props]);
  return <StatusBar />;
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
