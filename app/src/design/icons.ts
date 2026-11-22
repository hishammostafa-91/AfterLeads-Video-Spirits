// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * Icon conventions (07 §7.5): Lucide (ISC) is the base set — 16 px in
 * dense UI, 20 px in toolbars, stroke 1.75. Components import icons
 * from this barrel so the base set has one source; custom feature
 * icons (assets/icons/, original works, same grid and stroke) ship
 * with their features per 07 §7.5 and are NOT added here. Icon-only
 * buttons must carry a tooltip (label + shortcut) and an accessible
 * name (REQ-HLP-003); directional icons mirror in RTL per 07 §7.11
 * (media-directional glyphs only — see the RTL rules there).
 */

/** Icon size in dense UI surfaces (07 §7.5). */
export const ICON_SIZE_DENSE = 16;

/** Icon size in toolbars (07 §7.5). */
export const ICON_SIZE_TOOLBAR = 20;

/** Lucide stroke width for every icon in the UI (07 §7.5). */
export const ICON_STROKE_WIDTH = 1.75;

// The base set the shipped components use (07 §7.5; keep sorted).
export {
  Binary,
  Check,
  ChevronDown,
  ChevronFirst,
  ChevronLast,
  ChevronRight,
  CloudOff,
  Diamond,
  EllipsisVertical,
  Film,
  Info,
  Minus,
  MonitorPlay,
  PackageOpen,
  Pencil,
  Plus,
  Shuffle,
  Timer,
  TriangleAlert,
  Waves,
  // Toolbar additions (WP-05 shell, 07 s7.2; keep sorted).
  ArrowLeftRight,
  Bell,
  Hand,
  Link2,
  ListTodo,
  Magnet,
  MousePointer2,
  MoveDiagonal,
  MoveHorizontal,
  Redo2,
  Scissors,
  Settings2,
  Undo2,
  ZoomIn,
  // Panel registry icons (WP-05 dock, 07 s7.7; keep sorted).
  Captions,
  History,
  SlidersHorizontal,
  Sparkles,
} from "lucide-react";
