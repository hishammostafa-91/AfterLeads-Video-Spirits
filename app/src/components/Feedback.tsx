// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * Feedback (07 §7.6): ProgressBar, Meter (RAC), Spinner (custom —
 * indeterminate ONLY until the first progress value exists, per the
 * doc), and Skeleton placeholders.
 */
import type { JSX } from "react";
import { useIntl } from "react-intl";
import {
  Label as RACLabel,
  Meter as RACMeter,
  type MeterProps,
  ProgressBar as RACProgressBar,
  type ProgressBarProps,
} from "react-aria-components";
import styles from "./Feedback.module.css";

function cx(...parts: Array<string | undefined>): string {
  return parts.filter(Boolean).join(" ");
}

export interface ProgressBarWrapperProps extends Omit<
  ProgressBarProps,
  "className"
> {
  className?: string;
  label?: string;
}

/** Determinate progress (a known fraction of work done). */
export function ProgressBar({
  className,
  label,
  ...props
}: ProgressBarWrapperProps): JSX.Element {
  return (
    <RACProgressBar {...props} className={cx(styles.bar, className)}>
      {label && <RACLabel className={styles.barLabel}>{label}</RACLabel>}
    </RACProgressBar>
  );
}

export interface MeterWrapperProps extends Omit<MeterProps, "className"> {
  className?: string;
  label?: string;
}

/** A level gauge (e.g. cache fill) — value without progress semantics. */
export function Meter({
  className,
  label,
  ...props
}: MeterWrapperProps): JSX.Element {
  return (
    <RACMeter {...props} className={cx(styles.bar, className)}>
      {label && <RACLabel className={styles.barLabel}>{label}</RACLabel>}
    </RACMeter>
  );
}

export interface SpinnerProps {
  /**
   * The current progress value (0–100). While undefined the spinner
   * is indeterminate — a value makes it determinate (07 §7.6).
   */
  value?: number;
  className?: string;
}

/** An indeterminate-until-valued spinner. */
export function Spinner({ value, className }: SpinnerProps): JSX.Element {
  const { formatMessage } = useIntl();
  const classes = [styles.spinner, className].filter(Boolean).join(" ");
  const determinate = value !== undefined;
  return (
    <svg
      viewBox="0 0 24 24"
      className={classes}
      role="progressbar"
      aria-label={formatMessage({ id: "components.feedback.spinner" })}
      aria-valuenow={determinate ? Math.round(value) : undefined}
    >
      <circle
        cx={12}
        cy={12}
        r={9}
        fill="none"
        strokeWidth={2.5}
        stroke="var(--border-strong)"
        className={styles.spinnerTrack}
      />
      {determinate ? (
        <circle
          cx={12}
          cy={12}
          r={9}
          fill="none"
          strokeWidth={2.5}
          stroke="var(--accent-solid)"
          strokeDasharray={2 * Math.PI * 9}
          strokeDashoffset={2 * Math.PI * 9 * (1 - value / 100)}
          transform="rotate(-90 12 12)"
        />
      ) : (
        <circle
          cx={12}
          cy={12}
          r={9}
          fill="none"
          strokeWidth={2.5}
          stroke="var(--accent-solid)"
          strokeDasharray={`${String(2 * Math.PI * 9 * 0.25)} ${String(
            2 * Math.PI * 9 * 0.75,
          )}`}
          className={styles.spinnerIndeterminate}
        />
      )}
    </svg>
  );
}

export interface SkeletonProps {
  /** Width in characters-ish units (rem). */
  inlineSize?: number;
  blockSize?: number;
  shape?: "line" | "block";
  className?: string;
}

/** A loading placeholder block. */
export function Skeleton({
  inlineSize = 8,
  blockSize = 1,
  shape = "line",
  className,
}: SkeletonProps): JSX.Element {
  const classes = [
    styles.skeleton,
    shape === "block" ? styles.skeletonBlock : styles.skeletonLine,
    className,
  ]
    .filter(Boolean)
    .join(" ");
  return (
    <div
      aria-hidden="true"
      className={classes}
      style={{
        inlineSize: `${String(inlineSize)}rem`,
        blockSize: `${String(blockSize)}rem`,
      }}
    />
  );
}
