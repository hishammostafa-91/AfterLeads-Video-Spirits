// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * Tooltip (07 §7.6): a styled RAC Tooltip with the 500 ms open delay
 * (07 §7.6; REQ-HLP-003). Compose inside RAC TooltipTrigger; see
 * IconButton for the icon+label+shortcut usage.
 */
import type { JSX, ReactNode } from "react";
import {
  Tooltip as RACTooltip,
  type TooltipProps,
} from "react-aria-components";
import styles from "./Tooltip.module.css";

/** The canonical tooltip open delay (07 §7.6). */
export const TOOLTIP_DELAY_MS = 500;

export interface TooltipWrapperProps extends Omit<TooltipProps, "className"> {
  className?: string;
  children?: ReactNode;
}

/** A styled tooltip (render inside a TooltipTrigger). */
export function Tooltip({
  className,
  children,
  ...props
}: TooltipWrapperProps): JSX.Element {
  const classes = [styles.tooltip, className].filter(Boolean).join(" ");
  return (
    <RACTooltip {...props} className={classes}>
      {children}
    </RACTooltip>
  );
}
