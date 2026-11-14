// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * Selection-control stories (07 §7.6; T028 backfill).
 */
import { FormattedMessage, useIntl } from "react-intl";
import type { JSX } from "react";
import { useState } from "react";
import {
  Checkbox,
  ComboBox,
  ListBoxItem,
  Radio,
  RadioGroup,
  Select,
  Switch,
} from "../../components/Selection";
import styles from "./stories.module.css";
import type { StoryFile } from "./types";

function Checks(): JSX.Element {
  return (
    <div className={styles.column}>
      <Checkbox defaultSelected>
        <FormattedMessage id="components.selection.checkboxLabel" />
      </Checkbox>
      <Switch defaultSelected>
        <FormattedMessage id="components.selection.switchLabel" />
      </Switch>
    </div>
  );
}

function Radios(): JSX.Element {
  const { formatMessage } = useIntl();
  return (
    <RadioGroup
      aria-label={formatMessage({ id: "components.selection.groupLabel" })}
    >
      <Radio value="fit">
        <FormattedMessage id="components.selection.fitLabel" />
      </Radio>
      <Radio value="fill">
        <FormattedMessage id="components.selection.fillLabel" />
      </Radio>
    </RadioGroup>
  );
}

function Picks(): JSX.Element {
  const { formatMessage } = useIntl();
  const [selected, setSelected] = useState<string | null>("first");
  return (
    <div className={styles.column}>
      <Select
        label={formatMessage({ id: "components.selection.selectLabel" })}
        value={selected}
        onChange={(value) => {
          setSelected(value === null ? null : String(value));
        }}
      >
        <ListBoxItem id="first">
          <FormattedMessage id="components.selection.optionFirst" />
        </ListBoxItem>
        <ListBoxItem id="second">
          <FormattedMessage id="components.selection.optionSecond" />
        </ListBoxItem>
      </Select>
      <ComboBox
        label={formatMessage({ id: "components.selection.comboLabel" })}
      >
        <ListBoxItem id="first">
          <FormattedMessage id="components.selection.optionFirst" />
        </ListBoxItem>
        <ListBoxItem id="second">
          <FormattedMessage id="components.selection.optionSecond" />
        </ListBoxItem>
      </ComboBox>
    </div>
  );
}

/** FR-007 disabled + error states for the selection family. */
function SelectionStates(): JSX.Element {
  const { formatMessage } = useIntl();
  const [selected, setSelected] = useState<string | null>("first");
  return (
    <div className={styles.column}>
      <Checkbox isDisabled>
        <FormattedMessage id="components.selection.checkboxLabel" />
      </Checkbox>
      <Switch isDisabled defaultSelected>
        <FormattedMessage id="components.selection.switchLabel" />
      </Switch>
      <RadioGroup
        aria-label={formatMessage({ id: "components.selection.groupLabel" })}
      >
        <Radio value="fit" isDisabled>
          <FormattedMessage id="components.selection.fitLabel" />
        </Radio>
      </RadioGroup>
      <Select
        label={formatMessage({ id: "components.selection.selectLabel" })}
        isInvalid
        value={selected}
        onChange={(value) => {
          setSelected(value === null ? null : String(value));
        }}
      >
        <ListBoxItem id="first">
          <FormattedMessage id="components.selection.optionFirst" />
        </ListBoxItem>
        <ListBoxItem id="second">
          <FormattedMessage id="components.selection.optionSecond" />
        </ListBoxItem>
      </Select>
    </div>
  );
}

export const selectionStories: StoryFile = {
  id: "selection",
  stories: [
    { id: "selection.checkbox-switch", render: () => <Checks /> },
    { id: "selection.radio", render: () => <Radios /> },
    { id: "selection.select-combo", render: () => <Picks /> },
    { id: "selection.states", render: () => <SelectionStates /> },
  ],
};
