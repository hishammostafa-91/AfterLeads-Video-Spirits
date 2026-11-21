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
import { commandsStories } from "./Commands.stories";
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
  commandsStories,
];

export const storyFiles: StoryFile[] = allStoryFiles;
