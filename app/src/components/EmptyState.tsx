// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * EmptyState (07 §7.6): icon, one sentence, a primary action, and a
 * shortcut hint.
 */
import type { JSX, ReactNode } from "react";
import {
  ICON_SIZE_TOOLBAR,
  ICON_STROKE_WIDTH,
  PackageOpen,
} from "../design/icons";
import { FormattedMessage } from "react-intl";
import { Button } from "./Button";
import styles from "./EmptyState.module.css";

export interface EmptyStateProps {
  /** Icon node (Lucide 16–24px). */
  icon?: ReactNode;
  /** One sentence (already translated). */
  message: ReactNode;
  /** Message ID of the primary action's label. */
  actionMessageId: string;
  onAction: () => void;
  /** Keyboard shortcut glyph for the action (KB catalog). */
  shortcut?: string;
  className?: string;
}

export function EmptyState({
  icon,
  message,
  actionMessageId,
  onAction,
  shortcut,
  className,
}: EmptyStateProps): JSX.Element {
  const classes = [styles.empty, className].filter(Boolean).join(" ");
  return (
    <div className={classes}>
      <span className={styles.icon}>
        {icon ?? (
          <PackageOpen
            size={ICON_SIZE_TOOLBAR}
            strokeWidth={ICON_STROKE_WIDTH}
            aria-hidden="true"
          />
        )}
      </span>
      <p className={styles.sentence}>{message}</p>
      <Button variant="primary" onPress={onAction}>
        <FormattedMessage id={actionMessageId} />
      </Button>
      {shortcut && (
        <span className={styles.shortcutHint}>
          <FormattedMessage id="components.empty.shortcutPrefix" />
          <kbd className={styles.kbd} aria-hidden="true">
            {shortcut}
          </kbd>
        </span>
      )}
    </div>
  );
}
