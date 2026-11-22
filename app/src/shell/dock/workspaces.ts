// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * The workspace service (REQ-APP-003 built-ins; 07 s7.3): the four
 * default layouts as declarative specs, a materializer that builds a
 * dockview layout from a spec, per-workspace layout persistence
 * (REQ-APP-002) through the documented settings keys
 * `alvs.workspaces.active` / `alvs.workspaces.saved` (03 s3.6), and
 * the zustand store the shell reads.
 *
 * Storage shape: `alvs.workspaces.saved` is the only layout-valued
 * key, so per-workspace edits of the built-ins persist as entries
 * with the built-in ids (`WS-01`..`WS-04`); WP-24's custom
 * workspaces extend the same array with user ids (03 s3.6 "array of
 * { id, name, layout }").
 *
 * Sizing: 07 s7.2 fixes the Edit regions (left 22% / viewers 56% /
 * Inspector 22% of width; upper 55% / timeline 45% of height) and
 * 07 s7.3 fixes WS-02's timeline at 30% and WS-03's mixer at 35%
 * (of window height). The remaining unspecified sizes keep the
 * documented Edit analogues and say so in the spec comments.
 */
import { create } from "zustand";
import type {
  AddGroupOptions,
  AddPanelOptions,
  SerializedDockview,
} from "dockview-react";
import { PANEL_BY_ID } from "../../panels/registry";

export type WorkspaceId = "WS-01" | "WS-02" | "WS-03" | "WS-04";

export const WORKSPACE_IDS: readonly WorkspaceId[] = [
  "WS-01",
  "WS-02",
  "WS-03",
  "WS-04",
];

export const WORKSPACE_LABEL_KEY: Readonly<Record<WorkspaceId, string>> = {
  "WS-01": "kb.view.workspaceEdit",
  "WS-02": "kb.view.workspaceColor",
  "WS-03": "kb.view.workspaceAudio",
  "WS-04": "kb.view.workspaceCaptions",
};

/** One saved-workspaces entry (03 s3.6). */
export interface SavedWorkspaceEntry {
  id: string;
  name: string;
  layout: SerializedDockview;
}

/** The document the persistence seam reads and writes. */
export interface WorkspaceDoc {
  active?: WorkspaceId;
  saved?: SavedWorkspaceEntry[];
}

/**
 * The persistence seam (the theme seam pattern, research D12): the
 * noop default keeps state in memory; the harness binds a
 * localStorage port so restart round-trips are testable, and the
 * settings IPC port (WP-24; keys per 03 s3.6) binds the same
 * interface in the packaged app.
 */
export interface WorkspacePersistence {
  read(): Promise<WorkspaceDoc | null>;
  write(doc: WorkspaceDoc): Promise<void>;
}

const noopPersistence: WorkspacePersistence = {
  read: () => Promise.resolve(null),
  write: () => Promise.resolve(),
};

let workspacePersistence: WorkspacePersistence = noopPersistence;

/** Bind a persistence backend (harness: localStorage; WP-24: settings). */
export function setWorkspacePersistence(
  persistence: WorkspacePersistence,
): Promise<void> {
  workspacePersistence = persistence;
  return hydrateFromPersistence();
}

/** Test helper: detach persistence and reset to defaults. */
export function resetWorkspaceStoreForTests(): void {
  workspacePersistence = noopPersistence;
  useWorkspaceStore.setState({ active: "WS-01", overrides: {} });
}

/** A tabbed group of panels; `size` is a fraction of the container. */
export interface DockGroupSpec {
  panels: readonly string[];
  /** Fraction of the layout width (side docks, center split). */
  widthFraction?: number;
  /** Fraction of the layout height (bottom rows). */
  heightFraction?: number;
}

/** A declarative workspace layout (07 s7.3 table column order). */
export interface WorkspaceLayoutSpec {
  /** Center groups left-to-right, side by side. */
  center: readonly DockGroupSpec[];
  /** The left tabbed dock. */
  left?: DockGroupSpec;
  /** The right tabbed dock. */
  right?: DockGroupSpec;
  /** Bottom rows top-to-bottom (e.g. WS-03: Mixer above Timeline). */
  bottom?: readonly DockGroupSpec[];
}

export const DEFAULT_LAYOUTS: Readonly<
  Record<WorkspaceId, WorkspaceLayoutSpec>
> = {
  // 07 s7.3 WS-01 + the 07 s7.2 region defaults.
  "WS-01": {
    left: {
      panels: ["media", "effects", "transitions", "titles"],
      widthFraction: 0.22,
    },
    center: [
      { panels: ["source"], widthFraction: 0.28 },
      { panels: ["program"], widthFraction: 0.28 },
    ],
    right: {
      panels: ["inspector", "history"],
      widthFraction: 0.22,
    },
    bottom: [{ panels: ["timeline"], heightFraction: 0.45 }],
  },
  // 07 s7.3 WS-02: timeline reduced to 30% height.
  "WS-02": {
    left: { panels: ["scopes"], widthFraction: 0.24 },
    center: [{ panels: ["program"] }],
    right: { panels: ["inspector"], widthFraction: 0.24 },
    bottom: [{ panels: ["timeline"], heightFraction: 0.3 }],
  },
  // 07 s7.3 WS-03: Mixer (35%) above Timeline; left dock = Media +
  // Effects (audio category arrives with WP-14/WP-18 content).
  "WS-03": {
    left: { panels: ["media", "effects"], widthFraction: 0.22 },
    center: [{ panels: ["program"] }],
    right: { panels: ["inspector"], widthFraction: 0.22 },
    bottom: [
      { panels: ["mixer"], heightFraction: 0.35 },
      { panels: ["timeline"], heightFraction: 0.25 },
    ],
  },
  // 07 s7.3 WS-04: Captions (wide) + Transcript left; timeline at the
  // Edit default (07 gives no WS-04 height).
  "WS-04": {
    left: { panels: ["captions", "transcript"], widthFraction: 0.3 },
    center: [{ panels: ["program"] }],
    right: { panels: ["inspector"], widthFraction: 0.22 },
    bottom: [{ panels: ["timeline"], heightFraction: 0.45 }],
  },
};

/**
 * The narrow slice of `DockviewApi` the materializer uses; a recorder
 * fake implements it in the unit tests (dockview itself needs real
 * layout, which jsdom lacks). The option types are dockview's own.
 */
export interface DockLayoutApi {
  clear(): void;
  readonly width: number;
  readonly height: number;
  addGroup(options?: AddGroupOptions): { id: string };
  addPanel(options: AddPanelOptions): unknown;
}

function requirePanel(id: string): string {
  const panel = PANEL_BY_ID.get(id);
  if (!panel) throw new Error(`layout: unknown panel "${id}"`);
  // The tab title is the panel slug; PanelTab renders the localized
  // title from the registry (locale switches re-render, stored
  // layouts never bake in a language).
  return panel.id;
}

/**
 * Build the layout for `id` in `api`: center groups first, then the
 * side docks relative to the outermost center groups, then the bottom
 * rows appended downward. Every panel lands in a tabbed group
 * (dock/tab operations per FR-006 act on the result).
 */
export function materializeLayout(
  api: DockLayoutApi,
  spec: WorkspaceLayoutSpec,
): void {
  api.clear();
  const w = api.width;
  const h = api.height;

  const centerGroupIds: string[] = [];
  for (const group of spec.center) {
    const created =
      centerGroupIds.length === 0
        ? api.addGroup()
        : api.addGroup({
            referenceGroup: centerGroupIds[centerGroupIds.length - 1],
            direction: "right",
            initialWidth:
              group.widthFraction !== undefined
                ? group.widthFraction * w
                : undefined,
          });
    for (const panelId of group.panels) {
      api.addPanel({
        id: panelId,
        title: requirePanel(panelId),
        component: "panel",
        position: { referenceGroup: created.id },
      });
    }
    centerGroupIds.push(created.id);
  }
  const leftCenter = centerGroupIds[0];
  const rightCenter = centerGroupIds[centerGroupIds.length - 1];

  if (spec.right) {
    const created = api.addGroup({
      referenceGroup: rightCenter,
      direction: "right",
      initialWidth:
        spec.right.widthFraction !== undefined
          ? spec.right.widthFraction * w
          : undefined,
    });
    for (const panelId of spec.right.panels) {
      api.addPanel({
        id: panelId,
        title: requirePanel(panelId),
        component: "panel",
        position: { referenceGroup: created.id },
      });
    }
  }

  if (spec.left) {
    const created = api.addGroup({
      referenceGroup: leftCenter,
      direction: "left",
      initialWidth:
        spec.left.widthFraction !== undefined
          ? spec.left.widthFraction * w
          : undefined,
    });
    for (const panelId of spec.left.panels) {
      api.addPanel({
        id: panelId,
        title: requirePanel(panelId),
        component: "panel",
        position: { referenceGroup: created.id },
      });
    }
  }

  let previousRowId: string | null = null;
  for (const row of spec.bottom ?? []) {
    const created = api.addGroup({
      referenceGroup: previousRowId ?? leftCenter,
      direction: "below",
      initialHeight:
        row.heightFraction !== undefined ? row.heightFraction * h : undefined,
    });
    for (const panelId of row.panels) {
      api.addPanel({
        id: panelId,
        title: requirePanel(panelId),
        component: "panel",
        position: { referenceGroup: created.id },
      });
    }
    previousRowId = created.id;
  }
}

interface WorkspaceState {
  active: WorkspaceId;
  /** Layout overrides per workspace id (null = the default spec). */
  overrides: Partial<Record<WorkspaceId, SerializedDockview>>;
  /** Hydrate from the persistence seam (App boot). */
  hydrate: (doc: WorkspaceDoc) => void;
  /** Switch the active workspace (capture happens in the Dock host). */
  setActive: (id: WorkspaceId) => void;
  /** Store the current layout of `id` (a dockview `toJSON()` result). */
  capture: (id: WorkspaceId, layout: SerializedDockview) => void;
  /** Drop the active workspace's override ("Reset workspace"). */
  resetActive: () => void;
}

export const useWorkspaceStore = create<WorkspaceState>((set) => ({
  active: "WS-01",
  overrides: {},
  hydrate: (doc) => {
    const overrides: Partial<Record<WorkspaceId, SerializedDockview>> = {};
    for (const entry of doc.saved ?? []) {
      if ((WORKSPACE_IDS as readonly string[]).includes(entry.id)) {
        overrides[entry.id as WorkspaceId] = entry.layout;
      }
    }
    set({
      active: doc.active ?? "WS-01",
      overrides,
    });
  },
  setActive: (id) => {
    set({ active: id });
  },
  capture: (id, layout) => {
    set((state) => ({ overrides: { ...state.overrides, [id]: layout } }));
  },
  resetActive: () => {
    set((state) => {
      const overrides = Object.fromEntries(
        Object.entries(state.overrides).filter(([id]) => id !== state.active),
      ) as Partial<Record<WorkspaceId, SerializedDockview>>;
      return { overrides };
    });
  },
}));

function persist(): Promise<void> {
  const { active, overrides } = useWorkspaceStore.getState();
  const saved: SavedWorkspaceEntry[] = (Object.keys(overrides) as WorkspaceId[])
    .filter((id) => overrides[id] !== undefined)
    .map((id) => ({
      id,
      name: id,
      layout: overrides[id] as SerializedDockview,
    }));
  return workspacePersistence.write({ active, saved });
}

let hydrating = false;
async function hydrateFromPersistence(): Promise<void> {
  hydrating = true;
  try {
    const doc = await workspacePersistence.read();
    if (doc) useWorkspaceStore.getState().hydrate(doc);
  } finally {
    hydrating = false;
  }
}

/**
 * Store the current layout of `id` and persist (03 s3.6: writes are
 * debounced 500 ms by the caller that observes layout changes).
 */
export function captureAndPersist(
  id: WorkspaceId,
  layout: SerializedDockview,
): Promise<void> {
  useWorkspaceStore.getState().capture(id, layout);
  if (hydrating) return Promise.resolve();
  return persist();
}

/** Persist the active id (workspace switches). */
export function persistActive(): Promise<void> {
  return persist();
}

/** Test helper: whether a hydration pass is in flight. */
export function isHydrating(): boolean {
  return hydrating;
}
