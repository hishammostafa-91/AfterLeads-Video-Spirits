// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * The Welcome screen (REQ-APP-005; FR-011): New Project, Open
 * Project, recents (with the documented thumbnail placeholder until
 * WP-08 writes real thumbs, and a missing indicator for stale
 * paths - never an error toast), and the Getting Started link (HLP
 * territory: the Help menu's id carries the target until WP-26).
 */
import { useIntl } from "react-intl";
import type { JSX } from "react";
import { useEffect } from "react";
import { Button } from "../components/Button";
import type { RecentProject } from "../ipc/bindings/RecentProject";
import { useProjectStore } from "../projects/store";
import styles from "./Welcome.module.css";

export interface WelcomeProps {
  onNewProject: () => void;
  onOpenProject: () => void;
  /** Open a recent (a stale path surfaces the missing indicator). */
  onOpenRecent: (path: string) => void;
}

export function Welcome({
  onNewProject,
  onOpenProject,
  onOpenRecent,
}: WelcomeProps): JSX.Element {
  const { formatMessage } = useIntl();
  const recents = useProjectStore((state) => state.recents);
  const missingPaths = useProjectStore((state) => state.missingPaths);
  const refreshRecents = useProjectStore((state) => state.refreshRecents);
  useEffect(() => {
    void refreshRecents();
  }, [refreshRecents]);

  return (
    <section
      className={styles.welcome}
      aria-label={formatMessage({ id: "welcome.title" })}
    >
      <h1 className={styles.title}>{formatMessage({ id: "welcome.title" })}</h1>
      <p className={styles.subtitle}>
        {formatMessage({ id: "welcome.subtitle" })}
      </p>
      <div className={styles.actions}>
        <Button variant="primary" onPress={onNewProject}>
          {formatMessage({ id: "welcome.newProject" })}
        </Button>
        <Button variant="secondary" onPress={onOpenProject}>
          {formatMessage({ id: "welcome.openProject" })}
        </Button>
      </div>
      <div className={styles.recents}>
        <h2 className={styles.recentsTitle}>
          {formatMessage({ id: "welcome.recents" })}
        </h2>
        {recents.length === 0 ? (
          <p className={styles.recentsEmpty}>
            {formatMessage({ id: "welcome.recentsEmpty" })}
          </p>
        ) : (
          <ul className={styles.recentsList}>
            {recents.map((recent) => (
              <RecentRow
                key={recent.path}
                recent={recent}
                missing={missingPaths.has(recent.path)}
                onOpen={onOpenRecent}
              />
            ))}
          </ul>
        )}
      </div>
      <a
        className={styles.gettingStarted}
        href="#getting-started"
        onClick={(event) => {
          // The guide viewer ships with WP-26 (HLP territory); the
          // link records the target and stays inert until then.
          event.preventDefault();
        }}
      >
        {formatMessage({ id: "welcome.gettingStarted" })}
      </a>
    </section>
  );
}

function RecentRow({
  recent,
  missing,
  onOpen,
}: {
  recent: RecentProject;
  missing: boolean;
  onOpen: (path: string) => void;
}): JSX.Element {
  const { formatMessage } = useIntl();
  return (
    <li>
      <button
        type="button"
        className={styles.recent}
        onClick={() => {
          onOpen(recent.path);
        }}
      >
        {/* Thumbnail placeholder: WP-08 writes the real thumbs
         * (assets/help scope per the card's risk list). */}
        <span className={styles.thumb} aria-hidden="true" />
        <span className={styles.recentName}>
          {recent.name}
          {missing ? (
            <span className={styles.missing}>
              {formatMessage({ id: "welcome.missing" })}
            </span>
          ) : null}
        </span>
        <span className={styles.recentPath} dir="ltr">
          {recent.path}
        </span>
      </button>
    </li>
  );
}
