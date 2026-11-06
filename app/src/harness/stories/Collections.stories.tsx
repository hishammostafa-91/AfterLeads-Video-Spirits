// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * Collections + feedback stories (07 §7.6; T029–T030).
 */
import { FormattedMessage, useIntl } from "react-intl";
import type { JSX } from "react";
import { Film } from "lucide-react";
import {
  GridList,
  GridListItem,
  ListBox,
  ListBoxOption,
  Table,
  TableBody,
  TableCell,
  TableColumn,
  TableHeader,
  TableRow,
  Tree,
  TreeItem,
  WindowedListBox,
} from "../../components/Collections";
import { Badge } from "../../components/Badge";
import { EmptyState } from "../../components/EmptyState";
import {
  Meter,
  ProgressBar,
  Skeleton,
  Spinner,
} from "../../components/Feedback";
import styles from "./stories.module.css";
import type { StoryFile } from "./types";

function Trees(): JSX.Element {
  return (
    <Tree aria-label="Media">
      <TreeItem textValue="binOne">
        <FormattedMessage id="components.collections.binOne" />
      </TreeItem>
      <TreeItem textValue="binTwo">
        <FormattedMessage id="components.collections.binTwo" />
      </TreeItem>
    </Tree>
  );
}

function Grids(): JSX.Element {
  return (
    <GridList aria-label="Bin">
      <GridListItem>
        <FormattedMessage id="components.collections.clipOne" />
      </GridListItem>
      <GridListItem>
        <FormattedMessage id="components.collections.clipTwo" />
      </GridListItem>
    </GridList>
  );
}

function Tables(): JSX.Element {
  const { formatMessage } = useIntl();
  return (
    <Table aria-label={formatMessage({ id: "components.collections.table" })}>
      <TableHeader>
        <TableColumn>
          {formatMessage({ id: "components.collections.name" })}
        </TableColumn>
        <TableColumn>
          {formatMessage({ id: "components.collections.duration" })}
        </TableColumn>
      </TableHeader>
      <TableBody>
        <TableRow>
          <TableCell>
            {formatMessage({ id: "components.collections.clipOne" })}
          </TableCell>
          <TableCell>00:10:00:00</TableCell>
        </TableRow>
        <TableRow>
          <TableCell>
            {formatMessage({ id: "components.collections.clipTwo" })}
          </TableCell>
          <TableCell>00:00:30:12</TableCell>
        </TableRow>
      </TableBody>
    </Table>
  );
}

function Lists(): JSX.Element {
  return (
    <ListBox aria-label="Tracks">
      <ListBoxOption id="v1">
        <FormattedMessage id="components.collections.trackVideo" />
      </ListBoxOption>
      <ListBoxOption id="a1">
        <FormattedMessage id="components.collections.trackAudio" />
      </ListBoxOption>
    </ListBox>
  );
}

function Windowed(): JSX.Element {
  const items = Array.from({ length: 250 }, (_, i) => `row-${String(i + 1)}`);
  return (
    <WindowedListBox
      items={items}
      keyOf={(item) => item}
      renderOption={(item) => item}
      aria-label="Windowed"
    />
  );
}

function Progress(): JSX.Element {
  const { formatMessage } = useIntl();
  return (
    <div className={styles.column}>
      <ProgressBar
        label={formatMessage({ id: "components.feedback.render" })}
        value={65}
      />
      <Meter
        label={formatMessage({ id: "components.feedback.cache" })}
        value={30}
      />
      <div className={styles.row}>
        <Spinner />
        <Spinner value={70} />
      </div>
      <div className={styles.column}>
        <Skeleton />
        <Skeleton shape="block" inlineSize={14} blockSize={3} />
      </div>
    </div>
  );
}

function Badges(): JSX.Element {
  return (
    <div className={styles.row}>
      <Badge kind="proxy" />
      <Badge kind="offline" />
      <Badge kind="tenBit" />
      <Badge kind="hdr" />
      <Badge kind="vfr" />
    </div>
  );
}

function Empty(): JSX.Element {
  const { formatMessage } = useIntl();
  return (
    <EmptyState
      icon={<Film size={24} aria-hidden="true" />}
      message={formatMessage({ id: "components.empty.message" })}
      actionMessageId="components.empty.action"
      onAction={() => {}}
      shortcut="Ctrl+I"
    />
  );
}

export const collectionStories: StoryFile = {
  id: "collections",
  stories: [
    { id: "collections.tree", render: () => <Trees /> },
    { id: "collections.grid-list", render: () => <Grids /> },
    { id: "collections.table", render: () => <Tables /> },
    { id: "collections.listbox", render: () => <Lists /> },
    { id: "collections.windowed", render: () => <Windowed /> },
    { id: "feedback.progress", render: () => <Progress /> },
    { id: "badges.all", render: () => <Badges /> },
    { id: "empty.media", render: () => <Empty /> },
  ],
};
