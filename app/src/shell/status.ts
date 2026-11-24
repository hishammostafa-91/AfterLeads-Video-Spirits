// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * The status-bar live state (FR-005): the renderer name from
 * `app_info`, the autosave chip from `autosave:status`, and the
 * job-summary count from `jobs:update` summaries. Sources that ship
 * later (playback stats, proxies) stay neutral.
 */
import { create } from "zustand";
import type { AppInfo } from "../ipc/bindings/AppInfo";

/** The `autosave:status` payload (04 s4.10). */
export interface AutosaveStatus {
  state: "idle" | "saving" | "saved" | "failed";
  timeUtc: string | null;
}

/** One `jobs:update` summary row (04 s4.3.9's JobInfo shape is
 * WP-07's; the shell tracks only the count for the chip). */
export interface JobsSummary {
  running: number;
}

interface StatusState {
  renderer: string | null;
  autosave: AutosaveStatus | null;
  jobs: JobsSummary | null;
  setRendererFromInfo: (info: AppInfo) => void;
  setAutosave: (status: AutosaveStatus) => void;
  setJobs: (summary: JobsSummary) => void;
}

export const useStatusStore = create<StatusState>((set) => ({
  renderer: null,
  autosave: null,
  jobs: null,
  setRendererFromInfo: (info) => {
    // The renderer name as `app_info` reports it ("unknown" until
    // WP-09; the chip renders the neutral dash for null only).
    set({ renderer: info.renderer.name });
  },
  setAutosave: (autosave) => {
    set({ autosave });
  },
  setJobs: (jobs) => {
    set({ jobs });
  },
}));

/** Test helper. */
export function resetStatusStoreForTests(): void {
  useStatusStore.setState({
    renderer: null,
    autosave: null,
    jobs: null,
  });
}
