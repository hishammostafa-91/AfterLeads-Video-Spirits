// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * NumberField (07 §7.6): a scrubbable numeric field on RAC
 * NumberField — label-drag scrubbing with modifier factors (Shift ×10,
 * Ctrl ×0.1), wheel-when-focused, arrow keys, stepper buttons, typed
 * math entry (`+10`, `*2` — RAC parses plain numbers only, so relative
 * expressions are ours), a display-only unit suffix, and a mixed
 * (multi-selection) "—" state.
 */
import type { JSX } from "react";
import { useEffect, useRef } from "react";
import {
  Button as RACButton,
  Group as RACGroup,
  Input as RACInput,
  Label as RACLabel,
  NumberField as RACNumberField,
  type NumberFieldProps as RACNumberFieldProps,
} from "react-aria-components";
import { Minus, Plus } from "lucide-react";
import styles from "./NumberField.module.css";

/** Scrub speed factor from the held modifiers (07 §7.6). */
export function scrubMultiplier(modifiers: {
  shiftKey: boolean;
  ctrlKey: boolean;
}): number {
  if (modifiers.shiftKey) return 10;
  if (modifiers.ctrlKey) return 0.1;
  return 1;
}

export type NumberEntry =
  | { kind: "set"; value: number }
  | { kind: "relative"; op: "+" | "-" | "*"; operand: number }
  | null;

/**
 * Parse typed entry: unsigned numbers set the value; a leading
 * `+`/`-`/`*` makes it relative math against the current value
 * (07 §7.6) — negatives are entered as subtraction, matching the
 * relative-entry convention.
 */
export function parseNumberEntry(text: string): NumberEntry {
  const t = text.trim().replace(",", ".");
  const relative = /^([+*-])\s*(\d+(\.\d+)?)$/.exec(t);
  if (relative) {
    const op = relative[1] as "+" | "-" | "*";
    return { kind: "relative", op, operand: Number(relative[2]) };
  }
  const plain = /^\d+(\.\d+)?$/.exec(t);
  if (plain) return { kind: "set", value: Number(t) };
  return null;
}

/** Apply a parsed entry to the current value. */
export function applyEntry(
  current: number,
  entry: Exclude<NumberEntry, null>,
): number {
  if (entry.kind === "set") return entry.value;
  if (entry.op === "+") return current + entry.operand;
  if (entry.op === "-") return current - entry.operand;
  return current * entry.operand;
}

export interface NumberFieldWrapperProps extends Omit<
  RACNumberFieldProps,
  "className"
> {
  className?: string;
  label?: string;
  /** Display-only unit suffix, e.g. "%" or "px" (not translated). */
  suffix?: string;
  /** Mixed multi-selection state: the field shows "—" until edited. */
  isMixed?: boolean;
}

export function NumberField({
  className,
  label,
  suffix,
  isMixed = false,
  value,
  defaultValue,
  onChange,
  minValue,
  maxValue,
  step = 1,
  ...props
}: NumberFieldWrapperProps): JSX.Element {
  const inputRef = useRef<HTMLInputElement>(null);
  const classes = [styles.field, className].filter(Boolean).join(" ");

  const clamp = (n: number): number => {
    let out = n;
    if (minValue !== undefined) out = Math.max(minValue, out);
    if (maxValue !== undefined) out = Math.min(maxValue, out);
    return out;
  };

  const commitEntry = (text: string): void => {
    const current =
      value ?? defaultValue ?? (minValue !== undefined ? minValue : 0);
    const entry = parseNumberEntry(text);
    if (!entry) return;
    onChange?.(clamp(applyEntry(current, entry)));
  };

  // Wheel-when-focused: the focused field steps by the wheel delta
  // sign, honoring the scrub modifiers (07 §7.6).
  const onWheel = (event: React.WheelEvent<HTMLDivElement>): void => {
    if (!inputRef.current || document.activeElement !== inputRef.current) {
      return;
    }
    event.preventDefault();
    const factor = scrubMultiplier({
      shiftKey: event.shiftKey,
      ctrlKey: event.ctrlKey,
    });
    const dir = event.deltaY > 0 ? -1 : 1;
    const current =
      value ?? defaultValue ?? (minValue !== undefined ? minValue : 0);
    onChange?.(clamp(current + dir * step * factor));
  };

  // Typed math commits on Enter/blur; plain numbers fall through to
  // RAC's own commit. The draft is tracked in a ref (capture phase)
  // because a controlled RAC NumberField reverts unparsable text
  // synchronously — the DOM value cannot be read back at commit time.
  const draftRef = useRef<string | null>(null);
  const onChangeCapture = (
    event: React.ChangeEvent<HTMLInputElement>,
  ): void => {
    draftRef.current = event.target.value;
  };
  const commitMath = (event: React.SyntheticEvent): boolean => {
    const draft = draftRef.current;
    draftRef.current = null;
    if (draft === null) return false;
    const entry = parseNumberEntry(draft);
    if (entry?.kind !== "relative") return false;
    event.preventDefault();
    event.stopPropagation();
    commitEntry(draft);
    return true;
  };
  const onKeyDownCapture = (
    event: React.KeyboardEvent<HTMLInputElement>,
  ): void => {
    if (event.key === "Enter") {
      commitMath(event);
    }
  };
  const onBlurCapture = (event: React.FocusEvent<HTMLInputElement>): void => {
    commitMath(event);
  };

  // Scrub by dragging the label horizontally (07 §7.6).
  const startScrub = (event: React.PointerEvent<HTMLLabelElement>): void => {
    if (isMixed || props.isDisabled) return;
    event.preventDefault();
    const labelEl = event.currentTarget;
    labelEl.setPointerCapture(event.pointerId);
    let last = event.clientX;
    const move = (moveEvent: PointerEvent): void => {
      const dx = moveEvent.clientX - last;
      last = moveEvent.clientX;
      const factor = scrubMultiplier({
        shiftKey: moveEvent.shiftKey,
        ctrlKey: moveEvent.ctrlKey,
      });
      const current =
        value ?? defaultValue ?? (minValue !== undefined ? minValue : 0);
      onChange?.(clamp(current + dx * step * factor));
    };
    const end = (): void => {
      labelEl.removeEventListener("pointermove", move);
      labelEl.removeEventListener("pointerup", end);
      labelEl.removeEventListener("pointercancel", end);
    };
    labelEl.addEventListener("pointermove", move);
    labelEl.addEventListener("pointerup", end);
    labelEl.addEventListener("pointercancel", end);
  };

  // Mixed state: show "—" as the placeholder of the empty field.
  useEffect(() => {
    if (isMixed && inputRef.current) {
      inputRef.current.placeholder = "—";
    }
  }, [isMixed]);

  return (
    <RACNumberField
      {...props}
      value={isMixed ? undefined : value}
      defaultValue={isMixed ? undefined : defaultValue}
      onChange={isMixed ? undefined : onChange}
      minValue={minValue}
      maxValue={maxValue}
      step={step}
      className={classes}
    >
      {label && (
        <RACLabel className={styles.label} onPointerDown={startScrub}>
          {label}
        </RACLabel>
      )}
      <RACGroup className={styles.group} aria-label={label} onWheel={onWheel}>
        <RACButton slot="decrement" className={styles.stepper}>
          <Minus size={12} aria-hidden="true" />
        </RACButton>
        <RACInput
          ref={inputRef}
          className={styles.input}
          onChangeCapture={onChangeCapture}
          onKeyDownCapture={onKeyDownCapture}
          onBlurCapture={onBlurCapture}
        />
        {suffix && (
          <span className={styles.suffix} aria-hidden="true">
            {suffix}
          </span>
        )}
        <RACButton slot="increment" className={styles.stepper}>
          <Plus size={12} aria-hidden="true" />
        </RACButton>
      </RACGroup>
    </RACNumberField>
  );
}
