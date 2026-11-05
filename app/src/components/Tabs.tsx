// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * Tabs (07 §7.6): the panel-tab primitive on RAC Tabs — keyboard
 * roving focus, arrow navigation, and selection follow focus come
 * from RAC. Workspace/dockview integration stays in WP-05.
 */
import type { JSX, ReactNode } from "react";
import {
  Tab as RACTab,
  TabList as RACTabList,
  type TabListProps,
  TabPanel as RACTabPanel,
  type TabPanelProps,
  Tabs as RACTabs,
  type TabsProps,
} from "react-aria-components";
import styles from "./Tabs.module.css";

function cx(...parts: Array<string | undefined>): string {
  return parts.filter(Boolean).join(" ");
}

export interface TabsWrapperProps extends Omit<TabsProps, "className"> {
  className?: string;
  children?: ReactNode;
}

/** The tab container (owns selection + keyboard). */
export function Tabs({
  className,
  children,
  ...props
}: TabsWrapperProps): JSX.Element {
  return (
    <RACTabs {...props} className={cx(styles.tabs, className)}>
      {children}
    </RACTabs>
  );
}

export interface TabListWrapperProps<T> extends Omit<
  TabListProps<T>,
  "className"
> {
  className?: string;
  children?: ReactNode;
}

/** The horizontal tab strip. */
export function TabList<T extends object>({
  className,
  children,
  ...props
}: TabListWrapperProps<T>): JSX.Element {
  return (
    <RACTabList {...props} className={cx(styles.tabList, className)}>
      {children}
    </RACTabList>
  );
}

export interface TabWrapperProps extends Omit<
  React.ComponentProps<typeof RACTab>,
  "className"
> {
  className?: string;
  children?: ReactNode;
}

/** One tab (id matches its TabPanel's id). */
export function Tab({
  className,
  children,
  ...props
}: TabWrapperProps): JSX.Element {
  return (
    <RACTab {...props} className={cx(styles.tab, className)}>
      {children}
    </RACTab>
  );
}

export interface TabPanelWrapperProps extends Omit<TabPanelProps, "className"> {
  className?: string;
  children?: ReactNode;
}

/** The panel a tab selects. */
export function TabPanel({
  className,
  children,
  ...props
}: TabPanelWrapperProps): JSX.Element {
  return (
    <RACTabPanel {...props} className={cx(styles.tabPanel, className)}>
      {children}
    </RACTabPanel>
  );
}
