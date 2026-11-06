// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * ColorWheel (07 §7.6): the chroma wheel with its puck plus a luma
 * slider — RAC ColorWheel carries the chroma interaction (pointer,
 * keyboard, RTL-safe) with a double-click reset to the default and
 * Alt-drag fine control layered on top; the lightness slider is RAC
 * ColorSlider on the `lightness` channel.
 */
import type { JSX } from "react";
import { useRef } from "react";
import { useIntl } from "react-intl";
import {
  ColorSlider as RACColorSlider,
  ColorThumb,
  ColorWheel as RACColorWheel,
  parseColor,
} from "react-aria-components";
import styles from "./ColorWheel.module.css";

/** Angular delta between two wheel angles, wrapped to (-180, 180]. */
export function angleDelta(from: number, to: number): number {
  let d = to - from;
  while (d > 180) d -= 360;
  while (d <= -180) d += 360;
  return d;
}

export interface ColorWheelProps {
  /** The current color as a hex string. */
  value: string;
  onChange: (hex: string) => void;
  /** Double-click resets to this hex. */
  defaultValue: string;
  className?: string;
}

export function ColorWheel({
  value,
  onChange,
  defaultValue,
  className,
}: ColorWheelProps): JSX.Element {
  const { formatMessage } = useIntl();
  const color = parseColor(value);
  const classes = [styles.wrapper, className].filter(Boolean).join(" ");
  const wrapRef = useRef<HTMLDivElement>(null);
  const change = (next: ReturnType<typeof parseColor>): void => {
    onChange(next.toString("hex"));
  };

  // Alt-drag fine control: while Alt is held the drag takes over
  // (capture phase, before RAC's wheel handlers) and applies one
  // tenth of the angular travel to the hue channel (07 §7.6).
  const onPointerDownCapture = (
    event: React.PointerEvent<HTMLDivElement>,
  ): void => {
    if (!event.altKey) return;
    event.preventDefault();
    event.stopPropagation();
    const el = wrapRef.current;
    if (!el) return;
    const rect = el.getBoundingClientRect();
    const center = {
      x: rect.left + rect.width / 2,
      y: rect.top + rect.height / 2,
    };
    const angleAt = (x: number, y: number): number =>
      (Math.atan2(x - center.x, center.y - y) * 180) / Math.PI;
    const startAngle = angleAt(event.clientX, event.clientY);
    const startHue = color.getChannelValue("hue");
    el.setPointerCapture(event.pointerId);
    const move = (e: PointerEvent): void => {
      const delta = angleDelta(startAngle, angleAt(e.clientX, e.clientY));
      const hue = (((startHue + delta * 0.1) % 360) + 360) % 360;
      change(color.withChannelValue("hue", hue));
    };
    const end = (): void => {
      el.removeEventListener("pointermove", move);
      el.removeEventListener("pointerup", end);
      el.removeEventListener("pointercancel", end);
    };
    el.addEventListener("pointermove", move);
    el.addEventListener("pointerup", end);
    el.addEventListener("pointercancel", end);
  };

  return (
    <div className={classes}>
      <div
        ref={wrapRef}
        className={styles.wheelWrap}
        onPointerDownCapture={onPointerDownCapture}
        onDoubleClick={() => {
          onChange(defaultValue);
        }}
      >
        <RACColorWheel
          value={color}
          onChange={change}
          outerRadius={40}
          innerRadius={28}
          aria-label={formatMessage({ id: "components.color.wheel" })}
          className={styles.wheel}
        >
          <ColorThumb className={styles.puck} />
        </RACColorWheel>
      </div>
      <RACColorSlider
        channel="lightness"
        value={color.toFormat("hsl")}
        onChange={change}
        aria-label={formatMessage({ id: "components.color.luma" })}
        className={styles.luma}
      >
        <ColorThumb className={styles.puck} />
      </RACColorSlider>
    </div>
  );
}
