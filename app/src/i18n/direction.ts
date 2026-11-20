// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * Document direction and language (07 §7.11; REQ-I18N-002
 * infrastructure): `dir="rtl"` + `lang` for Arabic, LTR for the rest.
 * Components with non-mirrored surfaces (timeline axis, ruler,
 * transport, timecode) set `dir="ltr"` explicitly with a comment
 * citing §7.11 — see `format.ts`'s `ltrIsolate` for inline isolates.
 */
import type { LocaleTag } from "./runtime";

export function dirFor(locale: LocaleTag): "rtl" | "ltr" {
  return locale === "ar" ? "rtl" : "ltr";
}

/** Apply `dir`/`lang` to the document root for the active locale. */
export function applyDirection(locale: LocaleTag): void {
  document.documentElement.dir = dirFor(locale);
  document.documentElement.lang = locale;
}
