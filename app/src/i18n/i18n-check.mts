// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * The localization gate (`pnpm -C app i18n:check`; docs/08 §8.4).
 * T007 ships the first real slice: identical key sets across the six
 * shipped locales (LOC-01..06) and one translator description per key
 * in `en.meta.json` (07 §7.12 rules 2/4). T014 extends this checker
 * with ICU syntax validation and unused-key detection.
 */
// @ts-expect-error no @types/node in the baseline (vite.config.ts
// precedent); this script runs under node directly.
import { readFileSync } from "node:fs";

const LOCALES = ["en", "ar", "fr", "es", "de", "pt-BR"] as const;

const dir = new URL("../locales/", import.meta.url);
const read = (name: string): Record<string, unknown> =>
  JSON.parse(readFileSync(new URL(name, dir), "utf8")) as Record<
    string,
    unknown
  >;

let failed = false;
const fail = (message: string): void => {
  failed = true;
  console.error(`i18n:check: ${message}`);
};

const en = read("en.json");
const enKeys = new Set(Object.keys(en));

for (const locale of LOCALES) {
  if (locale === "en") continue;
  const keys = new Set(Object.keys(read(`${locale}.json`)));
  const missing = [...enKeys].filter((k) => !keys.has(k));
  const extra = [...keys].filter((k) => !enKeys.has(k));
  if (missing.length > 0)
    fail(`${locale} is missing keys: ${missing.join(", ")}`);
  if (extra.length > 0) fail(`${locale} has extra keys: ${extra.join(", ")}`);
}

const meta = read("en.meta.json");
const metaKeys = new Set(Object.keys(meta));
const withoutMeta = [...enKeys].filter((k) => !metaKeys.has(k));
if (withoutMeta.length > 0) {
  fail(`en.meta.json is missing descriptions: ${withoutMeta.join(", ")}`);
}
const unusedMeta = [...metaKeys].filter((k) => !enKeys.has(k));
if (unusedMeta.length > 0) {
  fail(`en.meta.json describes unknown keys: ${unusedMeta.join(", ")}`);
}

if (failed) {
  // @ts-expect-error no @types/node in the baseline.
  process.exitCode = 1;
} else {
  console.log(
    `i18n:check: ${enKeys.size} keys identical across ${LOCALES.length} locales; meta complete.`,
  );
}
