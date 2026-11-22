// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * The localization gate's structural core (docs/08 §8.4; T007/T014):
 * identical key sets across the six shipped locales (REQ-I18N-004),
 * one translator description per key (07 §7.12 rule 4), and no unused
 * catalog keys. Pure logic — imported by the `i18n:check` CLI and the
 * vitest `t_i18n_004` suite.
 */
import { readdirSync, readFileSync, statSync } from "node:fs";
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
  const enKeys = new Set(Object.keys(catalogs.en));
  for (const locale of LOCALES.slice(1)) {
    const keys = new Set(Object.keys(catalogs[locale]));
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
  // Namespaced message IDs referenced in code must exist in the
  // source catalog — story-grid ids (no namespace) are not message
  // IDs and are ignored by the prefix filter.
  const missingUsed = [...usedIds].filter(
    (k) => /^(components|harness|a11y)\./.test(k) && !enKeys.has(k),
  );
  if (missingUsed.length > 0)
    issues.push(
      `catalog keys used in code but missing from en.json: ${missingUsed.join(", ")}`,
    );
  return issues;
}

/** Scan a source tree for message IDs used in code (`id="…"` JSX
 * props and any `*MessageId`/`messageId`/`titleKey` literals in
 * props or objects). The command catalog's kb.* label keys are
 * constructed dynamically (kb.<id> in commands/catalog.ts), so the
 * doc-derived catalog fixture is read directly and its ids counted
 * as used; the panel registry's titleKey literals
 * (panels.<slug>.title, 07 s7.7) match the same pattern. */
export function collectUsedIds(srcRoot: string): Set<string> {
  const used = new Set<string>();
  const pattern =
    /\b(?:id|titleKey|(?:[a-zA-Z]+)?[Mm]essageId)\s*(?:=\s*"([^"]+)"|:\s*"([^"]+)")/g;
  const walk = (dir: string): void => {
    for (const entry of readdirSync(dir)) {
      const full = join(dir, entry);
      if (statSync(full).isDirectory()) {
        if (entry === "bindings" || entry === "node_modules") continue;
        walk(full);
      } else if (entry === "catalog.fixture.json") {
        // The command label keys are built dynamically; the fixture is
        // the doc-derived source of those ids (12 s12.10).
        const fixture = JSON.parse(
          readFileSync(full, "utf8"),
        ) as ReadonlyArray<{ id?: unknown }>;
        for (const row of fixture) {
          if (typeof row.id === "string") used.add(`kb.${row.id}`);
        }
      } else if (entry === "presets.sqp.fixture.json") {
        // Sequence preset names resolve via presetLabelKey (12 s12.8).
        const fixture = JSON.parse(
          readFileSync(full, "utf8"),
        ) as ReadonlyArray<{ id?: unknown }>;
        for (const row of fixture) {
          if (typeof row.id === "string") used.add(`presets.sqp.${row.id}`);
        }
      } else if (entry === "presets.fps.fixture.json") {
        // Frame-rate names resolve via frameRateLabelKey (12 s12.9).
        const fixture = JSON.parse(
          readFileSync(full, "utf8"),
        ) as ReadonlyArray<{ id?: unknown }>;
        for (const row of fixture) {
          if (typeof row.id === "string") used.add(`presets.fps.${row.id}`);
        }
      } else if (/\.[cm]?tsx?$/.test(entry)) {
        const source = readFileSync(full, "utf8");
        for (const match of source.matchAll(pattern)) {
          // The regex tuple types both capture groups as string, but
          // at most one is present at runtime - read them as optional.
          const groups = match as ReadonlyArray<string | undefined>;
          used.add(groups[1] ?? groups[2] ?? "");
        }
        // String literals returned through helper functions (e.g.
        // the DLG-01 name-rule message ids) count as usage too.
        // Test files are excluded: their fiction keys exist to prove
        // the checker itself (i18n-check.test.ts).
        if (!/\.test\.[cm]?tsx?$/.test(entry)) {
          for (const match of source.matchAll(
            /"((?:[a-z]+\.){1,3}[A-Za-z0-9.-]+)"/g,
          )) {
            used.add(match[1]);
          }
        }
      }
    }
  };
  walk(srcRoot);
  return used;
}
