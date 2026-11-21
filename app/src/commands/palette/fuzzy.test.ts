// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * Fuzzy matcher coverage: substring over subsequence, position and
 * tightness scoring, accent/case folding, Arabic as-is matching, and
 * stable KB-order tie-breaks.
 */
import { describe, expect, it } from "vitest";
import { fuzzyRank, fuzzyScore, normalize } from "./fuzzy";

describe("command palette fuzzy matching (REQ-APP-004)", () => {
  it("empty query matches everything at score 0", () => {
    expect(fuzzyScore("", "anything")).toBe(0);
    expect(fuzzyScore("   ", "anything")).toBe(0);
  });

  it("substring beats subsequence; earlier and tighter wins", () => {
    const sub = fuzzyScore("save", "Save");
    const seq = fuzzyScore("sve", "Save");
    expect(sub).not.toBeNull();
    expect(seq).not.toBeNull();
    expect(sub as number).toBeGreaterThan(seq as number);
    expect(fuzzyScore("save", "Save Project")).toBeLessThan(
      fuzzyScore("save", "Save") as number,
    );
    expect(fuzzyScore("pro", "Save Project")).toBeGreaterThan(
      fuzzyScore("ject", "Save Project") as number,
    );
  });

  it("subsequence requires all characters in order", () => {
    expect(fuzzyScore("fnp", "file.newProject")).not.toBeNull();
    expect(fuzzyScore("pnf", "file.newProject")).toBeNull();
    expect(fuzzyScore("zzz", "file.newProject")).toBeNull();
  });

  it("case and Latin accents fold; Arabic matches as-is", () => {
    expect(fuzzyScore("ENREGISTRER", "Enregistrer")).not.toBeNull();
    expect(fuzzyScore("enregistrer sous", "Enregistrer sous…")).not.toBeNull();
    expect(normalize("Éditer")).toBe("editer");
    expect(fuzzyScore("editer", "Éditer")).not.toBeNull();
    expect(fuzzyScore("مشروع", "مشروع جديد…")).not.toBeNull();
    expect(fuzzyScore("حفظ", "حفظ باسم…")).not.toBeNull();
  });

  it("ranks by score then stable order", () => {
    const ranked = fuzzyRank("zoo", [
      { haystack: "Zoom In", order: 2, item: "zoomIn" },
      { haystack: "Zoom Out", order: 3, item: "zoomOut" },
      { haystack: "Zoom to Fit", order: 4, item: "zoomFit" },
      { haystack: "Save", order: 1, item: "save" },
    ]);
    expect(ranked.map((entry) => entry.item)).toEqual([
      "zoomIn",
      "zoomOut",
      "zoomFit",
    ]);
    const tie = fuzzyRank("", [
      { haystack: "b", order: 2, item: "b" },
      { haystack: "a", order: 1, item: "a" },
    ]);
    expect(tie.map((entry) => entry.item)).toEqual(["a", "b"]);
  });
});
