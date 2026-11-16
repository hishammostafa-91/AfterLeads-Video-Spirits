// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * The `pnpm -C app i18n:check` CLI (docs/08 §8.4): runs the structural
 * core in `i18n-check-core.mts` over the real catalogs and validates
 * ICU syntax with the formatjs parser (T014). The vitest `t_i18n_004`
 * suite additionally exercises the checker's failure modes and the
 * pseudo-locales; both gates run locally and in CI.
 */
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { join } from "node:path";
import { createRequire } from "node:module";
import { collectIssues, collectUsedIds, LOCALES } from "./i18n-check-core.mts";

// ICU syntax validation (docs/08 s8.4; T014) uses the same parser
// react-intl/eslint-plugin-formatjs run (the formatjs toolchain is a
// declared dependency; the parser itself is only installed as its
// transitive, so it resolves through the plugin's scope - pnpm's
// layout keeps it there, not in app/node_modules).
const nodeRequire = createRequire(import.meta.url) as unknown as {
  resolve: (id: string) => string;
};
// Resolve the plugin's main entry (its package.json subpath is not
// exported); createRequire from inside the package then reaches its
// transitive parser in pnpm's store scope.
const pluginEntry = nodeRequire.resolve("eslint-plugin-formatjs");
const pluginRequire = createRequire(pluginEntry) as unknown as (id: string) => {
  parse: (message: string) => unknown;
};
const { parse } = pluginRequire("@formatjs/icu-messageformat-parser");

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
const meta = read("en.meta.json");

// ICU syntax validation (docs/08 s8.4; T014): every message must parse.
const icuIssues: string[] = [];
for (const locale of LOCALES) {
  const catalog = catalogs[locale];
  for (const [key, value] of Object.entries(catalog)) {
    if (typeof value !== "string") continue;
    try {
      parse(value);
    } catch (error) {
      icuIssues.push(`${locale}: invalid ICU in ${key}: ${String(error)}`);
    }
  }
}

const issues = [
  ...icuIssues,
  ...collectIssues(catalogs, meta, collectUsedIds(srcRoot)),
];

if (issues.length > 0) {
  for (const issue of issues) console.error(`i18n:check: ${issue}`);
  process.exitCode = 1;
} else {
  console.log(
    `i18n:check: ${String(Object.keys(catalogs.en).length)} keys identical across ${String(LOCALES.length)} locales; meta complete; no unused keys.`,
  );
}
