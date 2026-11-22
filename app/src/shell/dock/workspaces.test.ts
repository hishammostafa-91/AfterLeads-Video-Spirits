// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * Workspace service tests (T-APP-002 ground work): the four default
 * layouts match the 07 s7.3 table and the 07 s7.2 region defaults,
 * the materializer builds the documented structure (against a
 * recorder fake - dockview itself needs real layout), and the store
 * + persistence seam round-trip per-workspace overrides.
 */
import { beforeEach, describe, expect, it } from "vitest";
import {
  DEFAULT_LAYOUTS,
  WORKSPACE_IDS,
  captureAndPersist,
  materializeLayout,
  resetWorkspaceStoreForTests,
  setWorkspacePersistence,
  useWorkspaceStore,
  type DockLayoutApi,
} from "./workspaces";
import { localStorageWorkspacePersistence } from "../../harness/mock/workspacePort";
import { PANELS } from "../../panels/registry";

const W = 1920;
const H = 1080;

interface Recorded {
  groups: Array<{
    id: string;
    referenceGroup?: string;
    direction?: string;
    initialWidth?: number;
    initialHeight?: number;
  }>;
  panels: Array<{ id: string; group: string }>;
}

function recorderApi(): { api: DockLayoutApi; log: Recorded } {
  const log: Recorded = { groups: [], panels: [] };
  const api: DockLayoutApi = {
    width: W,
    height: H,
    clear: () => {
      log.groups = [];
      log.panels = [];
    },
    addGroup: (options) => {
      const id = `group-${String(log.groups.length + 1)}`;
      const record = options as
        | {
            referenceGroup?: string | { id: string };
            direction?: string;
            initialWidth?: number;
            initialHeight?: number;
          }
        | undefined;
      log.groups.push({
        id,
        referenceGroup:
          typeof record?.referenceGroup === "string"
            ? record.referenceGroup
            : undefined,
        direction: record?.direction,
        initialWidth: record?.initialWidth,
        initialHeight: record?.initialHeight,
      });
      return { id };
    },
    addPanel: (options) => {
      const position = options.position as
        { referenceGroup?: string | { id: string } } | undefined;
      const reference =
        typeof position?.referenceGroup === "string"
          ? position.referenceGroup
          : position?.referenceGroup?.id;
      log.panels.push({ id: options.id, group: reference ?? "" });
      return undefined;
    },
  };
  return { api, log };
}

const knownPanels = new Set(PANELS.map((panel) => panel.id));

describe("t_app_002 — default layouts (07 s7.3)", () => {
  it("every spec references registered panels only", () => {
    for (const id of WORKSPACE_IDS) {
      const spec = DEFAULT_LAYOUTS[id];
      for (const panel of [
        ...spec.center.flatMap((group) => group.panels),
        ...(spec.left?.panels ?? []),
        ...(spec.right?.panels ?? []),
        ...(spec.bottom?.flatMap((row) => row.panels) ?? []),
      ]) {
        if (!knownPanels.has(panel)) {
          throw new Error(`${id}: unknown panel "${panel}"`);
        }
      }
    }
  });

  it("every workspace docks the Program viewer and the Timeline (07 s7.3)", () => {
    for (const id of WORKSPACE_IDS) {
      const spec = DEFAULT_LAYOUTS[id];
      const center = spec.center.flatMap((group) => group.panels);
      const bottom = (spec.bottom ?? []).flatMap((row) => row.panels);
      expect(center, id).toContain("program");
      expect(bottom, id).toContain("timeline");
    }
  });

  it("WS-01 groups Media/Effects/Transitions/Titles and Inspector/History (07 s7.3)", () => {
    expect(DEFAULT_LAYOUTS["WS-01"].left?.panels).toEqual([
      "media",
      "effects",
      "transitions",
      "titles",
    ]);
    expect(DEFAULT_LAYOUTS["WS-01"].right?.panels).toEqual([
      "inspector",
      "history",
    ]);
  });

  it("WS-03 stacks the Mixer above the Timeline (07 s7.3)", () => {
    const rows = DEFAULT_LAYOUTS["WS-03"].bottom ?? [];
    expect(rows.map((row) => row.panels)).toEqual([["mixer"], ["timeline"]]);
  });
});

describe("t_app_002 — materializeLayout (07 s7.2 regions)", () => {
  it("WS-01: two center groups split the viewer region equally; docks at 22/22; timeline 45%", () => {
    const { api, log } = recorderApi();
    materializeLayout(api, DEFAULT_LAYOUTS["WS-01"]);
    // Groups in creation order: center x2, right, left, bottom.
    expect(log.groups.map((group) => group.id)).toEqual([
      "group-1",
      "group-2",
      "group-3",
      "group-4",
      "group-5",
    ]);
    // Viewers: 56% split equally. The first center group is the base
    // (it fills the viewer remainder); the second sets its 28% share
    // explicitly (07 s7.2).
    expect(log.groups[0].initialWidth).toBeUndefined();
    expect(log.groups[1].initialWidth).toBeCloseTo(0.28 * W, 0);
    expect(log.groups[1].referenceGroup).toBe("group-1");
    expect(log.groups[1].direction).toBe("right");
    // Inspector dock right at 22%, Media dock left at 22%.
    expect(log.groups[2].initialWidth).toBeCloseTo(0.22 * W, 0);
    expect(log.groups[2].direction).toBe("right");
    expect(log.groups[3].initialWidth).toBeCloseTo(0.22 * W, 0);
    expect(log.groups[3].direction).toBe("left");
    // Timeline: 45% of height, below the center (07 s7.2).
    expect(log.groups[4].initialHeight).toBeCloseTo(0.45 * H, 0);
    expect(log.groups[4].direction).toBe("below");
    // Tab membership: source | program | inspector+history | media dock | timeline.
    const tabsOf = (group: string): string[] =>
      log.panels
        .filter((panel) => panel.group === group)
        .map((panel) => panel.id);
    expect(tabsOf("group-1")).toEqual(["source"]);
    expect(tabsOf("group-2")).toEqual(["program"]);
    expect(tabsOf("group-3")).toEqual(["inspector", "history"]);
    expect(tabsOf("group-4")).toEqual([
      "media",
      "effects",
      "transitions",
      "titles",
    ]);
    expect(tabsOf("group-5")).toEqual(["timeline"]);
  });

  it("WS-03: the mixer row lands above the timeline row (07 s7.3)", () => {
    const { api, log } = recorderApi();
    materializeLayout(api, DEFAULT_LAYOUTS["WS-03"]);
    const mixer = log.groups.find((group) => group.initialHeight === 0.35 * H);
    const timeline = log.groups.find(
      (group) => group.initialHeight === 0.25 * H,
    );
    expect(mixer).toBeDefined();
    expect(timeline).toBeDefined();
    // The timeline row attaches below the mixer row.
    expect(timeline?.referenceGroup).toBe(mixer?.id);
    expect(timeline?.direction).toBe("below");
  });

  it("clears the layout before building", () => {
    const { api, log } = recorderApi();
    log.groups.push({ id: "stale" });
    materializeLayout(api, DEFAULT_LAYOUTS["WS-02"]);
    expect(log.groups.every((group) => group.id !== "stale")).toBe(true);
  });
});

describe("t_app_002 — store and persistence (03 s3.6 keys)", () => {
  beforeEach(() => {
    resetWorkspaceStoreForTests();
    localStorage.clear();
  });

  it("hydrate restores the active id and built-in overrides", () => {
    useWorkspaceStore.getState().hydrate({
      active: "WS-02",
      saved: [{ id: "WS-03", name: "WS-03", layout: {} as never }],
    });
    const state = useWorkspaceStore.getState();
    expect(state.active).toBe("WS-02");
    expect(Object.keys(state.overrides)).toEqual(["WS-03"]);
  });

  it("resetActive drops only the active override (KB-130)", () => {
    useWorkspaceStore.getState().hydrate({
      saved: [
        { id: "WS-01", name: "WS-01", layout: {} as never },
        { id: "WS-04", name: "WS-04", layout: {} as never },
      ],
    });
    useWorkspaceStore.getState().resetActive();
    const state = useWorkspaceStore.getState();
    expect(state.overrides["WS-01"]).toBeUndefined();
    expect(state.overrides["WS-04"]).toBeDefined();
  });

  it("the harness port round-trips the document (FR-006 AC2 seam)", async () => {
    const port = localStorageWorkspacePersistence(localStorage);
    await port.write({ active: "WS-03", saved: [] });
    const read = await port.read();
    expect(read).toEqual({ active: "WS-03", saved: [] });
  });

  it("setWorkspacePersistence hydrates and captureAndPersist writes overrides", async () => {
    await setWorkspacePersistence(
      localStorageWorkspacePersistence(localStorage),
    );
    expect(useWorkspaceStore.getState().active).toBe("WS-01");
    await captureAndPersist("WS-01", { grid: {} } as never);
    const doc = await localStorageWorkspacePersistence(localStorage).read();
    expect(doc?.active).toBe("WS-01");
    expect(doc?.saved?.[0]?.id).toBe("WS-01");
  });
});
