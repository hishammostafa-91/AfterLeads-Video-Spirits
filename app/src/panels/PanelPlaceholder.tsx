// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * A later-WP panel's placeholder body (spec D3): labeled, accessible,
 * and empty - the panel's icon and title, and the owning work package
 * that ships its real content. No fake data, no dead controls.
 */
import { useIntl } from "react-intl";
import type { JSX } from "react";
import { PANEL_BY_ID, type PanelSpec } from "./registry";
import styles from "./PanelPlaceholder.module.css";

export function PanelPlaceholder({ id }: { id: string }): JSX.Element {
  const panel: PanelSpec | undefined = PANEL_BY_ID.get(id);
  if (!panel) throw new Error(`panel placeholder: unknown panel "${id}"`);
  const { formatMessage } = useIntl();
  return (
    <div className={styles.placeholder}>
      <span className={styles.icon}>{panel.icon}</span>
      <p className={styles.title}>{formatMessage({ id: panel.titleKey })}</p>
      <p className={styles.note}>
        {formatMessage(
          { id: "panels.placeholder.body" },
          { wp: panel.owningWp },
        )}
      </p>
    </div>
  );
}
