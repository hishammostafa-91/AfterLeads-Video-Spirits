// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * Slider and AngleDial (07 §7.6): Slider on RAC Slider (keyboard,
 * snapping, RTL mirroring come from RAC); AngleDial is the custom
 * circular dial the doc describes — pointer drag sets the angle,
 * double-click resets to the default, keyboard arrows step
 * (Shift ×10), and a paired NumberField is the exact-value companion.
 */
import type { JSX, ReactNode } from "react";
import { useRef } from "react";
import {
  Label as RACLabel,
  Slider as RACSlider,
  type SliderProps,
  SliderFill as RACSliderFill,
  SliderOutput as RACSliderOutput,
  SliderThumb as RACSliderThumb,
  SliderTrack as RACSliderTrack,
} from "react-aria-components";
import styles from "./Slider.module.css";

function cx(...parts: Array<string | undefined>): string {
  return parts.filter(Boolean).join(" ");
}

export interface SliderWrapperProps<T = number | number[]> extends Omit<
  SliderProps<T>,
  "className"
> {
  className?: string;
  label?: ReactNode;
}

/** A linear slider with label, value output, fill, and thumb. */
export function Slider<T extends number | number[]>({
  className,
  label,
  ...props
}: SliderWrapperProps<T>): JSX.Element {
  return (
    <RACSlider {...props} className={cx(styles.slider, className)}>
      <div className={styles.row}>
        {label && <RACLabel className={styles.label}>{label}</RACLabel>}
        <RACSliderOutput className={styles.output} />
      </div>
      <RACSliderTrack className={styles.track}>
        <RACSliderFill className={styles.fill} />
        <RACSliderThumb className={styles.thumb} />
      </RACSliderTrack>
    </RACSlider>
  );
}

export interface AngleDialProps {
  /** Angle in degrees, 0–360 (0 at 12 o'clock, clockwise). */
  value: number;
  /** The value double-click resets to. */
  defaultValue?: number;
  onChange: (value: number) => void;
  /** Accessible name (already translated). */
  label: string;
  className?: string;
}

/** Normalize any angle into [0, 360). */
export function normalizeAngle(deg: number): number {
  return ((deg % 360) + 360) % 360;
}

/** Pointer position (relative to center) → angle in degrees. */
export function pointToAngle(x: number, y: number): number {
  // Screen y grows downward; 0° at 12 o'clock, clockwise.
  return normalizeAngle((Math.atan2(x, -y) * 180) / Math.PI);
}

/**
 * The angle dial (07 §7.6): drag anywhere on the disc to set the
 * angle; double-click resets; Arrow keys step 1° (Shift 10°). Exposed
 * as a `role="slider"` for assistive technology.
 */
export function AngleDial({
  value,
  defaultValue = 0,
  onChange,
  label,
  className,
}: AngleDialProps): JSX.Element {
  const dialRef = useRef<HTMLDivElement>(null);
  const pointerAngle = (event: React.PointerEvent<HTMLDivElement>): number => {
    const rect = dialRef.current?.getBoundingClientRect();
    const cxPoint = rect ? rect.left + rect.width / 2 : 0;
    const cyPoint = rect ? rect.top + rect.height / 2 : 0;
    return pointToAngle(event.clientX - cxPoint, event.clientY - cyPoint);
  };
  const onKeyDown = (event: React.KeyboardEvent<HTMLDivElement>): void => {
    const stepSize = event.shiftKey ? 10 : 1;
    if (event.key === "ArrowRight" || event.key === "ArrowUp") {
      event.preventDefault();
      onChange(normalizeAngle(value + stepSize));
    } else if (event.key === "ArrowLeft" || event.key === "ArrowDown") {
      event.preventDefault();
      onChange(normalizeAngle(value - stepSize));
    } else if (event.key === "Home") {
      event.preventDefault();
      onChange(normalizeAngle(defaultValue));
    }
  };
  return (
    <div
      ref={dialRef}
      role="slider"
      aria-label={label}
      aria-valuemin={0}
      aria-valuemax={360}
      aria-valuenow={Math.round(normalizeAngle(value))}
      aria-valuetext={`${String(Math.round(normalizeAngle(value)))}\u00b0`}
      tabIndex={0}
      className={cx(styles.dial, className)}
      onKeyDown={onKeyDown}
      onDoubleClick={() => {
        onChange(normalizeAngle(defaultValue));
      }}
      onPointerDown={(event) => {
        event.currentTarget.setPointerCapture(event.pointerId);
        onChange(pointerAngle(event));
      }}
      onPointerMove={(event) => {
        if (event.currentTarget.hasPointerCapture(event.pointerId)) {
          onChange(pointerAngle(event));
        }
      }}
    >
      <span
        className={styles.dialNeedle}
        style={{ rotate: `${String(normalizeAngle(value))}deg` }}
        aria-hidden="true"
      />
    </div>
  );
}
