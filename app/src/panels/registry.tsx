// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * The panel registry (07 s7.7 PNL-01..15): the single typed table of
 * every dockable panel - its stable slug (the dockview panel id), the
 * PNL id, title key, icon, and the work package that ships its real
 * content. WP-05 docks labeled, accessible placeholders (spec D3: no
 * fake data; each names its owning WP, mirroring the disabled-command
 * rule). The owning WPs come from the canonical REQ table in 01
 * s3 (e.g. REQ-JOB-002 -> WP-07, REQ-FX-002 -> WP-14), not guesses.
 */
import {
  Captions,
  Film,
  History,
  ListTodo,
  MonitorPlay,
  PackageOpen,
  Settings2,
  SlidersHorizontal,
  Sparkles,
  Timer,
  Waves,
} from "../design/icons";
import type { ReactNode } from "react";

/** A dockable panel (07 s7.7). */
export interface PanelSpec {
  /** Stable slug used as the dockview panel id. */
  id: string;
  /** The PNL-xx id from 07 s7.7. */
  pnlId: string;
  /** Locale key of the panel title (tab label). */
  titleKey: string;
  /** Panel icon (tabs / placeholders). */
  icon: ReactNode;
  /** The WP that ships this panel's real content (label on the placeholder). */
  owningWp: string;
  /** The `view.focus*` command that focuses it (12 s12.10 KB-135..143). */
  focusCommandId: string | null;
}

export const PANELS: readonly PanelSpec[] = [
  {
    id: "media",
    pnlId: "PNL-01",
    titleKey: "panels.media.title",
    icon: <Film aria-hidden="true" />,
    owningWp: "WP-08",
    focusCommandId: "view.focusMedia",
  },
  {
    id: "effects",
    pnlId: "PNL-02",
    titleKey: "panels.effects.title",
    icon: <Sparkles aria-hidden="true" />,
    owningWp: "WP-14",
    focusCommandId: "view.focusEffects",
  },
  {
    id: "transitions",
    pnlId: "PNL-03",
    titleKey: "panels.transitions.title",
    icon: <Timer aria-hidden="true" />,
    owningWp: "WP-16",
    focusCommandId: null,
  },
  {
    id: "titles",
    pnlId: "PNL-04",
    titleKey: "panels.titles.title",
    icon: <PackageOpen aria-hidden="true" />,
    owningWp: "WP-17",
    focusCommandId: null,
  },
  {
    id: "inspector",
    pnlId: "PNL-05",
    titleKey: "panels.inspector.title",
    icon: <SlidersHorizontal aria-hidden="true" />,
    owningWp: "WP-13",
    focusCommandId: "view.focusInspector",
  },
  {
    id: "program",
    pnlId: "PNL-06",
    titleKey: "panels.program.title",
    icon: <MonitorPlay aria-hidden="true" />,
    owningWp: "WP-11",
    focusCommandId: "view.focusProgram",
  },
  {
    id: "source",
    pnlId: "PNL-07",
    titleKey: "panels.source.title",
    icon: <MonitorPlay aria-hidden="true" />,
    owningWp: "WP-11",
    focusCommandId: "view.focusSource",
  },
  {
    id: "timeline",
    pnlId: "PNL-08",
    titleKey: "panels.timeline.title",
    icon: <Film aria-hidden="true" />,
    owningWp: "WP-13",
    focusCommandId: "view.focusTimeline",
  },
  {
    id: "mixer",
    pnlId: "PNL-09",
    titleKey: "panels.mixer.title",
    icon: <Waves aria-hidden="true" />,
    owningWp: "WP-18",
    focusCommandId: "view.focusMixer",
  },
  {
    id: "scopes",
    pnlId: "PNL-10",
    titleKey: "panels.scopes.title",
    icon: <Waves aria-hidden="true" />,
    owningWp: "WP-19",
    focusCommandId: "view.focusScopes",
  },
  {
    id: "captions",
    pnlId: "PNL-11",
    titleKey: "panels.captions.title",
    icon: <Captions aria-hidden="true" />,
    owningWp: "WP-20",
    focusCommandId: "view.focusCaptions",
  },
  {
    id: "markers",
    pnlId: "PNL-12",
    titleKey: "panels.markers.title",
    icon: <ListTodo aria-hidden="true" />,
    owningWp: "WP-13",
    focusCommandId: null,
  },
  {
    id: "history",
    pnlId: "PNL-13",
    titleKey: "panels.history.title",
    icon: <History aria-hidden="true" />,
    owningWp: "WP-13",
    focusCommandId: null,
  },
  {
    id: "jobs",
    pnlId: "PNL-14",
    titleKey: "panels.jobs.title",
    icon: <ListTodo aria-hidden="true" />,
    owningWp: "WP-07",
    focusCommandId: null,
  },
  {
    id: "transcript",
    pnlId: "PNL-15",
    titleKey: "panels.transcript.title",
    icon: <Settings2 aria-hidden="true" />,
    owningWp: "WP-21",
    focusCommandId: null,
  },
];

export const PANEL_BY_ID: ReadonlyMap<string, PanelSpec> = new Map(
  PANELS.map((panel) => [panel.id, panel]),
);

/** The command id -> panel slug map (KB-135..143). */
export const PANEL_BY_FOCUS_COMMAND: ReadonlyMap<string, PanelSpec> = new Map(
  PANELS.flatMap((panel) =>
    panel.focusCommandId ? [[panel.focusCommandId, panel] as const] : [],
  ),
);
