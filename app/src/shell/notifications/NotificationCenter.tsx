// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * The notification center (FR-014 AC2): the history list as a dock
 * panel body - error rows carry the catalog code chip and their
 * recovery actions as buttons, info rows render the message.
 */
import { useIntl } from "react-intl";
import type { JSX } from "react";
import { Button } from "../../components/Button";
import type { CommandRuntimeFlags } from "../../commands/registry";
import { useNotificationStore } from "./store";
import styles from "./NotificationCenter.module.css";

export interface NotificationCenterProps {
  flags: CommandRuntimeFlags;
}

export function NotificationCenter({
  flags,
}: NotificationCenterProps): JSX.Element {
  const { formatMessage } = useIntl();
  const entries = useNotificationStore((state) => state.entries);
  const runRecovery = useNotificationStore((state) => state.runRecovery);
  const clear = useNotificationStore((state) => state.clear);
  if (entries.length === 0) {
    return (
      <p className={styles.empty}>
        {formatMessage({ id: "notifications.empty" })}
      </p>
    );
  }
  return (
    <div className={styles.center}>
      <div className={styles.header}>
        <span className={styles.count}>
          {formatMessage(
            { id: "notifications.count" },
            { count: entries.length },
          )}
        </span>
        <Button
          variant="ghost"
          onPress={() => {
            clear();
          }}
        >
          {formatMessage({ id: "notifications.clear" })}
        </Button>
      </div>
      <ul className={styles.list}>
        {entries.map((entry) => (
          <li key={entry.id} className={styles.row} data-kind={entry.kind}>
            {entry.code ? (
              <span className={styles.code}>{entry.code}</span>
            ) : null}
            <span className={styles.message}>
              {formatMessage({ id: entry.messageKey }, entry.params)}
            </span>
            {entry.recovery.length > 0 ? (
              <span className={styles.actions}>
                {entry.recovery.map((action) => (
                  <Button
                    key={action.commandId}
                    variant="secondary"
                    onPress={() => {
                      void runRecovery(entry, action, flags);
                    }}
                  >
                    {formatMessage({ id: action.labelKey })}
                  </Button>
                ))}
              </span>
            ) : null}
          </li>
        ))}
      </ul>
    </div>
  );
}
