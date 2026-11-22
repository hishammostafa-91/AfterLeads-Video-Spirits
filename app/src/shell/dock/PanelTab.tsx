// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * The panel tab (07 s7.9.1): the focused panel - the active panel of
 * the active group - carries a 1 px accent line on its tab.
 *
 * Accessibility follows the APG deletable-tabs pattern: the ARIA `tab`
 * role's children are presentational, so a nested close *control*
 * would be an axe nested-interactive violation and lose its role for
 * AT. Closing is the x glyph (pointer, CSS-drawn - no DOM text that
 * could mismatch the tab's accessible name) plus Delete on the
 * focused tab (keyboard). Middle-click closes (07 s7.8). Titles
 * render from the panel registry, and the host's aria-label is kept
 * in step through `api.setTitle` (dockview derives it from the panel
 * title; visible text must be part of the accessible name).
 */
import { useIntl } from "react-intl";
import { useEffect, useRef } from "react";
import type { JSX } from "react";
import type { IDockviewPanelHeaderProps } from "dockview-react";
import { PANEL_BY_ID } from "../../panels/registry";
import styles from "./PanelTab.module.css";

export function PanelTab({ api }: IDockviewPanelHeaderProps): JSX.Element {
  const { formatMessage } = useIntl();
  const rootRef = useRef<HTMLDivElement | null>(null);
  const middleButton = useRef(false);
  // The stored title is the panel slug; render the localized name.
  const panel = PANEL_BY_ID.get(api.title ?? "");
  const title = panel
    ? formatMessage({ id: panel.titleKey })
    : (api.title ?? "");
  // Keep the dockview host's aria-label equal to the visible text.
  useEffect(() => {
    api.setTitle(title);
  }, [api, title]);

  // Keyboard closing (APG deletable tabs): Delete on the focused tab.
  // Focus lands on the dockview host element, so the listener attaches
  // there; the host lookup goes through the semantic role selector.
  useEffect(() => {
    const host = rootRef.current?.closest('[role="tab"]');
    if (!(host instanceof HTMLElement)) return;
    const onKeyDown = (event: KeyboardEvent): void => {
      if (event.key === "Delete") {
        event.preventDefault();
        api.close();
      }
    };
    host.addEventListener("keydown", onKeyDown);
    return () => {
      host.removeEventListener("keydown", onKeyDown);
    };
  }, [api]);

  return (
    <div
      ref={rootRef}
      className={styles.tab}
      data-panel={panel?.id}
      onPointerDown={(event) => {
        middleButton.current = event.button === 1;
      }}
      onPointerUp={(event) => {
        if (middleButton.current && event.button === 1) {
          middleButton.current = false;
          api.close();
        }
      }}
      onPointerLeave={() => {
        middleButton.current = false;
      }}
    >
      <span className={styles.title}>{title}</span>
      {/* Presentational glyph (CSS-drawn); pointer close. The keyboard
       * path is Delete on the focused tab (see above). */}
      <span
        className={styles.close}
        data-close="true"
        aria-hidden="true"
        onClick={(event) => {
          event.stopPropagation();
          api.close();
        }}
      />
    </div>
  );
}
