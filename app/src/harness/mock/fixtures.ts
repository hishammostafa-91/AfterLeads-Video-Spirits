// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * Mock backend skeleton (card Deliverables; research D11): typed
 * fixture factories over the WP-02 `src/ipc/bindings` shapes plus a
 * stub event bus — story data only, no IPC calls (WP-05 owns the real
 * command layer; job fixtures arrive with the package that owns the
 * jobs model).
 */
import type { MediaItem } from "../../ipc/bindings/MediaItem";

let fixtureSeq = 0;
const nextId = (prefix: string): string =>
  `${prefix}-fixture-${String((fixtureSeq += 1))}`;

/** A minimal valid media item fixture (03 §3.4.2 shape). */
export function mediaFixture(overrides: Partial<MediaItem> = {}): MediaItem {
  return {
    id: nextId("media"),
    kind: "video",
    name: "Interview A.mp4",
    path: "C:/media/interview-a.mp4",
    relPath: "interview-a.mp4",
    fingerprint: {
      size: 1,
      mtimeUtc: "2026-01-01T00:00:00Z",
      headTailSha256: "aa",
    },
    probe: { durationFlicks: 10 * 705_600_000, container: "mp4", streams: [] },
    imageSequence: null,
    binId: null,
    label: null,
    notes: "",
    derived: { optimized: null, stabilized: null },
    addedUtc: "2026-01-01T00:00:00Z",
    ...overrides,
  };
}

/** Stub event bus: components subscribe; stories publish. */
export type HarnessEvent = { topic: string; payload: unknown };
export function createEventBus(): {
  publish: (event: HarnessEvent) => void;
  subscribe: (listener: (event: HarnessEvent) => void) => () => void;
} {
  const listeners = new Set<(event: HarnessEvent) => void>();
  return {
    publish: (event) => {
      for (const listener of listeners) listener(event);
    },
    subscribe: (listener) => {
      listeners.add(listener);
      return () => {
        listeners.delete(listener);
      };
    },
  };
}
