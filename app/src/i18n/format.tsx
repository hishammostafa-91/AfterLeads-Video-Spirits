// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * Locale formatting (REQ-I18N-003; 07 §7.12 rule 6): numbers and dates
 * via `Intl` with the active locale and `numberingSystem: "latn"` —
 * Western digits everywhere; timecode, frame numbers, and durations
 * are wrapped in LTR isolates (`<bdi dir="ltr">`) per 07 §7.11.
 */
import type { JSX } from "react";
import type { LocaleTag } from "./runtime";

/** Format a number in the locale with Western digits (07 §7.12 rule 6). */
export function formatNumber(value: number, locale: LocaleTag): string {
  return new Intl.NumberFormat(locale, {
    numberingSystem: "latn",
  }).format(value);
}

/** Format a date in the locale with Western digits. */
export function formatDate(
  date: Date,
  locale: LocaleTag,
  options: Intl.DateTimeFormatOptions = { dateStyle: "medium" },
): string {
  return new Intl.DateTimeFormat(locale, {
    ...options,
    numberingSystem: "latn",
  }).format(date);
}

/**
 * Wrap always-LTR text (timecode `HH:MM:SS:FF`, frame counts,
 * durations) in an isolated `<bdi dir="ltr">` so RTL prose never
 * reorders it (07 §7.11).
 */
export function ltrIsolate(text: string): JSX.Element {
  return <bdi dir="ltr">{text}</bdi>;
}
