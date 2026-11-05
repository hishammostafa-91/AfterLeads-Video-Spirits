// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * T-I18N-004 (09 §9.10): key sets identical across all six locales
 * (the check fails on missing or extra keys), no unused keys, and
 * every message's ICU syntax is valid (react-intl parses each catalog
 * entry with placeholder values — REQ-I18N-004; docs/08 §8.4).
 */
import { createIntl } from "react-intl";
import { describe, expect, it } from "vitest";
import {
  collectIssues,
  LOCALES,
  type LocaleCatalogs,
} from "./i18n-check-core.mts";
import metaJson from "../locales/en.meta.json";
import { MESSAGES } from "./runtime";

const catalogs = MESSAGES as unknown as LocaleCatalogs;
const realMeta = metaJson as unknown as Record<string, unknown>;

/** Placeholder names used by a message ({count}, {value}, …). */
function placeholders(message: string): Set<string> {
  const names = new Set<string>();
  for (const match of message.matchAll(/\{(\w+)(?:\s*,|\})/g)) {
    if (!/^(plural|select|selectordinal|number|date|time)$/.test(match[1])) {
      names.add(match[1]);
    }
  }
  return names;
}

describe("t_i18n_004 key-set discipline (REQ-I18N-004)", () => {
  it("passes on the real catalogs", () => {
    const used = new Set(Object.keys(catalogs.en));
    expect(collectIssues(catalogs, realMeta, used)).toEqual([]);
  });

  it("fails on a locale missing a key", () => {
    const broken: LocaleCatalogs = {
      ...catalogs,
      fr: Object.fromEntries(
        Object.entries(catalogs.fr).filter(([k]) => k !== "harness.title"),
      ),
    };
    const issues = collectIssues(
      broken,
      realMeta,
      new Set(Object.keys(catalogs.en)),
    );
    expect(issues.join("\n")).toMatch(/fr is missing keys: harness\.title/);
  });

  it("fails on an extra key", () => {
    const broken: LocaleCatalogs = {
      ...catalogs,
      de: { ...catalogs.de, "harness.future.key": "x" },
    };
    const issues = collectIssues(
      broken,
      realMeta,
      new Set(Object.keys(catalogs.en)),
    );
    expect(issues.join("\n")).toMatch(
      /de has extra keys: harness\.future\.key/,
    );
  });

  it("fails on an unused key", () => {
    const issues = collectIssues(
      catalogs,
      realMeta,
      new Set(["harness.title"]),
    );
    expect(issues.join("\n")).toMatch(/unused catalog keys/);
  });
});

describe("t_i18n_004 ICU syntax (docs/08 s8.4)", () => {
  /** createIntl with a throwing onError: syntax errors must surface. */
  const strictIntl = (locale: string, messages: Record<string, string>) =>
    createIntl({
      locale,
      messages,
      onError: (error: unknown) => {
        throw error;
      },
    });

  it.each(LOCALES)("locale %s: every message parses", (locale) => {
    const intl = strictIntl(locale, catalogs[locale] ?? {});
    for (const [id, message] of Object.entries(catalogs[locale] ?? {})) {
      const values = Object.fromEntries(
        [...placeholders(message)].map((name) => [name, 2]),
      );
      expect(() => intl.formatMessage({ id }, values), id).not.toThrow();
    }
  });

  it("rejects invalid ICU syntax", () => {
    const intl = strictIntl("en", {
      broken: "{count, plural, one {# clip}",
    });
    expect(() => intl.formatMessage({ id: "broken" }, { count: 1 })).toThrow();
  });
});
