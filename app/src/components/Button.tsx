// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * Buttons (07 §7.6): Button (primary/secondary/ghost/danger),
 * ToggleButton, and IconButton (icon + tooltip with label+shortcut +
 * accessible name — REQ-HLP-003). Built on React Aria Components.
 */
import type { JSX, ReactNode } from "react";
import {
  Button as RACButton,
  type ButtonProps as RACButtonProps,
  ToggleButton as RACToggleButton,
  type ToggleButtonProps as RACToggleButtonProps,
  Tooltip,
  TooltipTrigger,
} from "react-aria-components";
import { useIntl } from "react-intl";
import styles from "./Button.module.css";

export type ButtonVariant = "primary" | "secondary" | "ghost" | "danger";

function classes(
  variant: ButtonVariant,
  extra: string,
  className: string | undefined,
): string {
  return [styles.button, styles[variant], extra, className]
    .filter(Boolean)
    .join(" ");
}

export interface ButtonProps extends Omit<RACButtonProps, "className"> {
  variant?: ButtonVariant;
  className?: string;
  children?: ReactNode;
}

export function Button({
  variant = "secondary",
  className,
  ...props
}: ButtonProps): JSX.Element {
  return <RACButton {...props} className={classes(variant, "", className)} />;
}

export interface IconButtonProps extends Omit<RACButtonProps, "className"> {
  icon: ReactNode;
  labelMessageId: string;
  shortcut?: string;
  className?: string;
}

export function IconButton({
  icon,
  labelMessageId,
  shortcut,
  ...props
}: IconButtonProps): JSX.Element {
  const { formatMessage } = useIntl();
  const label = formatMessage({ id: labelMessageId });
  return (
    <TooltipTrigger delay={500}>
      <RACButton {...props} aria-label={label}>
        {icon}
      </RACButton>
      <Tooltip>{shortcut ? `${label} (${shortcut})` : label}</Tooltip>
    </TooltipTrigger>
  );
}

export interface ToggleButtonProps extends Omit<
  RACToggleButtonProps,
  "className"
> {
  className?: string;
  children?: ReactNode;
}

export function ToggleButton({
  className,
  ...props
}: ToggleButtonProps): JSX.Element {
  return (
    <RACToggleButton
      {...props}
      className={classes("secondary", styles.toggle, className)}
    />
  );
}
