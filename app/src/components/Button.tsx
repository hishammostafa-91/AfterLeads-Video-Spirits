// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * Buttons (07 §7.6): Button (primary/secondary/ghost/danger),
 * ToggleButton, IconButton (icon + tooltip with label+shortcut +
 * accessible name — REQ-HLP-003), and SegmentedControl
 * (RadioGroup-composed single-choice segments, per contracts §3).
 */
import type { JSX, ReactNode } from "react";
import {
  Button as RACButton,
  type ButtonProps as RACButtonProps,
  RadioButton,
  type RadioFieldProps,
  RadioField,
  type RadioGroupProps as RACRadioGroupProps,
  RadioGroup as RACRadioGroup,
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
  className,
  ...props
}: IconButtonProps): JSX.Element {
  const { formatMessage } = useIntl();
  const label = formatMessage({ id: labelMessageId });
  return (
    <TooltipTrigger delay={500}>
      <RACButton
        {...props}
        aria-label={label}
        className={classes("ghost", styles.iconButton, className)}
      >
        {icon}
      </RACButton>
      <Tooltip>
        {shortcut
          ? formatMessage(
              { id: "components.iconButton.tooltip" },
              { label, shortcut },
            )
          : label}
      </Tooltip>
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

export interface SegmentedControlProps extends Omit<
  RACRadioGroupProps,
  "className" | "children"
> {
  className?: string;
  /** `<Segment>` children (07 §7.6: RadioGroup-composed). */
  children?: ReactNode;
}

/**
 * A single-choice segmented control (07 §7.6) — a horizontal
 * RadioGroup rendered as joined segments; arrow keys and typeahead
 * come from the radio semantics.
 */
export function SegmentedControl({
  className,
  children,
  ...props
}: SegmentedControlProps): JSX.Element {
  return (
    <RACRadioGroup
      {...props}
      orientation="horizontal"
      className={[styles.segmented, className].filter(Boolean).join(" ")}
    >
      {children}
    </RACRadioGroup>
  );
}

export interface SegmentProps extends Omit<RadioFieldProps, "className"> {
  className?: string;
  children?: ReactNode;
}

/** One segment of a SegmentedControl. */
export function Segment({
  className,
  children,
  ...props
}: SegmentProps): JSX.Element {
  return (
    <RadioField
      {...props}
      className={[styles.segmentField, className].filter(Boolean).join(" ")}
    >
      <RadioButton className={styles.segment}>{children}</RadioButton>
    </RadioField>
  );
}
