// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * The status bar (REQ-APP-007; 07 s7.2/s7.10): background job
 * summary, autosave state with the last save time, playback
 * statistics (fps + dropped frames), proxy mode, and the renderer
 * (GPU name or "WARP"). Live values come from the status store
 * (`app_info` + `autosave:status` + `jobs:update` summaries);
 * sources that ship later (WP-07/WP-11) render their defined place
 * with the neutral dash - never a fake value.
 */
import type { JSX } from "react";
import { useIntl } from "react-intl";
import { useStatusStore } from "./status";
import styles from "./StatusBar.module.css";

const DASH = "—";

export function StatusBar(): JSX.Element {
  const { formatMessage } = useIntl();
  const rendererName = useStatusStore((state) => state.renderer);
  const autosave = useStatusStore((state) => state.autosave);
  const jobs = useStatusStore((state) => state.jobs);

  const jobsSummary = jobs
    ? formatMessage({ id: "shell.status.jobsRunning" }, { count: jobs.running })
    : DASH;
  const lastSavedAt = autosave?.timeUtc
    ? new Date(autosave.timeUtc).toLocaleTimeString(undefined, {
        hour: "2-digit",
        minute: "2-digit",
      })
    : undefined;
  const autosaveText =
    autosave === null
      ? DASH
      : autosave.state === "saving"
        ? formatMessage({ id: "shell.status.autosaving" })
        : autosave.state === "failed"
          ? formatMessage({ id: "shell.status.autosaveFailed" })
          : lastSavedAt
            ? `${formatMessage({ id: "shell.status.savedAt" })} ${lastSavedAt}`
            : DASH;

  return (
    <footer
      className={styles.statusBar}
      aria-label={formatMessage({ id: "shell.status.label" })}
    >
      <span
        className={styles.field}
        title={formatMessage({ id: "shell.status.jobs" })}
      >
        {jobsSummary}
      </span>
      <span
        className={styles.field}
        title={formatMessage({ id: "shell.status.autosave" })}
      >
        {autosaveText}
      </span>
      <span
        className={styles.field}
        title={formatMessage({ id: "shell.status.playback" })}
      >
        {DASH}
      </span>
      <span
        className={styles.field}
        title={formatMessage({ id: "shell.status.proxy" })}
      >
        {DASH}
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
