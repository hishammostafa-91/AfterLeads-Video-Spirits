// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * The custom title bar (REQ-APP-001; 07 §7.2): 32 px, hosts the menu
 * bar and the project title with the dirty marker (REQ-APP-012), and
 * carries data-tauri-drag-region so the window drags from the bar
 * once native decorations are replaced by the in-app caption area
 * (T010; the system caption buttons area keeps Snap Layouts).
 */
import { useIntl } from "react-intl";
import type { JSX } from "react";
import { AppMenuBar } from "./MenuBar";
import type { CommandRuntimeFlags } from "../commands/registry";
import styles from "./TitleBar.module.css";

export interface TitleBarProps {
  flags: CommandRuntimeFlags;
  /** Open project name; omitted renders the neutral title. */
  projectName?: string;
  /** REQ-APP-012 dirty marker. */
  isDirty?: boolean;
}

export function TitleBar({
  flags,
  projectName,
  isDirty = false,
}: TitleBarProps): JSX.Element {
  const { formatMessage } = useIntl();
  return (
    <header className={styles.titleBar} data-tauri-drag-region>
      {/* Original mark: a rotated accent square (no third-party art). */}
      <span className={styles.logo} aria-hidden="true" />
      <AppMenuBar flags={flags} />
      <p className={styles.project} data-tauri-drag-region>
        {projectName ? (
          <>
            {projectName}
            {isDirty ? (
              <span
                className={styles.dirtyDot}
                role="img"
                aria-label={formatMessage({ id: "shell.titleBar.dirty" })}
              >
                •
              </span>
            ) : null}
          </>
        ) : (
          formatMessage({ id: "shell.titleBar.neutral" })
        )}
      </p>
    </header>
  );
}
