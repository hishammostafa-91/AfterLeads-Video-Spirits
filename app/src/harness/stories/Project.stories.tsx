// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * Project lifecycle stories (T027): the Welcome states (empty /
 * recents / missing) and the DLG bodies (DLG-01, DLG-20, the E1007
 * lock conflict) rendered inline - fixed overlays land outside the
 * story cards, so the bodies are the same components the dialogs
 * host. The mock transport carries the fixture recents.
 */
import { useEffect } from "react";
import { Welcome } from "../../shell/Welcome";
import { AppShell } from "../../shell/AppShell";
import { NewProjectBody } from "../../dialogs/NewProject";
import { UnsavedChangesBody } from "../../dialogs/UnsavedChanges";
import { LockConflictBody } from "../../dialogs/LockConflict";
import { setIpcTransport } from "../../ipc/transport";
import { ipcMockTransport } from "../mock/ipcMock";
import { useProjectStore } from "../../projects/store";
import shared from "./stories.module.css";
import type { StoryFile } from "./types";

function WelcomeStory(
  recents: Array<{ path: string; name: string }>,
  missing: string[] = [],
): () => React.JSX.Element {
  return function WelcomeStoryCard(): React.JSX.Element {
    useEffect(() => {
      const mock = ipcMockTransport({
        recents: recents.map((entry) => ({
          path: entry.path,
          name: entry.name,
          thumbnail: null,
          openedUtc: "2026-01-01T00:00:00Z",
          pinned: false,
        })),
      });
      setIpcTransport(mock);
      useProjectStore.setState({
        recents: [],
        missingPaths: new Set(missing),
        snapshot: null,
      });
      // Welcome mounts -> refreshRecents pulls the mock's list.
    }, []);
    return (
      <div className={shared.area}>
        <Welcome
          onNewProject={() => undefined}
          onOpenProject={() => undefined}
          onOpenRecent={() => undefined}
        />
      </div>
    );
  };
}

function DlgStory(body: React.JSX.Element): () => React.JSX.Element {
  return () => <div className={shared.area}>{body}</div>;
}

export const projectStories: StoryFile = {
  id: "project",
  stories: [
    {
      id: "welcome.empty",
      render: WelcomeStory([]),
    },
    {
      id: "welcome.recents",
      render: WelcomeStory(
        [
          {
            path: "C:\\Projects\\Interviews\\Interviews.alvs",
            name: "Interviews",
          },
          { path: "C:\\Projects\\Broll\\Broll.alvs", name: "Broll" },
        ],
        [],
      ),
    },
    {
      id: "welcome.missing",
      render: WelcomeStory(
        [
          {
            path: "C:\\Projects\\Interviews\\Interviews.alvs",
            name: "Interviews",
          },
          { path: "D:\\Gone\\Deleted.alvs", name: "Deleted" },
        ],
        ["D:\\Gone\\Deleted.alvs"],
      ),
    },
    {
      id: "dlg.newProject",
      render: DlgStory(
        <NewProjectBody
          defaultFolder="C:\\Users\\User\\Videos"
          error={null}
          onCreate={() => Promise.resolve(false)}
          onDone={() => undefined}
        />,
      ),
    },
    {
      id: "dlg.unsaved",
      render: DlgStory(
        <UnsavedChangesBody
          projectName="Interviews"
          onChoose={() => undefined}
        />,
      ),
    },
    {
      id: "dlg.lock",
      render: DlgStory(
        <LockConflictBody
          takeoverAvailable
          onOpenReadonly={() => undefined}
          onTakeOver={() => undefined}
          onCancel={() => undefined}
        />,
      ),
    },
    {
      id: "project.flow",
      render: () => <ProjectFlowStory />,
      wide: true,
    },
  ],
};

/** The full lifecycle flow over the mock backend (T-APP-004
 * e2e-lite): Welcome -> DLG-01 -> the editing shell. */
function ProjectFlowStory(): React.JSX.Element {
  useEffect(() => {
    setIpcTransport(
      ipcMockTransport({
        recents: [
          {
            path: "C:\\Projects\\Interviews\\Interviews.alvs",
            name: "Interviews",
            thumbnail: null,
            openedUtc: "2026-01-01T00:00:00Z",
            pinned: false,
          },
        ],
      }),
    );
    useProjectStore.setState({
      snapshot: null,
      dirty: false,
      recents: [],
      missingPaths: new Set<string>(),
      lockConflict: null,
      lastError: null,
    });
  }, []);
  return <AppShell />;
}
