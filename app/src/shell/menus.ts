// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * The menu bar structure (12 §12.11): seven menus in order, each an
 * ordered list of groups — the groups are the §12.11 separator sets,
 * and every menu's items follow KB order from 12 §12.10 (the catalog
 * carries the submenu placement, e.g. "View › Panels"). The bar
 * renders from this data plus the command registry (labels,
 * shortcuts, enablement); commands unimplemented in any shipped work
 * package stay disabled with the later-version tooltip.
 *
 * Dynamic entries §12.11 defines outside the command catalog: File's
 * "Open Recent ▸" submenu (recent projects, REQ-APP-005/REQ-PRJ
 * wiring) renders right after `file.openProject` in the first group
 * — see OPEN_RECENT_AFTER_ID.
 */
import { COMMANDS } from "../commands/catalog";

export type MenuId =
  "File" | "Edit" | "Clip" | "Timeline" | "Playback" | "View" | "Help";

/** One separator-delimited group of command ids (KB order). */
export interface MenuGroup {
  items: readonly string[];
}

export interface MenuSpec {
  id: MenuId;
  groups: readonly MenuGroup[];
}

/** Where File's dynamic "Open Recent ▸" submenu sits (12 §12.11). */
export const OPEN_RECENT_AFTER_ID = "file.openProject";

export const MENUS: readonly MenuSpec[] = [
  {
    id: "File",
    groups: [
      { items: ["file.newProject", "file.openProject"] },
      { items: ["file.save", "file.saveAs", "file.closeProject"] },
      { items: ["file.importMedia", "file.importCaptions"] },
      {
        items: [
          "file.export",
          "file.exportFrame",
          "file.exportMarkers",
          "file.exportEdl",
        ],
      },
      { items: ["file.collectFiles", "file.restoreBackup"] },
      { items: ["file.projectSettings", "file.settings"] },
      { items: ["file.exit"] },
    ],
  },
  {
    id: "Edit",
    groups: [
      { items: ["edit.undo", "edit.redo"] },
      {
        items: [
          "edit.cut",
          "edit.copy",
          "edit.paste",
          "edit.pasteInsert",
          "edit.pasteAttributes",
          "edit.duplicate",
        ],
      },
      { items: ["edit.delete", "edit.rippleDelete"] },
      { items: ["edit.selectAll", "edit.deselectAll"] },
      { items: ["edit.rename", "edit.find"] },
      { items: ["edit.keyboardShortcuts"] },
    ],
  },
  {
    id: "Clip",
    groups: [
      { items: ["clip.split", "clip.splitAll"] },
      {
        items: [
          "clip.toggleEnabled",
          "clip.toggleLink",
          "clip.group",
          "clip.ungroup",
        ],
      },
      {
        items: [
          "clip.speedDuration",
          "clip.reverse",
          "clip.freezeFrame",
          "clip.nest",
        ],
      },
      { items: ["clip.matchFrame"] },
      {
        items: ["clip.addMaskRect", "clip.addMaskEllipse", "clip.addMaskPath"],
      },
      { items: ["clip.labelColor"] },
      {
        items: ["clip.normalizeLoudness", "clip.stabilize", "clip.transcribe"],
      },
      { items: ["clip.copyGrade", "clip.pasteGrade"] },
      {
        items: [
          "media.createProxy",
          "media.optimize",
          "media.relink",
          "media.revealInExplorer",
        ],
      },
    ],
  },
  {
    id: "Timeline",
    groups: [
      { items: ["timeline.newSequence", "timeline.sequenceSettings"] },
      {
        items: [
          "timeline.addVideoTrack",
          "timeline.addAudioTrack",
          "timeline.addCaptionTrack",
        ],
      },
      {
        items: [
          "timeline.insert",
          "timeline.overwrite",
          "timeline.lift",
          "timeline.extract",
        ],
      },
      {
        items: [
          "timeline.trimPrevToPlayhead",
          "timeline.trimNextToPlayhead",
          "timeline.extendEdit",
        ],
      },
      {
        items: ["timeline.addVideoTransition", "timeline.addAudioTransition"],
      },
      {
        items: ["timeline.toggleSnapping", "timeline.toggleLinkedSelection"],
      },
      { items: ["timeline.closeGap", "timeline.deleteAllGaps"] },
      {
        items: [
          "timeline.nudgeLeft",
          "timeline.nudgeRight",
          "timeline.nudgeLeft10",
          "timeline.nudgeRight10",
          "timeline.audioSubframeNudge",
        ],
      },
      { items: ["timeline.zoomIn", "timeline.zoomOut", "timeline.zoomFit"] },
      {
        items: [
          "timeline.addMarker",
          "timeline.editMarker",
          "timeline.nextMarker",
          "timeline.prevMarker",
        ],
      },
      { items: ["timeline.addCaption"] },
      { items: ["timeline.listView"] },
      {
        items: [
          "timeline.recordVoiceOver",
          "timeline.autoDuck",
          "timeline.transcribeSequence",
        ],
      },
      {
        items: [
          "tool.select",
          "tool.blade",
          "tool.trim",
          "tool.slip",
          "tool.slide",
          "tool.hand",
          "tool.zoom",
        ],
      },
      {
        items: [
          "timeline.selectPrevClip",
          "timeline.selectNextClip",
          "timeline.focusTrackUp",
          "timeline.focusTrackDown",
        ],
      },
      {
        items: [
          "timeline.trimEdgeLeft",
          "timeline.trimEdgeRight",
          "timeline.trimEdgeLeft10",
          "timeline.trimEdgeRight10",
        ],
      },
    ],
  },
  {
    id: "Playback",
    groups: [
      {
        items: [
          "playback.playPause",
          "playback.shuttleReverse",
          "playback.pause",
          "playback.shuttleForward",
        ],
      },
      {
        items: [
          "playback.stepBack",
          "playback.stepForward",
          "playback.stepBack10",
          "playback.stepForward10",
          "playback.prevEdit",
          "playback.nextEdit",
          "playback.goStart",
          "playback.goEnd",
        ],
      },
      {
        items: [
          "playback.markIn",
          "playback.markOut",
          "playback.clearIn",
          "playback.clearOut",
          "playback.clearInOut",
          "playback.goIn",
          "playback.goOut",
        ],
      },
      { items: ["playback.playInToOut", "playback.toggleLoop"] },
      { items: ["playback.goToTimecode"] },
      {
        items: [
          "playback.toggleAudioScrub",
          "playback.previewQuality",
          "playback.playEveryFrame",
        ],
      },
      { items: ["playback.fullscreen", "playback.externalPreview"] },
      // KB-159 (Announce Timecode) is a §12.10 Playback command the
      // §12.11 group table does not name; it closes the menu in KB
      // order.
      { items: ["playback.announceTimecode"] },
    ],
  },
  {
    id: "View",
    groups: [
      {
        items: [
          "view.workspaceEdit",
          "view.workspaceColor",
          "view.workspaceAudio",
          "view.workspaceCaptions",
          "view.saveWorkspace",
          "view.resetWorkspace",
        ],
      },
      { items: ["view.commandPalette"] },
      {
        items: ["view.maximizePanel", "view.nextPanel", "view.prevPanel"],
      },
      {
        items: [
          "view.focusMedia",
          "view.focusSource",
          "view.focusTimeline",
          "view.focusProgram",
          "view.focusInspector",
          "view.focusEffects",
          "view.focusCaptions",
          "view.focusMixer",
          "view.focusScopes",
        ],
      },
      {
        items: ["view.uiZoomIn", "view.uiZoomOut", "view.uiZoomReset"],
      },
      {
        items: ["view.toggleSafeAreas", "view.toggleGrid", "view.compareSplit"],
      },
      { items: ["view.theme"] },
    ],
  },
  {
    id: "Help",
    groups: [
      {
        items: [
          "help.gettingStarted",
          "help.userGuide",
          "help.shortcutSheet",
          "help.replayTour",
        ],
      },
      { items: ["help.reportProblem", "help.diagnostics"] },
      { items: ["help.checkUpdates"] },
      { items: ["help.about"] },
    ],
  },
];

/** The flattened, KB-ordered items of one menu (rendering helper). */
export function menuItems(menu: MenuSpec): readonly string[] {
  return menu.groups.flatMap((group) => group.items);
}

/** The catalog kb number of a command id ("KB-079" → 79). */
function kbOf(id: string): number {
  const spec = COMMANDS.find((command) => command.id === id);
  if (!spec) throw new Error(`menus: unknown command "${id}"`);
  return Number(spec.kb.slice(3));
}

/** Menu ids in bar order (07 §7.2 title bar; 12 §12.11). */
export const MENU_ORDER: readonly MenuId[] = MENUS.map((menu) => menu.id);

export { kbOf };
