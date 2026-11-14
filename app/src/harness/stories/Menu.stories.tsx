// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * Menu family stories (07 §7.6; T018).
 */
import { FormattedMessage } from "react-intl";
import { useState } from "react";
import type { JSX } from "react";
import { MenuTrigger } from "react-aria-components";
import { Button } from "../../components/Button";
import {
  ContextMenu,
  Menu,
  MenuBar,
  MenuBarItem,
  MenuItem,
  MenuItemCheckbox,
  Submenu,
} from "../../components/Menu";
import styles from "./stories.module.css";
import type { StoryFile } from "./types";

function MenuDemo(): JSX.Element {
  return (
    <MenuTrigger>
      <Button>
        <FormattedMessage id="components.menu.edit" />
      </Button>
      <Menu>
        <MenuItem id="undo" shortcut="Ctrl+Z">
          <FormattedMessage id="components.menu.undo" />
        </MenuItem>
        <MenuItem id="redo" shortcut="Ctrl+Y">
          <FormattedMessage id="components.menu.redo" />
        </MenuItem>
        <Submenu label="New">
          <MenuItem id="project">
            <FormattedMessage id="components.menu.newProject" />
          </MenuItem>
        </Submenu>
      </Menu>
    </MenuTrigger>
  );
}

function MenuBarDemo(): JSX.Element {
  return (
    <MenuBar labelMessageId="components.menu.barLabel">
      <MenuBarItem label={<FormattedMessage id="components.menu.file" />}>
        <MenuItem id="new" shortcut="Ctrl+N">
          <FormattedMessage id="components.menu.newProject" />
        </MenuItem>
      </MenuBarItem>
      <MenuBarItem label={<FormattedMessage id="components.menu.edit" />}>
        <MenuItem id="undo" shortcut="Ctrl+Z">
          <FormattedMessage id="components.menu.undo" />
        </MenuItem>
      </MenuBarItem>
    </MenuBar>
  );
}

function ContextMenuDemo(): JSX.Element {
  return (
    <ContextMenu
      items={
        <MenuItem id="rename">
          <FormattedMessage id="components.menu.rename" />
        </MenuItem>
      }
    >
      <div className={styles.area}>
        <FormattedMessage id="components.menu.contextArea" />
      </div>
    </ContextMenu>
  );
}

/** FR-007 disabled + error states and the §7.6 checkable items. */
function MenuStates(): JSX.Element {
  const [waveform, setWaveform] = useState(true);
  const [safeAreas, setSafeAreas] = useState(false);
  return (
    <MenuTrigger>
      <Button>
        <FormattedMessage id="components.menu.edit" />
      </Button>
      <Menu>
        <MenuItemCheckbox
          id="waveform"
          isChecked={waveform}
          onAction={() => {
            setWaveform((prev) => !prev);
          }}
        >
          <FormattedMessage id="components.menu.waveform" />
        </MenuItemCheckbox>
        <MenuItemCheckbox
          id="safeAreas"
          isChecked={safeAreas}
          onAction={() => {
            setSafeAreas((prev) => !prev);
          }}
        >
          <FormattedMessage id="components.menu.safeAreas" />
        </MenuItemCheckbox>
        <MenuItem id="undo" isDisabled shortcut="Ctrl+Z">
          <FormattedMessage id="components.menu.undo" />
        </MenuItem>
      </Menu>
    </MenuTrigger>
  );
}

export const menuStories: StoryFile = {
  id: "menu",
  stories: [
    { id: "menu.dropdown", render: () => <MenuDemo /> },
    { id: "menu.menubar", render: () => <MenuBarDemo /> },
    { id: "menu.context", render: () => <ContextMenuDemo /> },
    { id: "menu.states", render: () => <MenuStates /> },
  ],
};
