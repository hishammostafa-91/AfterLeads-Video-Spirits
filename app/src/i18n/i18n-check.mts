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

// ICU syntax validation (docs/08 s8.4; T014) parses with the exact
// parser version the react-intl runtime formats messages with:
// react-intl is a declared dependency and depends on
// @formatjs/icu-messageformat-parser, so resolving through its scope
// pins the checker to the production parser without adding an
// undeclared transitive to the baseline (G-11; round-2 L-04 — this
// previously rode eslint-plugin-formatjs's private dependency).
const requireFromReactIntl = createRequire(
  createRequire(import.meta.url).resolve("react-intl"),
);
const { parse } = requireFromReactIntl(
  "@formatjs/icu-messageformat-parser",
) as {
  parse: (message: string) => unknown;
};

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
