// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * Pseudo-locales (12 §12.14; research D14): LOC-90 `en-XA` (accented
 * Latin, +40% length, wrapping brackets — the 07 §7.12 rule 5 layout
 * budget) and LOC-91 `ar-XB` (RTL with mirrored rendering and bidi
 * marks). Generated at test time from the real catalogs so they can
 * never drift; test/harness use only, never shipped as locale files.
 */
import type { LocaleTag } from "./runtime";

export const PSEUDO_TAGS = ["en-XA", "ar-XB"] as const;
export type PseudoTag = (typeof PSEUDO_TAGS)[number];

const ACCENTS: Record<string, string> = {
  a: "à",
  e: "é",
  i: "î",
  o: "ô",
  u: "û",
  A: "À",
  E: "É",
  I: "Î",
  O: "Ô",
  U: "Û",
};

function accent(ch: string): string {
  return ACCENTS[ch] ?? ch;
}

/** Transform only literal text — ICU `{…}` expressions stay intact. */
function transformLiteral(
  message: string,
  transform: (text: string) => string,
): string {
  let out = "";
  let depth = 0;
  let literal = "";
  for (const ch of message) {
    if (ch === "{") {
      if (depth === 0) {
        out += transform(literal);
        literal = "";
      }
      depth += 1;
      out += ch;
    } else if (ch === "}") {
      depth -= 1;
      out += ch;
    } else if (depth === 0) {
      literal += ch;
    } else {
      out += ch;
    }
  }
  return out + transform(literal);
}

/** LOC-90: accented, padded to ≥1.4× the source length, bracketed. */
export function pseudoEnglish(
  catalog: Record<string, string>,
): Record<string, string> {
  const out: Record<string, string> = {};
  for (const [key, value] of Object.entries(catalog)) {
    let accented = transformLiteral(value, (text) =>
      Array.from(text, accent).join(""),
    );
    const budget = Math.ceil(value.length * 1.4);
    while (accented.length < budget) accented += "àà";
    out[key] = `[${accented}]`;
  }
  return out;
}

/** LOC-91: wrapped in a right-to-left isolate with bidi marks. */
export function pseudoArabic(
  catalog: Record<string, string>,
): Record<string, string> {
  const out: Record<string, string> = {};
  const RLI = "⁧";
  const PDI = "⁩";
  const RLM = "‏";
  for (const [key, value] of Object.entries(catalog)) {
    out[key] = `${RLM}${RLI}${value}${PDI}${RLM}`;
  }
  return out;
}

/** The base locale a pseudo tag renders with. */
export function pseudoBase(tag: PseudoTag): LocaleTag {
  return tag === "en-XA" ? "en" : "ar";
}

/** Generate a pseudo catalog from the real one (tests only). */
export function pseudoCatalog(
  tag: PseudoTag,
  source: Record<string, string>,
): Record<string, string> {
  return tag === "en-XA" ? pseudoEnglish(source) : pseudoArabic(source);
}
