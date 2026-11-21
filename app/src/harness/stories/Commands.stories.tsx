// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * Command stories: the palette body rendered inline (the modal
 * variant is fixed-position and cannot sit in the story grid; the
 * body is the same component the modal hosts).
 */
import { CommandPaletteBody } from "../../commands/palette/CommandPalette";
import styles from "../../commands/palette/CommandPalette.module.css";
import type { StoryFile } from "./types";

function PaletteDemo(): React.JSX.Element {
  return (
    <div className={styles.dialog}>
      <CommandPaletteBody
        flags={{ focusedContext: "global", projectOpen: true }}
      />
    </div>
  );
}

export const commandsStories: StoryFile = {
  id: "commands",
  stories: [{ id: "commands.palette", render: () => <PaletteDemo /> }],
};
