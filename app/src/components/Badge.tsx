// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * Badge (07 §7.6): text + icon state chips — Proxy, Offline, 10-bit,
 * HDR, VFR. State is never carried by color alone: every chip pairs
 * its icon with its text (REQ-A11Y-003; 07 §7.6).
 */
import type { JSX, ReactNode } from "react";
import { Binary, CloudOff, MonitorPlay, Shuffle, Waves } from "../design/icons";
import { FormattedMessage } from "react-intl";
import styles from "./Badge.module.css";

export type BadgeKind = "proxy" | "offline" | "tenBit" | "hdr" | "vfr";

const BADGE_DEFS: Record<
  BadgeKind,
  { messageId: string; icon: (size: number) => ReactNode }
> = {
  proxy: {
    messageId: "components.badge.proxy",
    icon: (size) => <Shuffle size={size} aria-hidden="true" />,
  },
  offline: {
    messageId: "components.badge.offline",
    icon: (size) => <CloudOff size={size} aria-hidden="true" />,
  },
  tenBit: {
    messageId: "components.badge.tenBit",
    icon: (size) => <Binary size={size} aria-hidden="true" />,
  },
  hdr: {
    messageId: "components.badge.hdr",
    icon: (size) => <MonitorPlay size={size} aria-hidden="true" />,
  },
  vfr: {
    messageId: "components.badge.vfr",
    icon: (size) => <Waves size={size} aria-hidden="true" />,
  },
};

export interface BadgeProps {
  kind: BadgeKind;
  className?: string;
}

/** A media state chip: icon + text (never color alone). */
export function Badge({ kind, className }: BadgeProps): JSX.Element {
  const classes = [styles.badge, className].filter(Boolean).join(" ");
  const { messageId, icon } = BADGE_DEFS[kind];
  return (
    <span className={classes} data-kind={kind}>
      {icon(12)}
      <span className={styles.text}>
        <FormattedMessage id={messageId} />
      </span>
    </span>
  );
}
