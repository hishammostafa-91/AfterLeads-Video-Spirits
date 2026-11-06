// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * PointControl (07 §7.6): the numeric row for one selected curve
 * point — X/Y percentages plus the handle slot choosing how the
 * point's bezier handles behave.
 */
import type { JSX } from "react";
import { useIntl } from "react-intl";
import { ListBoxItem } from "./Selection";
import { NumberField } from "./NumberField";
import { Select } from "./Selection";
import styles from "./PointControl.module.css";

export type HandleMode = "none" | "smooth";

export interface PointControlProps {
  /** Point position in percent (0–100 each axis). */
  x: number;
  y: number;
  onX: (percent: number) => void;
  onY: (percent: number) => void;
  handle: HandleMode;
  onHandle: (mode: HandleMode) => void;
  className?: string;
}

export function PointControl({
  x,
  y,
  onX,
  onY,
  handle,
  onHandle,
  className,
}: PointControlProps): JSX.Element {
  const { formatMessage } = useIntl();
  const classes = [styles.row, className].filter(Boolean).join(" ");
  return (
    <div className={classes}>
      <NumberField
        label={formatMessage({ id: "components.curve.x" })}
        value={x}
        minValue={0}
        maxValue={100}
        step={1}
        suffix="%"
        onChange={onX}
      />
      <NumberField
        label={formatMessage({ id: "components.curve.y" })}
        value={y}
        minValue={0}
        maxValue={100}
        step={1}
        suffix="%"
        onChange={onY}
      />
      <Select
        label={formatMessage({ id: "components.curve.handle" })}
        value={handle}
        onChange={(key) => {
          const next = String(key);
          if (next === "none" || next === "smooth") onHandle(next);
        }}
      >
        <ListBoxItem id="none">
          {formatMessage({ id: "components.curve.handleNone" })}
        </ListBoxItem>
        <ListBoxItem id="smooth">
          {formatMessage({ id: "components.curve.handleSmooth" })}
        </ListBoxItem>
      </Select>
    </div>
  );
}
