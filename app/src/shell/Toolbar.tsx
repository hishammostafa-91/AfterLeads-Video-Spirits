// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * The toolbar (REQ-APP-001; 07 §7.2 anatomy): tools V/B/T/Y/U/H/Z,
 * undo/redo, Snap and Link toggles, the workspace switcher (the
 * shell's live commands), Export…, notifications, settings, and jobs
 * access. Controls that invoke commands map to 12 §12.10 ids and
 * disable with their label + shortcut tooltip until their work
 * package ships; the bell and the jobs access are panel controls
 * from the anatomy (FR-004 AC1), disabled until their panels exist.
 */
import type { JSX, ReactNode } from "react";
import { useIntl } from "react-intl";
import {
  ArrowLeftRight,
  Bell,
  Hand,
  Link2,
  ListTodo,
  Magnet,
  MousePointer2,
  MoveDiagonal,
  MoveHorizontal,
  Redo2,
  Scissors,
  Settings2,
  Undo2,
  ZoomIn,
} from "../design/icons";
import { Button, IconButton } from "../components/Button";
import { MenuBarItem, MenuSeparator } from "../components/Menu";
import { CommandMenuItem } from "./MenuBar";
import {
  COMMAND_BY_ID,
  type CommandSpec,
  type Shortcut,
} from "../commands/catalog";
import { chordLabel } from "../commands/keyLabel";
import { commandRow, type CommandRuntimeFlags } from "../commands/registry";
import styles from "./Toolbar.module.css";

const TOOL_ICONS: Readonly<Record<string, ReactNode>> = {
  "tool.select": <MousePointer2 aria-hidden="true" />,
  "tool.blade": <Scissors aria-hidden="true" />,
  "tool.trim": <ArrowLeftRight aria-hidden="true" />,
  "tool.slip": <MoveDiagonal aria-hidden="true" />,
  "tool.slide": <MoveHorizontal aria-hidden="true" />,
  "tool.hand": <Hand aria-hidden="true" />,
  "tool.zoom": <ZoomIn aria-hidden="true" />,
};

function firstChord(spec: CommandSpec): string | undefined {
  const shortcut: Shortcut | undefined =
    spec.shortcuts.length > 0 ? spec.shortcuts[0] : undefined;
  return shortcut ? chordLabel(shortcut) : undefined;
}

function rowFor(id: string, flags: CommandRuntimeFlags): boolean {
  const spec = COMMAND_BY_ID.get(id);
  if (!spec) throw new Error(`toolbar: unknown command "${id}"`);
  return commandRow(spec, flags).enabled;
}

/** An icon button bound to a command; disabled unless enabled. */
function CommandIconButton({
  id,
  icon,
  flags,
}: {
  id: string;
  icon: ReactNode;
  flags: CommandRuntimeFlags;
}): JSX.Element {
  const spec = COMMAND_BY_ID.get(id);
  if (!spec) throw new Error(`toolbar: unknown command "${id}"`);
  return (
    <IconButton
      icon={icon}
      labelMessageId={spec.labelKey}
      shortcut={firstChord(spec)}
      isDisabled={!rowFor(id, flags)}
    />
  );
}

export type WorkspaceId = "WS-01" | "WS-02" | "WS-03" | "WS-04";

const WORKSPACE_LABEL_KEY: Readonly<Record<WorkspaceId, string>> = {
  "WS-01": "kb.view.workspaceEdit",
  "WS-02": "kb.view.workspaceColor",
  "WS-03": "kb.view.workspaceAudio",
  "WS-04": "kb.view.workspaceCaptions",
};

const WORKSPACE_COMMAND_ID: Readonly<Record<WorkspaceId, string>> = {
  "WS-01": "view.workspaceEdit",
  "WS-02": "view.workspaceColor",
  "WS-03": "view.workspaceAudio",
  "WS-04": "view.workspaceCaptions",
};

const WORKSPACES: readonly WorkspaceId[] = ["WS-01", "WS-02", "WS-03", "WS-04"];

export interface ToolbarProps {
  flags: CommandRuntimeFlags;
  /** Active workspace id (the switcher's label). */
  activeWorkspace: WorkspaceId;
}

export function Toolbar({ flags, activeWorkspace }: ToolbarProps): JSX.Element {
  const { formatMessage } = useIntl();
  return (
    <div
      className={styles.toolbar}
      role="toolbar"
      aria-label={formatMessage({ id: "shell.toolbar.label" })}
    >
      {Object.keys(TOOL_ICONS).map((id) => (
        <CommandIconButton
          key={id}
          id={id}
          icon={TOOL_ICONS[id]}
          flags={flags}
        />
      ))}
      <span className={styles.divider} aria-hidden="true" />
      <CommandIconButton
        id="edit.undo"
        icon={<Undo2 aria-hidden="true" />}
        flags={flags}
      />
      <CommandIconButton
        id="edit.redo"
        icon={<Redo2 aria-hidden="true" />}
        flags={flags}
      />
      <span className={styles.divider} aria-hidden="true" />
      <CommandIconButton
        id="timeline.toggleSnapping"
        icon={<Magnet aria-hidden="true" />}
        flags={flags}
      />
      <CommandIconButton
        id="timeline.toggleLinkedSelection"
        icon={<Link2 aria-hidden="true" />}
        flags={flags}
      />
      <span className={styles.divider} aria-hidden="true" />
      <MenuBarItem
        standalone
        label={`${formatMessage({ id: "shell.toolbar.workspace" })}: ${formatMessage({ id: WORKSPACE_LABEL_KEY[activeWorkspace] })}`}
      >
        {WORKSPACES.map((ws) => (
          <CommandMenuItem
            key={ws}
            id={WORKSPACE_COMMAND_ID[ws]}
            flags={flags}
          />
        ))}
        <MenuSeparator />
        <CommandMenuItem id="view.resetWorkspace" flags={flags} />
      </MenuBarItem>
      <span className={styles.spacer} aria-hidden="true" />
      <Button variant="secondary" isDisabled={!rowFor("file.export", flags)}>
        {formatMessage({ id: "kb.file.export" })}
      </Button>
      {/* Panel controls from the 07 s7.2 anatomy (FR-004 AC1). */}
      <IconButton
        icon={<Bell aria-hidden="true" />}
        labelMessageId="shell.toolbar.notifications"
        isDisabled
      />
      <IconButton
        icon={<Settings2 aria-hidden="true" />}
        labelMessageId="kb.file.settings"
        isDisabled={!rowFor("file.settings", flags)}
      />
      <IconButton
        icon={<ListTodo aria-hidden="true" />}
        labelMessageId="shell.toolbar.jobs"
        isDisabled
      />
    </div>
  );
}
