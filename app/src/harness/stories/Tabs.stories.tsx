// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * Tabs stories (07 §7.6; T020).
 */
import { FormattedMessage } from "react-intl";
import type { JSX } from "react";
import { Tab, TabList, TabPanel, Tabs } from "../../components/Tabs";
import type { StoryFile } from "./types";

function TabsDemo(): JSX.Element {
  return (
    <Tabs defaultSelectedKey="properties">
      <TabList aria-label="Inspector">
        <Tab id="properties">
          <FormattedMessage id="components.tabs.properties" />
        </Tab>
        <Tab id="effects">
          <FormattedMessage id="components.tabs.effects" />
        </Tab>
      </TabList>
      <TabPanel id="properties">
        <FormattedMessage id="components.tabs.properties" />
      </TabPanel>
      <TabPanel id="effects">
        <FormattedMessage id="components.tabs.effects" />
      </TabPanel>
    </Tabs>
  );
}

/** FR-007 disabled state: the middle tab is not selectable. */
function TabsStates(): JSX.Element {
  return (
    <Tabs defaultSelectedKey="properties">
      <TabList aria-label="Inspector">
        <Tab id="properties">
          <FormattedMessage id="components.tabs.properties" />
        </Tab>
        <Tab id="effects" isDisabled>
          <FormattedMessage id="components.tabs.effects" />
        </Tab>
      </TabList>
      <TabPanel id="properties">
        <FormattedMessage id="components.tabs.properties" />
      </TabPanel>
    </Tabs>
  );
}

export const tabsStories: StoryFile = {
  id: "tabs",
  stories: [
    { id: "tabs.inspector", render: () => <TabsDemo /> },
    { id: "tabs.states", render: () => <TabsStates /> },
  ],
};
