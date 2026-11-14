// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * Button family stories (07 §7.6; T017 backfill).
 */
import { FormattedMessage } from "react-intl";
import type { JSX } from "react";
import { Pencil } from "lucide-react";
import { Button, IconButton, ToggleButton } from "../../components/Button";
import styles from "./stories.module.css";
import type { StoryFile } from "./types";

function Variants(): JSX.Element {
  return (
    <div className={styles.row}>
      <Button variant="primary">
        <FormattedMessage id="components.button.ok" />
      </Button>
      <Button variant="secondary">
        <FormattedMessage id="components.dialog.cancel" />
      </Button>
      <Button variant="ghost">
        <FormattedMessage id="components.button.snap" />
      </Button>
      <Button variant="danger">
        <FormattedMessage id="components.dialog.discard" />
      </Button>
    </div>
  );
}

function Icon(): JSX.Element {
  return (
    <IconButton
      icon={<Pencil size={16} />}
      labelMessageId="components.button.edit"
      shortcut="F2"
    />
  );
}

function Toggle(): JSX.Element {
  return (
    <ToggleButton defaultSelected>
      <FormattedMessage id="components.button.snap" />
    </ToggleButton>
  );
}

function Disabled(): JSX.Element {
  return (
    <Button isDisabled>
      <FormattedMessage id="components.button.ok" />
    </Button>
  );
}

export const buttonStories: StoryFile = {
  id: "button",
  stories: [
    { id: "button.variants", render: () => <Variants /> },
    { id: "button.icon", render: () => <Icon /> },
    { id: "button.toggle", render: () => <Toggle /> },
    { id: "button.disabled", render: () => <Disabled /> },
  ],
};
