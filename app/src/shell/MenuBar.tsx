// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * The application menu bar (REQ-APP-001; 12 §12.11): the seven menus
 * rendered from the §12.11 data in `menus.ts` — labels, shortcuts,
 * enablement, and checked marks come from the command registry, and
 * commands unimplemented in any shipped work package render disabled
 * with the localized later-version note (WP-05 card).
 */
import { Fragment, type JSX } from "react";
import { useIntl } from "react-intl";
import {
  Menu,
  MenuBar as ShellMenuBar,
  MenuBarItem,
  MenuItem,
  MenuItemCheckbox,
  MenuSeparator,
} from "../components/Menu";
import { COMMAND_BY_ID, type Shortcut } from "../commands/catalog";
import { chordLabel } from "../commands/keyLabel";
import { commandRow, type CommandRuntimeFlags } from "../commands/registry";
import { MENUS, type MenuId } from "./menus";
import menuStyles from "./MenuBar.module.css";

/** Literal label keys in the scanner's `id: "…"` shape (12 §12.11). */
const MENU_LABEL_KEYS: ReadonlyArray<{ menu: MenuId; id: string }> = [
  { menu: "File", id: "shell.menu.File" },
  { menu: "Edit", id: "shell.menu.Edit" },
  { menu: "Clip", id: "shell.menu.Clip" },
  { menu: "Timeline", id: "shell.menu.Timeline" },
  { menu: "Playback", id: "shell.menu.Playback" },
  { menu: "View", id: "shell.menu.View" },
  { menu: "Help", id: "shell.menu.Help" },
];

function labelKeyFor(menu: MenuId): string {
  const entry = MENU_LABEL_KEYS.find((candidate) => candidate.menu === menu);
  if (!entry) throw new Error(`menu bar: no label key for ${menu}`);
  return entry.id;
}

function CommandMenuItem({
  id,
  flags,
}: {
  id: string;
  flags: CommandRuntimeFlags;
}): JSX.Element {
  const { formatMessage } = useIntl();
  const spec = COMMAND_BY_ID.get(id);
  if (!spec) throw new Error(`menu bar: unknown command "${id}"`);
  const row = commandRow(spec, flags);
  const shortcut: Shortcut | undefined =
    spec.shortcuts.length > 0 ? spec.shortcuts[0] : undefined;
  const label = formatMessage({ id: spec.labelKey });
  const laterNote = row.laterVersion
    ? formatMessage({ id: "commands.laterVersion" })
    : undefined;
  if (spec.checkable) {
    return (
      <MenuItemCheckbox
        key={spec.id}
        isChecked={row.checked}
        isDisabled={!row.enabled}
      >
        {label}
      </MenuItemCheckbox>
    );
  }
  return (
    <MenuItem
      key={spec.id}
      isDisabled={!row.enabled}
      shortcut={shortcut ? chordLabel(shortcut) : undefined}
    >
      {label}
      {laterNote ? (
        <span className={menuStyles["later-note"]}> — {laterNote}</span>
      ) : null}
    </MenuItem>
  );
}

export interface AppMenuBarProps {
  flags: CommandRuntimeFlags;
}

/** The seven-menu bar (labels via the literal MENU_LABEL_KEY map). */
export function AppMenuBar({ flags }: AppMenuBarProps): JSX.Element {
  const { formatMessage } = useIntl();
  return (
    <ShellMenuBar labelMessageId="shell.menuBar.label">
      {MENUS.map((menu) => {
        const label = formatMessage({ id: labelKeyFor(menu.id) });
        return (
          <MenuBarItem key={menu.id} label={label}>
            <Menu>
              {menu.groups.map((group, groupIndex) => (
                <Fragment key={`${menu.id}-${String(groupIndex)}`}>
                  {groupIndex > 0 ? <MenuSeparator /> : null}
                  {group.items.map((id) => (
                    <CommandMenuItem key={id} id={id} flags={flags} />
                  ))}
                </Fragment>
              ))}
            </Menu>
          </MenuBarItem>
        );
      })}
    </ShellMenuBar>
  );
}
