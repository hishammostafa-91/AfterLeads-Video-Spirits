// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * Story types (07 §7.6; research D7): each component family exports
 * one story file; the harness renders the union as the story grid the
 * Playwright matrix captures (stories double as documentation).
 */
import type { JSX } from "react";

export interface StorySpec {
  /**
   * Story id - doubles as the documentation label shown above the
   * demo (e.g. "button.primary"). The nine snapshot states of the
   * matrix (default, hover, focus-visible, pressed, disabled, error,
   * RTL, compact, high-contrast) are enumerated by the spec files.
   */
  id: string;
  render: () => JSX.Element;
  /**
   * Span the full grid row (product-surface stories like the dock
   * need real estate; the fixed rem tracks stay intact).
   */
  wide?: boolean;
}

export interface StoryFile {
  /** Component-family id, e.g. "button". */
  id: string;
  stories: StorySpec[];
}
