// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
// @vitest-environment node
/**
 * Token hygiene (round-2 M-01 companion): every custom property
 * referenced through a CSS var() reference in the shipped CSS and
 * TSX must be defined in shipped CSS. Undefined references resolve
 * silently — a var() whose custom property does not exist makes the
 * declaration invalid at computed-value time and the property falls
 * back to its initial value, which is how the shadow and overlay
 * tokens shipped with no elevation and transparent overlay surfaces
 * (round-2 findings).
 */
// @ts-expect-error no @types/node in the baseline (tokens.test.ts
// precedent).
import { readdirSync, readFileSync, statSync } from "node:fs";
// @ts-expect-error no @types/node in the baseline.
import { join } from "node:path";
// @ts-expect-error no @types/node in the baseline.
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";

const SRC = fileURLToPath(new URL("../", import.meta.url));

const DEFINING = /(--[a-zA-Z0-9-]+)\s*:/g;
const REFERENCING = /var\((--[a-zA-Z0-9-]+)/g;

const defined = new Set<string>();
const referenced = new Map<string, string[]>();

const walk = (dir: string): void => {
  for (const entry of readdirSync(dir)) {
    const full = join(dir, entry);
    if (statSync(full).isDirectory()) {
      if (entry === "bindings" || entry === "node_modules") continue;
      walk(full);
    } else if (/\.(css|tsx?|mts)$/.test(entry)) {
      const rel = full.slice(SRC.length);
      const lines = readFileSync(full, "utf8").split("\n");
      lines.forEach((line, idx) => {
        for (const match of line.matchAll(DEFINING)) {
          defined.add(match[1]);
        }
        for (const match of line.matchAll(REFERENCING)) {
          const sites = referenced.get(match[1]) ?? [];
          sites.push(`${rel}:${idx + 1}`);
          referenced.set(match[1], sites);
        }
      });
    }
  }
};
walk(SRC);

describe("token hygiene (FR-001: components reference tokens only)", () => {
  it("every referenced custom property is defined in shipped CSS", () => {
    const undefinedRefs = [...referenced.keys()]
      .filter((name) => !defined.has(name))
      .sort()
      .map((name) => `${name} <- ${(referenced.get(name) ?? []).join(", ")}`);
    expect(undefinedRefs).toEqual([]);
  });

  it("the scanner itself sees the sheet (sanity)", () => {
    // Guards against a walk bug making the first test vacuous.
    expect(defined.size).toBeGreaterThan(50);
    expect(referenced.has("--accent-solid")).toBe(true);
    expect(referenced.has("--bg-app")).toBe(true);
  });
});
