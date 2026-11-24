// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * The dock region (FR-006/FR-007): a dockview-react host bound to
 * the workspace service. All layout mutations flow through the
 * controller here - workspace switch/reset, panel focus
 * (KB-135..143), panel cycle (F6/Shift+F6), maximize (`) - while
 * dockview owns the gestures (dock, tab, resize, float). Layout
 * edits persist debounced (03 s3.6: 500 ms) into the active
 * workspace's override; dockview's own live-region strings are
 * localized through its `messages` catalog.
 */
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useIntl } from "react-intl";
import type { JSX } from "react";
import {
  DockviewReact,
  type DockviewApi,
  type DockviewMessages,
  type DockviewReadyEvent,
  type IDockviewPanelProps,
} from "dockview-react";
import "dockview/dist/styles/dockview.css";
import "./dockview-theme.css";
import { PanelPlaceholder } from "../../panels/PanelPlaceholder";
import { PANEL_BY_FOCUS_COMMAND, PANEL_BY_ID } from "../../panels/registry";
import { useShellPanelsStore } from "../notifications/shellPanels";
import { NotificationCenter } from "../notifications/NotificationCenter";
import { announceStatus } from "../../components/a11y";
import { useProjectStore } from "../../projects/store";
import { registerHandler, unregisterHandler } from "../../commands/registry";
import {
  DEFAULT_LAYOUTS,
  captureAndPersist,
  materializeLayout,
  persistActive,
  useWorkspaceStore,
  type WorkspaceId,
} from "./workspaces";
import { PanelTab } from "./PanelTab";
import styles from "./Dock.module.css";

const DOCKVIEW_THEME = {
  name: "alvs",
  className: "dockview-theme-alvs",
} as const;

/** A panel body: the placeholder surface until its WP ships. */
function PanelBody({ api }: IDockviewPanelProps): JSX.Element {
  return <PanelPlaceholder id={api.id} />;
}

/** The notification center body (a shell-owned panel, FR-014). */
function NotificationsBody(): JSX.Element {
  const projectOpen = useProjectStore((s) => s.snapshot !== null);
  return (
    <NotificationCenter flags={{ focusedContext: "global", projectOpen }} />
  );
}

const PERSIST_DEBOUNCE_MS = 500;

/**
 * Move DOM focus to a panel's tab host (dockview marks it
 * `data-tab-panel-id`; the host is the focusable `role=tab`). Used by
 * the focus commands and the F6 cycle so keyboard users land on the
 * tab (APG deletable tabs: Delete closes the focused tab).
 */
function focusTabHost(panelId: string): void {
  const host = document.querySelector<HTMLElement>(
    `[data-tab-panel-id="${CSS.escape(panelId)}"]`,
  );
  host?.focus();
}

export function Dock(): JSX.Element {
  const { formatMessage } = useIntl();
  const apiRef = useRef<DockviewApi | null>(null);
  const suppressNextChange = useRef(false);
  const persistTimer = useRef<number | null>(null);
  const active = useWorkspaceStore((state) => state.active);
  const [maximized, setMaximized] = useState(false);

  /** Localize dockview's assistive strings (G-13). */
  const messages = useMemo<Partial<DockviewMessages>>(
    () => ({
      closeTab: (title) =>
        formatMessage({ id: "shell.dock.closeTab" }, { title }),
      closeTabPlain: () => formatMessage({ id: "shell.dock.closeTabPlain" }),
      panelOpened: (title) =>
        formatMessage({ id: "shell.dock.panelOpened" }, { title }),
      panelClosed: (title) =>
        formatMessage({ id: "shell.dock.panelClosed" }, { title }),
      groupMaximized: (title) =>
        formatMessage({ id: "shell.dock.groupMaximized" }, { title }),
      groupRestored: (title) =>
        formatMessage({ id: "shell.dock.groupRestored" }, { title }),
      groupFloated: (title) =>
        formatMessage({ id: "shell.dock.groupFloated" }, { title }),
      groupDocked: (title) =>
        formatMessage({ id: "shell.dock.groupDocked" }, { title }),
      groupPoppedOut: (title) =>
        formatMessage({ id: "shell.dock.groupPoppedOut" }, { title }),
    }),
    [formatMessage],
  );

  /** Apply the stored override or the default layout of `id`. */
  const applyWorkspace = useCallback((id: WorkspaceId) => {
    const api = apiRef.current;
    if (!api) return;
    suppressNextChange.current = true;
    const override = useWorkspaceStore.getState().overrides[id];
    if (override) {
      api.clear();
      api.fromJSON(override);
    } else {
      materializeLayout(api, DEFAULT_LAYOUTS[id]);
    }
  }, []);

  /** Switch workspace: capture the old layout, apply, persist (FR-007). */
  const switchWorkspace = useCallback(
    (id: WorkspaceId) => {
      const api = apiRef.current;
      const current = useWorkspaceStore.getState().active;
      if (!api || id === current) return;
      void captureAndPersist(current, api.toJSON());
      useWorkspaceStore.getState().setActive(id);
      applyWorkspace(id);
      void persistActive();
    },
    [applyWorkspace],
  );

  /** Reset: drop the override, restore the 07 s7.3 table (KB-130). */
  const resetWorkspace = useCallback(() => {
    const api = apiRef.current;
    const current = useWorkspaceStore.getState().active;
    if (!api) return;
    useWorkspaceStore.getState().resetActive();
    applyWorkspace(current);
    void persistActive();
  }, [applyWorkspace]);

  /** Focus a panel, opening it first when closed (View > Panels).
   * DOM focus moves to its tab (APG: Delete on the focused tab). */
  const focusPanel = useCallback((panelId: string) => {
    const api = apiRef.current;
    if (!api) return;
    const existing = api.getPanel(panelId);
    if (existing) {
      existing.api.setActive();
      focusTabHost(panelId);
      return;
    }
    const group =
      api.activeGroup ?? (api.groups.length > 0 ? api.groups[0] : undefined);
    if (!group) return;
    api.addPanel({
      id: panelId,
      title: panelId,
      component: "panel",
      position: { referenceGroup: group.id },
    });
    focusTabHost(panelId);
  }, []);

  const onReady = useCallback(
    (event: DockviewReadyEvent) => {
      apiRef.current = event.api;
      applyWorkspace(useWorkspaceStore.getState().active);
      // Maximized state hook (tests + status wiring read the attr).
      setMaximized(event.api.hasMaximizedGroup());
      event.api.onDidMaximizedGroupChange((change) => {
        setMaximized(change.isMaximized);
      });
      // Persist user layout edits per workspace, debounced (03 s3.6).
      event.api.onDidLayoutChange(() => {
        if (suppressNextChange.current) {
          suppressNextChange.current = false;
          return;
        }
        if (persistTimer.current !== null) {
          window.clearTimeout(persistTimer.current);
        }
        persistTimer.current = window.setTimeout(() => {
          persistTimer.current = null;
          const api = apiRef.current;
          const id = useWorkspaceStore.getState().active;
          if (api) void captureAndPersist(id, api.toJSON());
        }, PERSIST_DEBOUNCE_MS);
      });
      // Announce focus moves (FR-015 AC2).
      event.api.onDidActivePanelChange((event) => {
        const panel = event.panel;
        if (!panel) return;
        const spec = PANEL_BY_ID.get(panel.id);
        if (!spec) return;
        announceStatus(
          formatMessage(
            { id: "shell.dock.announceFocus" },
            { name: formatMessage({ id: spec.titleKey }) },
          ),
        );
      });
    },
    [applyWorkspace, formatMessage],
  );

  // The command handlers (registry-owned ids; FR-015/FR-007).
  useEffect(() => {
    const handlers: Array<[string, () => void]> = [
      [
        "view.workspaceEdit",
        () => {
          switchWorkspace("WS-01");
        },
      ],
      [
        "view.workspaceColor",
        () => {
          switchWorkspace("WS-02");
        },
      ],
      [
        "view.workspaceAudio",
        () => {
          switchWorkspace("WS-03");
        },
      ],
      [
        "view.workspaceCaptions",
        () => {
          switchWorkspace("WS-04");
        },
      ],
      [
        "view.resetWorkspace",
        () => {
          resetWorkspace();
        },
      ],
      [
        "view.nextPanel",
        () => {
          apiRef.current?.activateNext({ includePanel: true });
          const panel = apiRef.current?.activePanel;
          if (panel) focusTabHost(panel.id);
        },
      ],
      [
        "view.prevPanel",
        () => {
          apiRef.current?.activatePrevious({ includePanel: true });
          const panel = apiRef.current?.activePanel;
          if (panel) focusTabHost(panel.id);
        },
      ],
      [
        "view.maximizePanel",
        () => {
          const panel = apiRef.current?.activePanel;
          if (!panel) return;
          if (panel.api.isMaximized()) {
            panel.api.exitMaximized();
          } else {
            panel.api.maximize();
          }
        },
      ],
      ...[...PANEL_BY_FOCUS_COMMAND.entries()].map(
        ([commandId, panel]) =>
          [
            commandId,
            () => {
              focusPanel(panel.id);
            },
          ] as [string, () => void],
      ),
    ];
    for (const [id, handler] of handlers) registerHandler(id, handler);
    return () => {
      for (const [id] of handlers) unregisterHandler(id);
    };
  }, [switchWorkspace, resetWorkspace, focusPanel]);

  // Announce workspace switches (the store drives the toolbar label);
  // the boot application is not a switch, so it stays silent.
  const announcedOnce = useRef(false);
  useEffect(() => {
    if (!announcedOnce.current) {
      announcedOnce.current = true;
      return;
    }
    announceStatus(
      formatMessage(
        { id: "shell.dock.announceWorkspace" },
        {
          name: formatMessage({
            id:
              active === "WS-01"
                ? "kb.view.workspaceEdit"
                : active === "WS-02"
                  ? "kb.view.workspaceColor"
                  : active === "WS-03"
                    ? "kb.view.workspaceAudio"
                    : "kb.view.workspaceCaptions",
          }),
        },
      ),
    );
  }, [active, formatMessage]);

  // Shell-owned panels: the bell's request opens (or focuses) the
  // notification center panel in the dock (FR-004/FR-014).
  const shellPanelRequests = useShellPanelsStore((state) => state.requestSeq);
  useEffect(() => {
    if (shellPanelRequests === 0) return;
    const api = apiRef.current;
    if (!api) return;
    const existing = api.getPanel("notifications");
    if (existing) {
      existing.api.setActive();
      focusTabHost("notifications");
      return;
    }
    const group =
      api.activeGroup ?? (api.groups.length > 0 ? api.groups[0] : undefined);
    if (!group) return;
    api.addPanel({
      id: "notifications",
      title: "notifications",
      component: "notifications",
      position: { referenceGroup: group.id },
    });
    focusTabHost("notifications");
  }, [shellPanelRequests]);

  return (
    <main
      className={styles.dock}
      aria-label={formatMessage({ id: "shell.dock.region" })}
      data-active-workspace={active}
      data-maximized={maximized ? "true" : "false"}
    >
      <DockviewReact
        onReady={onReady}
        components={{ panel: PanelBody, notifications: NotificationsBody }}
        defaultTabComponent={PanelTab}
        theme={DOCKVIEW_THEME}
        messages={messages}
      />
    </main>
  );
}
