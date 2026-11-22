// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * The mock IPC transport (ADR-020's mock backend): answers every
 * implemented command (04 s4.3.1 app commands + s4.3.2 project
 * commands) with fixture data and records the calls - story and
 * test data only, no Tauri shell. The project side keeps one
 * open project in memory (FR-001) with the same validation rules
 * as the backend (names, presets, fps, sample rates).
 */
import type { IpcTransport } from "../../ipc/transport";
import type { AppInfo } from "../../ipc/bindings/AppInfo";
import type { Project } from "../../ipc/bindings/Project";
import type { ProjectSnapshot } from "../../ipc/bindings/ProjectSnapshot";
import type { RecentProject } from "../../ipc/bindings/RecentProject";
import type { NewProjectRequest } from "../../ipc/bindings/NewProjectRequest";
import {
  DEFAULT_SAMPLE_RATE,
  frameRateById,
  presetById,
} from "../../projects/presets";

/** Fixture `app_info` - mock values, the honest unknown renderer. */
export function appInfoFixture(): AppInfo {
  return {
    version: "0.1.0",
    commit: "mock",
    buildDate: "2026-01-01",
    identifier: "com.afterleads.videospirits",
    renderer: { name: "unknown", isWarp: false },
    safeMode: false,
    portable: false,
    locale: "en",
  };
}

/** A minimal valid Project fixture (03 s3.4.1 shape). */
export function projectFixture(name: string): Project {
  const sequenceId = "seq-fixture-main";
  return {
    format: "afterleads-video-spirits.project",
    schemaVersion: 1,
    id: "project-fixture",
    name,
    createdUtc: "2026-01-01T00:00:00Z",
    modifiedUtc: "2026-01-01T00:00:00Z",
    savedBy: {
      app: "AfterLeads-Video-Spirits",
      version: "0.1.0",
      commit: "mock",
    },
    settings: { defaultSequenceId: sequenceId, proxyMode: false },
    media: [],
    bins: [],
    assets: [],
    sequences: [
      {
        id: sequenceId,
        name: "Main",
        format: {
          width: 1920,
          height: 1080,
          frameRate: { num: 30, den: 1 },
          sampleRate: DEFAULT_SAMPLE_RATE,
          audioLayout: "stereo",
          colorSpace: "rec709-sdr",
          dropFrameTimecode: false,
          autoMatchPending: true,
        },
        tracks: [],
        captionTracks: [],
        markers: [],
        inPoint: 0,
        outPoint: 0,
        masterAudio: { volumeDb: 0, effects: [] },
      },
    ],
    ui: {
      activeSequenceId: sequenceId,
      playheads: {},
      timelineZoom: {},
      timelineScroll: {},
      sourceViewer: { mediaId: null, in: 0, out: 0, playhead: 0 },
      selectedTrackTargets: {},
    },
  };
}

/** The recorded calls of a mock transport. */
export interface MockCalls {
  app_info: number;
  app_ready: number;
  app_quit: Array<{ force: boolean }>;
  project_new: Array<NewProjectRequest>;
  project_open: string[];
  project_save: number;
  project_close: Array<{ discardChanges: boolean }>;
}

/** The mock's open project state (one at a time, FR-001). */
interface MockProjectState {
  snapshot: ProjectSnapshot | null;
  recents: RecentProject[];
}

/**
 * A mock transport over the implemented commands. `paths` seeds the
 * recents; entries under `missingPaths` answer E1001 on open (the
 * Welcome's missing indicator).
 */
export function ipcMockTransport(options?: {
  recents?: RecentProject[];
  missingPaths?: string[];
}): IpcTransport & { calls: MockCalls; state: MockProjectState } {
  const calls: MockCalls = {
    app_info: 0,
    app_ready: 0,
    app_quit: [],
    project_new: [],
    project_open: [],
    project_save: 0,
    project_close: [],
  };
  const state: MockProjectState = {
    snapshot: null,
    recents: options?.recents ?? [],
  };
  const missing = new Set(options?.missingPaths ?? []);
  const record = (path: string, name: string): void => {
    state.recents = [
      {
        path,
        name,
        thumbnail: null,
        openedUtc: "2026-01-01T00:00:00Z",
        pinned: false,
      },
      ...state.recents.filter((entry) => entry.path !== path),
    ];
  };
  const snapshotOf = (project: Project, path: string): ProjectSnapshot => ({
    project,
    path,
    readOnly: null,
  });
  return {
    calls,
    state,
    // The real backend rejects with raw envelope objects (Tauri
    // serializes Result errors as plain JSON) - the mock mirrors
    // that wire shape exactly, which is the point of a mock backend.
    invoke: (() => {
      const target: IpcTransport["invoke"] = <T>(
        command: string,
        payload?: unknown,
      ): Promise<T> => {
        /* eslint-disable @typescript-eslint/prefer-promise-reject-errors -- envelope rejections are the documented wire shape (04 s4.9) */
        const reject = (value: unknown): Promise<T> => Promise.reject(value);
        switch (command) {
          case "app_info":
            calls.app_info += 1;
            return Promise.resolve(appInfoFixture() as T);
          case "app_ready":
            calls.app_ready += 1;
            return Promise.resolve(undefined as T);
          case "app_quit": {
            const request = (payload ?? {}) as { force?: boolean };
            calls.app_quit.push({ force: request.force === true });
            return Promise.resolve(undefined as T);
          }
          case "project_recent":
            return Promise.resolve(state.recents as T);
          case "project_new": {
            const request = (payload ?? {}) as NewProjectRequest;
            calls.project_new.push(request);
            if (
              request.name.length === 0 ||
              request.name.length > 200 ||
              /[<>:"/\\|?*\u0000-\u001f]/.test(request.name) ||
              presetById(request.sequencePreset) === undefined ||
              frameRateById(request.frameRate) === undefined ||
              (request.sampleRate !== 44_100 && request.sampleRate !== 48_000)
            ) {
              return reject(
                envelope("E1003", "error", "mock: invalid new-project request"),
              );
            }
            const project = projectFixture(request.name);
            const path = `${request.folder}\\${request.name}\\${request.name}.alvs`;
            state.snapshot = snapshotOf(project, path);
            record(path, request.name);
            return Promise.resolve(state.snapshot as T);
          }
          case "project_open":
          case "project_open_readonly":
          case "project_take_over_lock": {
            const { path } = (payload ?? {}) as { path?: string };
            if (typeof path !== "string") {
              return reject(envelope("E1003", "error", "mock: path required"));
            }
            calls.project_open.push(path);
            if (missing.has(path)) {
              return reject(
                envelope("E1001", "error", `mock: ${path} not found`),
              );
            }
            const name =
              path
                .split("\\")
                .pop()
                ?.replace(/\.alvs$/iu, "") ?? "Project";
            state.snapshot = snapshotOf(projectFixture(name), path);
            record(path, name);
            return Promise.resolve(state.snapshot as T);
          }
          case "project_save":
            calls.project_save += 1;
            return Promise.resolve({
              path: state.snapshot?.path ?? "",
              backupPath: null,
            } as T);
          case "project_save_as": {
            const { path } = (payload ?? {}) as { path?: string };
            if (state.snapshot !== null && typeof path === "string") {
              state.snapshot = snapshotOf(state.snapshot.project, path);
              record(path, state.snapshot.project.name);
            }
            return Promise.resolve({ path: path ?? "", backupPath: null } as T);
          }
          case "project_close": {
            const request = (payload ?? {}) as { discardChanges?: boolean };
            calls.project_close.push({
              discardChanges: request.discardChanges === true,
            });
            state.snapshot = null;
            return Promise.resolve(undefined as T);
          }
          default:
            return Promise.reject(
              new Error(
                `mock backend: command "${command}" is not implemented`,
              ),
            );
          /* eslint-enable @typescript-eslint/prefer-promise-reject-errors */
        }
      };
      return target;
    })(),
  };
}

/** A fixture envelope (04 s4.9 shape). */
function envelope(
  code: string,
  severity: "info" | "warning" | "error",
  detail: string,
): {
  code: string;
  severity: string;
  messageKey: string;
  params: Record<string, string>;
  detail: string;
  retryable: boolean;
  recovery: Array<{ commandId: string; labelKey: string }>;
} {
  return {
    code,
    severity,
    messageKey: `error.${code}`,
    params: {},
    detail,
    retryable: false,
    recovery: [],
  };
}
