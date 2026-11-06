// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * Font-bundle integrity (docs/12 §12.16; research D9/F10): re-hash
 * every bundled face against `assets/fonts/fonts.json` and assert
 * each family's OFL license file ships beside it. Runs in the plain
 * `pnpm -C app test` gate.
 */
// @vitest-environment node
// @ts-expect-error no @types/node in the baseline (vite.config.ts
// precedent); the integrity check runs under node directly.
import { createHash } from "node:crypto";
// @ts-expect-error no @types/node in the baseline.
import { existsSync, readFileSync } from "node:fs";
// @ts-expect-error no @types/node in the baseline.
import { dirname, join, resolve } from "node:path";
// @ts-expect-error no @types/node in the baseline.
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";

interface FontEntry {
  id: string;
  family: string;
  file: string;
  sourceUrl: string;
  sha256: string;
  licenseFile: string;
}

const FONTS_DIR = resolve(
  dirname(fileURLToPath(import.meta.url)),
  "../../../assets/fonts",
);

const manifest: { families: FontEntry[] } = JSON.parse(
  readFileSync(join(FONTS_DIR, "fonts.json"), "utf8"),
) as { families: FontEntry[] };

describe("font bundle integrity (12 s12.16)", () => {
  it("records the three families FNT-01..03", () => {
    expect(manifest.families.map((f) => f.id)).toEqual([
      "FNT-01",
      "FNT-02",
      "FNT-03",
    ]);
  });

  for (const family of manifest.families) {
    it(`${family.id} (${family.family}) hash matches and license ships`, () => {
      const fontPath = join(FONTS_DIR, family.file);
      expect(existsSync(fontPath)).toBe(true);
      const hash = createHash("sha256")
        .update(readFileSync(fontPath))
        .digest("hex");
      expect(hash).toBe(family.sha256);
      expect(existsSync(join(FONTS_DIR, family.licenseFile))).toBe(true);
    });
  }
});
