// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * TimecodeField (07 §7.6): the segmented HH:MM:SS:FF field — typed
 * absolute entry, `+N`/`-N` relative frame entry, arrow-key stepping
 * with rollover, and always `dir="ltr"`: a timecode is one of the
 * non-mirrored surfaces (docs/07 §7.11), so the component pins its
 * direction and layout regardless of the UI locale.
 */
import type { JSX } from "react";
import { useRef, useState } from "react";
import { useIntl } from "react-intl";
import styles from "./TimecodeField.module.css";

export interface Timecode {
  hours: number;
  minutes: number;
  seconds: number;
  frames: number;
}

/** Segments → whole frames at the given frame rate. */
export function timecodeToFrames(tc: Timecode, fps: number): number {
  return ((tc.hours * 60 + tc.minutes) * 60 + tc.seconds) * fps + tc.frames;
}

/** Whole frames → segments at the given frame rate. */
export function framesToTimecode(frames: number, fps: number): Timecode {
  const total = Math.max(0, Math.floor(frames));
  const hours = Math.floor(total / (3600 * fps));
  const minutes = Math.floor(total / (60 * fps)) % 60;
  const seconds = Math.floor(total / fps) % 60;
  const f = total % Math.floor(fps);
  return { hours, minutes, seconds, frames: f };
}

export type TimecodeEntry =
  | { kind: "absolute"; frames: number }
  | { kind: "relative"; delta: number }
  | null;

/**
 * Parse typed entry: `HH:MM:SS:FF` (or partial) sets an absolute
 * timecode; `+N` / `-N` are relative frames (07 §7.6).
 */
export function parseTimecodeEntry(text: string, fps: number): TimecodeEntry {
  const t = text.trim();
  const relative = /^([+-])\s*(\d+)$/.exec(t);
  if (relative) {
    const n = Number(relative[2]);
    return { kind: "relative", delta: relative[1] === "+" ? n : -n };
  }
  const parts = t.split(/[:.]/).map((p) => p.trim());
  if (parts.length === 0 || parts.length > 4) return null;
  const numbers: number[] = [];
  for (const part of parts) {
    if (!/^\d{1,2}$/.test(part)) return null;
    numbers.push(Number(part));
  }
  while (numbers.length < 4) numbers.unshift(0);
  const [hours, minutes, seconds, frames] = numbers;
  if (minutes > 59 || seconds > 59 || frames >= fps) return null;
  return {
    kind: "absolute",
    frames: timecodeToFrames({ hours, minutes, seconds, frames }, fps),
  };
}

const SEGMENTS = [
  { key: "hh", messageId: "components.timecode.segment.hh", max: 99 },
  { key: "mm", messageId: "components.timecode.segment.mm", max: 59 },
  { key: "ss", messageId: "components.timecode.segment.ss", max: 59 },
  { key: "ff", messageId: "components.timecode.segment.ff", max: null },
] as const;

function pad(n: number): string {
  return String(n).padStart(2, "0");
}

export interface TimecodeFieldProps {
  /** Whole frames. */
  value: number;
  onChange: (frames: number) => void;
  /** Frame rate of the FF segment rollover. */
  fps: number;
  className?: string;
}

/**
 * The segmented timecode field. Every segment auto-selects on focus,
 * ArrowUp/ArrowDown steps it (FF rolls within the frame rate, MM/SS
 * within 59, HH clamps), and typing commits on Enter/blur — full or
 * partial timecodes set, `+N`/`-N` nudge by frames. The whole field
 * is `dir="ltr"` (non-mirrored surface, docs/07 §7.11).
 */
export function TimecodeField({
  value,
  onChange,
  fps,
  className,
}: TimecodeFieldProps): JSX.Element {
  const { formatMessage } = useIntl();
  const tc = framesToTimecode(value, fps);
  const inputsRef = useRef<Array<HTMLInputElement | null>>([]);
  const classes = [styles.field, className].filter(Boolean).join(" ");
  // Draft text per segment while typing; null renders the committed
  // value so external updates (scrub, stepping, undo) always show.
  const [drafts, setDrafts] = useState<Array<string | null>>([
    null,
    null,
    null,
    null,
  ]);
  const digits = [tc.hours, tc.minutes, tc.seconds, tc.frames];

  const commit = (): void => {
    const raws = inputsRef.current.map((input, index) =>
      input === null || input.value === ""
        ? pad(digits[index])
        : input.value.trim(),
    );
    // Relative entry (`+N` / `-N`) typed into any single segment
    // nudges the whole timecode by frames (07 s7.6).
    const relative = raws.find((raw) => /^[+-]\d+$/.test(raw));
    if (relative !== undefined) {
      const delta = Number(relative);
      onChange(Math.max(0, value + delta));
      setDrafts([null, null, null, null]);
      return;
    }
    const entry = parseTimecodeEntry(raws.join(":"), fps);
    if (entry) {
      onChange(
        entry.kind === "relative"
          ? Math.max(0, value + entry.delta)
          : entry.frames,
      );
    }
    setDrafts([null, null, null, null]);
  };

  const stepSegment = (index: number, delta: number): void => {
    // FF rolls within the frame rate; MM/SS roll within 59; HH clamps.
    const max =
      index === 3 ? Math.floor(fps) - 1 : (SEGMENTS[index]?.max ?? 59);
    const current = digits[index];
    let next = current + delta;
    if (next > max) next = index === 0 ? current : 0;
    if (next < 0) next = index === 0 ? 0 : max;
    const segments = [...digits];
    segments[index] = next;
    setDrafts([null, null, null, null]);
    onChange(
      timecodeToFrames(
        {
          hours: segments[0],
          minutes: segments[1],
          seconds: segments[2],
          frames: segments[3],
        },
        fps,
      ),
    );
  };

  const segmentProps = (index: number) => ({
    "aria-label": formatMessage({ id: SEGMENTS[index].messageId }),
    onFocus: (event: React.FocusEvent<HTMLInputElement>): void => {
      event.currentTarget.select();
    },
    onChange: (event: React.ChangeEvent<HTMLInputElement>): void => {
      const text = event.currentTarget.value;
      setDrafts((prev) => prev.map((d, i) => (i === index ? text : d)));
    },
    onKeyDown: (event: React.KeyboardEvent<HTMLInputElement>): void => {
      if (event.key === "ArrowUp") {
        event.preventDefault();
        stepSegment(index, 1);
      } else if (event.key === "ArrowDown") {
        event.preventDefault();
        stepSegment(index, -1);
      } else if (event.key === "Enter") {
        commit();
      }
    },
    onBlur: commit,
  });

  return (
    // A timecode is a non-mirrored surface (07 §7.11): pin dir="ltr".
    <div
      dir="ltr"
      className={classes}
      aria-label={formatMessage({ id: "components.timecode.label" })}
    >
      {digits.map((digit, index) => (
        <span key={index} className={styles.segmentWrap}>
          {index > 0 && (
            <span className={styles.colon} aria-hidden="true">
              :
            </span>
          )}
          <input
            ref={(el) => {
              inputsRef.current[index] = el;
            }}
            className={styles.segment}
            inputMode="numeric"
            maxLength={2}
            value={drafts[index] ?? pad(digit)}
            {...segmentProps(index)}
          />
        </span>
      ))}
    </div>
  );
}
