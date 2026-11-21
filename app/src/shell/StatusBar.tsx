// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * The status bar (REQ-APP-007; 07 §7.2/§7.10): background job
 * summary, autosave state with the last save time, playback
 * statistics (fps + dropped frames), proxy mode, and the renderer
 * (GPU name or "WARP"). Fields whose data sources ship with later
 * work packages (WP-07/WP-11) render their defined place with the
 * neutral dash — never a fake value.
 */
import type { JSX } from "react";
import { useIntl } from "react-intl";
import styles from "./StatusBar.module.css";

export type AutosaveState = "idle" | "saving" | "saved" | "failed";

export interface StatusBarProps {
  /** e.g. "2 jobs running"; undefined until WP-07. */
  jobsSummary?: string;
  autosaveState?: AutosaveState;
  /** Last save time (localized string); undefined when never saved. */
  lastSavedAt?: string;
  /** Frames per second during playback; undefined until WP-11. */
  fps?: number;
  droppedFrames?: number;
  proxyMode?: boolean;
  /** GPU name or "WARP" (from app_info.renderer); undefined pre-app. */
  rendererName?: string;
}

const DASH = "—";

export function StatusBar({
  jobsSummary,
  autosaveState,
  lastSavedAt,
  fps,
  droppedFrames,
  proxyMode = false,
  rendererName,
}: StatusBarProps): JSX.Element {
  const { formatMessage } = useIntl();
  return (
    <footer
      className={styles.statusBar}
      aria-label={formatMessage({ id: "shell.status.label" })}
    >
      <span
        className={styles.field}
        title={formatMessage({ id: "shell.status.jobs" })}
      >
        {jobsSummary ?? DASH}
      </span>
      <span
        className={styles.field}
        title={formatMessage({ id: "shell.status.autosave" })}
      >
        {autosaveState === "saving"
          ? formatMessage({ id: "shell.status.autosaving" })
          : autosaveState === "failed"
            ? formatMessage({ id: "shell.status.autosaveFailed" })
            : lastSavedAt
              ? `${formatMessage({ id: "shell.status.savedAt" })} ${lastSavedAt}`
              : DASH}
      </span>
      <span
        className={styles.field}
        title={formatMessage({ id: "shell.status.playback" })}
      >
        {fps !== undefined
          ? `${String(fps)} fps • ${String(droppedFrames ?? 0)}`
          : DASH}
      </span>
      <span
        className={styles.field}
        title={formatMessage({ id: "shell.status.proxy" })}
      >
        {proxyMode ? formatMessage({ id: "shell.status.proxiesOn" }) : DASH}
      </span>
      <span
        className={styles.fieldRenderer}
        title={formatMessage({ id: "shell.status.renderer" })}
      >
        {rendererName ?? DASH}
      </span>
    </footer>
  );
}
