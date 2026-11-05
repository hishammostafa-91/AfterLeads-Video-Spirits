// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * The localization gate's structural core (docs/08 §8.4; T007/T014):
 * identical key sets across the six shipped locales (REQ-I18N-004),
 * one translator description per key (07 §7.12 rule 4), and no unused
 * catalog keys. Pure logic — imported by the `i18n:check` CLI and the
 * vitest `t_i18n_004` suite.
 */
// @ts-expect-error no @types/node in the baseline (vite.config.ts
// precedent); the scanner runs under node directly.
import { readdirSync, readFileSync, statSync } from "node:fs";
// @ts-expect-error no @types/node in the baseline.
import { join } from "node:path";

export const LOCALES = ["en", "ar", "fr", "es", "de", "pt-BR"] as const;
export type LocaleCatalogs = Record<string, Record<string, string>>;
export type Issue = string;

/** Structural issues across the catalogs (empty = the gate passes). */
export function collectIssues(
  catalogs: LocaleCatalogs,
  meta: Record<string, unknown>,
  usedIds: Set<string>,
): Issue[] {
  const issues: Issue[] = [];
  const enKeys = new Set(Object.keys(catalogs.en ?? {}));
  for (const locale of LOCALES.slice(1)) {
    const keys = new Set(Object.keys(catalogs[locale] ?? {}));
    const missing = [...enKeys].filter((k) => !keys.has(k));
    const extra = [...keys].filter((k) => !enKeys.has(k));
    if (missing.length > 0)
      issues.push(`${locale} is missing keys: ${missing.join(", ")}`);
    if (extra.length > 0)
      issues.push(`${locale} has extra keys: ${extra.join(", ")}`);
  }
  const metaKeys = new Set(Object.keys(meta));
  const withoutMeta = [...enKeys].filter((k) => !metaKeys.has(k));
  if (withoutMeta.length > 0)
    issues.push(
      `en.meta.json is missing descriptions: ${withoutMeta.join(", ")}`,
    );
  const unusedMeta = [...metaKeys].filter((k) => !enKeys.has(k));
  if (unusedMeta.length > 0)
    issues.push(
      `en.meta.json describes unknown keys: ${unusedMeta.join(", ")}`,
    );
  const unused = [...enKeys].filter((k) => !usedIds.has(k));
  if (unused.length > 0)
    issues.push(
      `unused catalog keys (no reference in app/src): ${unused.join(", ")}`,
    );
  return issues;
}

/** Scan a source tree for message IDs used in code (`id="…"` / `id: "…"`). */
export function collectUsedIds(srcRoot: string): Set<string> {
  const used = new Set<string>();
  const pattern = /\b(?:id|labelMessageId)\s*(?:=\s*"([^"]+)"|:\s*"([^"]+)")/g;
  const walk = (dir: string): void => {
    for (const entry of readdirSync(dir)) {
      const full = join(dir, entry);
      if (statSync(full).isDirectory()) {
        if (entry === "bindings" || entry === "node_modules") continue;
        walk(full);
      } else if (/\.[cm]?tsx?$/.test(entry)) {
        const source = readFileSync(full, "utf8");
        for (const match of source.matchAll(pattern)) {
          used.add(match[1] ?? match[2] ?? "");
        }
      }
    }
  };
  walk(srcRoot);
  return used;
}
