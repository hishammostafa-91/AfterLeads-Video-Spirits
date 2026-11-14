// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * CurveEditor (07 §7.6): the SVG curve editor — click empty space to
 * add a point, double-click or Delete to remove one, drag to move,
 * arrow keys nudge the focused point (Shift ×10), channel tabs switch
 * the edited channel, and Reset restores the default curve. Points
 * are normalized {x, y} in [0, 1], sorted by x.
 */
import type { JSX } from "react";
import { useRef, useState } from "react";
import { FormattedMessage, useIntl } from "react-intl";
import { Tab, TabList, TabPanel, Tabs } from "./Tabs";
import { Button } from "./Button";
import styles from "./CurveEditor.module.css";

export interface CurvePoint {
  /** Horizontal position, 0–1 (input). */
  x: number;
  /** Vertical position, 0–1 (output, 1 at the top edge). */
  y: number;
}

export type CurveChannel = "y" | "red" | "green" | "blue";

export type Curve = Record<CurveChannel, readonly CurvePoint[]>;

export const DEFAULT_CURVE: Curve = {
  y: [
    { x: 0, y: 0 },
    { x: 1, y: 1 },
  ],
  red: [
    { x: 0, y: 0 },
    { x: 1, y: 1 },
  ],
  green: [
    { x: 0, y: 0 },
    { x: 1, y: 1 },
  ],
  blue: [
    { x: 0, y: 0 },
    { x: 1, y: 1 },
  ],
};

const clamp01 = (n: number): number => Math.min(1, Math.max(0, n));

/** Points stay sorted by x; new points land at their x position. */
export function insertPoint(
  points: readonly CurvePoint[],
  point: CurvePoint,
): CurvePoint[] {
  const next = [...points, point].sort((a, b) => a.x - b.x);
  return next.map((p) => ({ x: clamp01(p.x), y: clamp01(p.y) }));
}

/** Remove by index. */
export function removePoint(
  points: readonly CurvePoint[],
  index: number,
): CurvePoint[] {
  return points.filter((_, i) => i !== index);
}

/** Move by index, keeping x order (no crossing past neighbours). */
export function movePoint(
  points: readonly CurvePoint[],
  index: number,
  delta: { x?: number; y?: number },
): CurvePoint[] {
  const point = index >= 0 && index < points.length ? points[index] : undefined;
  if (point === undefined) return [...points];
  const prev = index - 1 >= 0 ? points[index - 1] : undefined;
  const next = index + 1 < points.length ? points[index + 1] : undefined;
  let x = clamp01(point.x + (delta.x ?? 0));
  if (prev) x = Math.max(x, prev.x);
  if (next) x = Math.min(x, next.x);
  const moved = { x, y: clamp01(point.y + (delta.y ?? 0)) };
  return points.map((p, i) => (i === index ? moved : p));
}

const SIZE = 160;
const VIEW_BOX = `0 0 ${String(SIZE)} ${String(SIZE)}`;
const STEP = 0.01;

function toPixels(p: CurvePoint): { cx: number; cy: number } {
  return { cx: p.x * SIZE, cy: (1 - p.y) * SIZE };
}

function curvePath(points: readonly CurvePoint[]): string {
  return points
    .map((p, i) => {
      const { cx, cy } = toPixels(p);
      return `${i === 0 ? "M" : "L"}${cx.toFixed(2)},${cy.toFixed(2)}`;
    })
    .join(" ");
}

const CHANNELS: ReadonlyArray<{
  key: CurveChannel;
  messageId: string;
}> = [
  { key: "y", messageId: "components.curve.channel.y" },
  { key: "red", messageId: "components.curve.channel.red" },
  { key: "green", messageId: "components.curve.channel.green" },
  { key: "blue", messageId: "components.curve.channel.blue" },
];

export interface CurveEditorProps {
  value: Curve;
  onChange: (curve: Curve) => void;
  className?: string;
}

export function CurveEditor({
  value,
  onChange,
  className,
}: CurveEditorProps): JSX.Element {
  const { formatMessage } = useIntl();
  const [channel, setChannel] = useState<CurveChannel>("y");
  const [focused, setFocused] = useState<number | null>(null);
  const svgRef = useRef<SVGSVGElement>(null);
  const classes = [styles.editor, className].filter(Boolean).join(" ");
  const points = value[channel];

  const setChannelPoints = (next: readonly CurvePoint[]): void => {
    onChange({ ...value, [channel]: next });
  };

  const positionFromEvent = (event: React.PointerEvent): CurvePoint => {
    const rect = svgRef.current?.getBoundingClientRect();
    const px = rect ? (event.clientX - rect.left) / rect.width : 0;
    const py = rect ? 1 - (event.clientY - rect.top) / rect.height : 0;
    return { x: clamp01(px), y: clamp01(py) };
  };

  return (
    <div className={classes}>
      <Tabs
        defaultSelectedKey="y"
        onSelectionChange={(key) => {
          setChannel(String(key) as CurveChannel);
          setFocused(null);
        }}
      >
        <TabList
          aria-label={formatMessage({ id: "components.curve.channels" })}
        >
          {CHANNELS.map(({ key, messageId }) => (
            <Tab key={key} id={key}>
              {formatMessage({ id: messageId })}
            </Tab>
          ))}
        </TabList>
        {/* The graph is the selected channel's tab panel: the tab's
            aria-controls must resolve (react-aria sets it only on the
            selected tab; a Tabs without any TabPanel leaves it
            dangling and axe flags aria-valid-attr-value). */}
        <TabPanel id={channel} className={styles.panel}>
          <svg
            ref={svgRef}
            viewBox={VIEW_BOX}
            className={styles.curve}
            role="application"
            aria-label={formatMessage({ id: "components.curve.editor" })}
            onPointerDown={(event) => {
              // Click on empty space adds a point at the position.
              if (event.target === svgRef.current) {
                setChannelPoints(insertPoint(points, positionFromEvent(event)));
              }
            }}
          >
            <path
              className={styles.path}
              d={curvePath(points)}
              fill="none"
              stroke="var(--accent-solid)"
              strokeWidth={2}
            />
            {points.map((p, index) => {
              const { cx, cy } = toPixels(p);
              return (
                <circle
                  key={`${p.x.toFixed(3)}-${String(index)}`}
                  cx={cx}
                  cy={cy}
                  r={focused === index ? 6 : 5}
                  className={
                    focused === index ? styles.pointFocused : styles.point
                  }
                  tabIndex={0}
                  role="slider"
                  aria-label={formatMessage(
                    { id: "components.curve.point" },
                    { index: index + 1 },
                  )}
                  aria-valuenow={Math.round(p.y * 100)}
                  aria-valuetext={`${String(Math.round(p.x * 100))}% / ${String(
                    Math.round(p.y * 100),
                  )}%`}
                  onPointerDown={(event) => {
                    event.stopPropagation();
                    setFocused(index);
                    event.currentTarget.setPointerCapture(event.pointerId);
                  }}
                  onPointerMove={(event) => {
                    if (
                      event.currentTarget.hasPointerCapture(event.pointerId)
                    ) {
                      const at = positionFromEvent(event);
                      setChannelPoints(
                        movePoint(points, index, {
                          x: at.x - p.x,
                          y: at.y - p.y,
                        }),
                      );
                    }
                  }}
                  onFocus={() => {
                    setFocused(index);
                  }}
                  onKeyDown={(event) => {
                    const jump = event.shiftKey ? STEP * 10 : STEP;
                    if (event.key === "ArrowRight" || event.key === "ArrowUp") {
                      event.preventDefault();
                      setChannelPoints(
                        movePoint(points, index, {
                          x: event.key === "ArrowRight" ? jump : 0,
                          y: event.key === "ArrowUp" ? jump : 0,
                        }),
                      );
                    } else if (
                      event.key === "ArrowLeft" ||
                      event.key === "ArrowDown"
                    ) {
                      event.preventDefault();
                      setChannelPoints(
                        movePoint(points, index, {
                          x: event.key === "ArrowLeft" ? -jump : 0,
                          y: event.key === "ArrowDown" ? -jump : 0,
                        }),
                      );
                    } else if (
                      event.key === "Delete" ||
                      event.key === "Backspace"
                    ) {
                      event.preventDefault();
                      setChannelPoints(removePoint(points, index));
                      setFocused(null);
                    }
                  }}
                  onDoubleClick={() => {
                    setChannelPoints(removePoint(points, index));
                    setFocused(null);
                  }}
                />
              );
            })}
          </svg>
          <Button
            variant="ghost"
            onPress={() => {
              setChannelPoints(DEFAULT_CURVE[channel]);
              setFocused(null);
            }}
          >
            <FormattedMessage id="components.curve.reset" />
          </Button>
        </TabPanel>
      </Tabs>
    </div>
  );
}
