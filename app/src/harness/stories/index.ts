// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * The story registry (research D7): the union of all component story
 * files rendered by the harness grid and captured by the Playwright
 * matrix (T033).
 */
import type { StoryFile } from "./types";
import { buttonStories } from "./Button.stories";
import { collectionStories } from "./Collections.stories";
import { colorStories } from "./Color.stories";
import { fieldsStories } from "./Fields.stories";
import { menuStories } from "./Menu.stories";
import { numericStories } from "./Numeric.stories";
import { overlayStories } from "./Overlays.stories";
import { selectionStories } from "./Selection.stories";
import { tabsStories } from "./Tabs.stories";

const allStoryFiles: StoryFile[] = [
  buttonStories,
  fieldsStories,
  selectionStories,
  menuStories,
  overlayStories,
  tabsStories,
  numericStories,
  colorStories,
  collectionStories,
];

// RAC 1.22's virtualized collections (Tree/GridList/Table) render
// hidden content through a <template> that conflicts with React 19.3
// under jsdom (createTextNode during commit) once other overlays have
// mounted in the same document. Their jsdom component tests run in
// isolated files (Collections.test.tsx); the full story matrix —
// including these — runs in the real-browser Playwright harness
// (research D7), so the vitest harness grid skips just this file.
const JSDOM_UNSTABLE = new Set(["collections"]);

export const storyFiles: StoryFile[] = import.meta.env.VITEST
  ? allStoryFiles.filter((file) => !JSDOM_UNSTABLE.has(file.id))
  : allStoryFiles;
