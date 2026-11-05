// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * The `pnpm -C app i18n:check` CLI (docs/08 §8.4): runs the structural
 * core in `i18n-check-core.mts` over the real catalogs. ICU syntax
 * validation rides the vitest `t_i18n_004` suite (react-intl parses
 * every message), so the full §8.4 contract is enforced across the two
 * gates that both run locally and in CI.
 */
// @ts-expect-error no @types/node in the baseline (vite.config.ts
// precedent); this script runs under node directly.
import { readFileSync } from "node:fs";
// @ts-expect-error no @types/node in the baseline.
import { fileURLToPath } from "node:url";
// @ts-expect-error no @types/node in the baseline.
import { join } from "node:path";
import { collectIssues, collectUsedIds, LOCALES } from "./i18n-check-core.mts";

const localesDir = fileURLToPath(new URL("../locales/", import.meta.url));
const srcRoot = fileURLToPath(new URL("../", import.meta.url));
const read = (name: string): Record<string, unknown> =>
  JSON.parse(readFileSync(join(localesDir, name), "utf8")) as Record<
    string,
    unknown
  >;

const catalogs = Object.fromEntries(
  LOCALES.map((locale) => [locale, read(`${locale}.json`)]),
) as Record<string, Record<string, string>>;
const meta = read("en.meta.json") as Record<string, unknown>;
const issues = collectIssues(catalogs, meta, collectUsedIds(srcRoot));

if (issues.length > 0) {
  for (const issue of issues) console.error(`i18n:check: ${issue}`);
  // @ts-expect-error no @types/node in the baseline.
  process.exitCode = 1;
} else {
  console.log(
    `i18n:check: ${Object.keys(catalogs.en ?? {}).length} keys identical across ${LOCALES.length} locales; meta complete; no unused keys.`,
  );
}
