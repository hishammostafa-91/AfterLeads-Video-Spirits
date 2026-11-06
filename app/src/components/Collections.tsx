// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * Collections (07 §7.6): Tree, GridList, Table, and ListBox on React
 * Aria Components, plus the custom `useWindowing` hook — the
 * no-new-dependency virtualization (research D10) that renders a
 * window of rows once a collection grows beyond 200 rows, with
 * aria-setsize/aria-posinset preserving the full list semantics.
 */
import type { JSX, ReactNode } from "react";
import { useCallback, useRef, useState } from "react";
import {
  Cell as RACCell,
  Column as RACColumn,
  GridList as RACGridList,
  type GridListItemProps,
  type GridListProps,
  GridListItem as RACGridListItem,
  ListBox as RACListBox,
  type ListBoxProps,
  ListBoxItem as RACListBoxItem,
  Row as RACRow,
  Table as RACTable,
  type TableProps,
  TableBody as RACTableBody,
  TableHeader as RACTableHeader,
  Tree as RACTree,
  type TreeItemProps,
  type TreeProps,
  TreeItem as RACTreeItem,
} from "react-aria-components";
import styles from "./Collections.module.css";

function cx(...parts: Array<string | undefined>): string {
  return parts.filter(Boolean).join(" ");
}

/* -------------------------------------------------------------------- *
 * useWindowing (research D10)
 * -------------------------------------------------------------------- */

export interface WindowingOptions {
  /** Fixed row height in pixels. */
  rowHeight: number;
  /** Rows rendered beyond the viewport on each side. */
  overscan?: number;
  /** Virtualization starts above this many rows (07 §7.6: 200). */
  threshold?: number;
}

export interface WindowingResult<T> {
  /** Whether windowing is active (items.length above the threshold). */
  windowed: boolean;
  /** The slice of items to render. */
  rendered: readonly T[];
  /** Index of the first rendered item within the full list. */
  startIndex: number;
  /** Total height of the full list, for the scroll container. */
  totalHeight: number;
  /** Offset of the rendered slice, for the spacer. */
  offsetY: number;
  /** Props for the scroll container element. */
  containerProps: {
    ref: (node: HTMLElement | null) => void;
    onScroll: (event: React.UIEvent<HTMLElement>) => void;
  };
}

/**
 * Render-window virtualization: below the threshold every row renders
 * (the collection components' own semantics); above it, only the
 * viewport window (plus overscan) renders inside a spacer-sized
 * container. `getClientRect` failures (jsdom: zero-size elements)
 * fall back to a fixed small window so the boundary is testable.
 */
export function useWindowing<T>(
  items: readonly T[],
  { rowHeight, overscan = 5, threshold = 200 }: WindowingOptions,
): WindowingResult<T> {
  const [node, setNode] = useState<HTMLElement | null>(null);
  const [scrollTop, setScrollTop] = useState(0);
  const nodeRef = useRef<HTMLElement | null>(null);

  const ref = useCallback((next: HTMLElement | null) => {
    nodeRef.current = next;
    setNode(next);
  }, []);
  const onScroll = useCallback((event: React.UIEvent<HTMLElement>) => {
    setScrollTop(event.currentTarget.scrollTop);
  }, []);

  const total = items.length;
  const windowed = total > threshold;
  if (!windowed) {
    return {
      windowed: false,
      rendered: items,
      startIndex: 0,
      totalHeight: total * rowHeight,
      offsetY: 0,
      containerProps: { ref, onScroll },
    };
  }

  const viewport = node?.clientHeight ?? 0;
  // jsdom (and hidden containers) report 0 — use a deterministic
  // fallback window so the boundary stays testable everywhere.
  const visibleRows = viewport > 0 ? Math.ceil(viewport / rowHeight) : 10;
  const rawStart = Math.floor(scrollTop / rowHeight) - overscan;
  const startIndex = Math.max(0, rawStart);
  const endIndex = Math.min(total, startIndex + visibleRows + overscan * 2);
  return {
    windowed: true,
    rendered: items.slice(startIndex, endIndex),
    startIndex,
    totalHeight: total * rowHeight,
    offsetY: startIndex * rowHeight,
    containerProps: { ref, onScroll },
  };
}

/* -------------------------------------------------------------------- *
 * WindowedListBox — the windowed collection pattern
 * -------------------------------------------------------------------- */

export interface WindowedListBoxProps<T> {
  items: readonly T[];
  /** Renders one option's content from its item. */
  renderOption: (item: T) => ReactNode;
  /** Derives each option's value/id from its item. */
  keyOf: (item: T) => string;
  /** Accessible name of the listbox. */
  "aria-label"?: string;
  rowHeight?: number;
  className?: string;
}

/**
 * A listbox that renders every option below the windowing threshold
 * and only the scroll window above it (aria-setsize/aria-posinset
 * keep the full list semantics for assistive technology).
 */
export function WindowedListBox<T>({
  items,
  renderOption,
  keyOf,
  rowHeight = 28,
  className,
  ...props
}: WindowedListBoxProps<T>): JSX.Element {
  const {
    windowed,
    rendered,
    startIndex,
    totalHeight,
    offsetY,
    containerProps,
  } = useWindowing(items, { rowHeight });
  const classes = cx(styles.windowedList, className);
  return (
    <div
      {...containerProps}
      className={classes}
      style={{ blockSize: totalHeight, overflow: "auto" }}
    >
      {windowed && <div style={{ blockSize: offsetY }} aria-hidden="true" />}
      <div role="listbox" className={styles.options} {...props}>
        {rendered.map((item, i) => (
          <div
            key={keyOf(item)}
            role="option"
            aria-selected={false}
            aria-setsize={items.length}
            aria-posinset={startIndex + i + 1}
            className={styles.option}
          >
            {renderOption(item)}
          </div>
        ))}
      </div>
    </div>
  );
}

/* -------------------------------------------------------------------- *
 * RAC collection wrappers
 * -------------------------------------------------------------------- */

export interface TreeWrapperProps<T> extends Omit<TreeProps<T>, "className"> {
  className?: string;
  children?: ReactNode;
}

/** A tree (bins and their clips). */
export function Tree<T extends object>({
  className,
  children,
  ...props
}: TreeWrapperProps<T>): JSX.Element {
  return (
    <RACTree {...props} className={cx(styles.tree, className)}>
      {children}
    </RACTree>
  );
}

export interface TreeItemWrapperProps extends Omit<TreeItemProps, "className"> {
  className?: string;
}

export function TreeItem({
  className,
  children,
  ...props
}: TreeItemWrapperProps): JSX.Element {
  return (
    <RACTreeItem {...props} className={cx(styles.item, className)}>
      {children}
    </RACTreeItem>
  );
}

export interface GridListWrapperProps<T> extends Omit<
  GridListProps<T>,
  "className"
> {
  className?: string;
  children?: ReactNode;
}

/** A grid list (media items in a bin). */
export function GridList<T extends object>({
  className,
  children,
  ...props
}: GridListWrapperProps<T>): JSX.Element {
  return (
    <RACGridList {...props} className={cx(styles.gridList, className)}>
      {children}
    </RACGridList>
  );
}

export interface GridListItemWrapperProps extends Omit<
  GridListItemProps,
  "className"
> {
  className?: string;
}

export function GridListItem({
  className,
  children,
  ...props
}: GridListItemWrapperProps): JSX.Element {
  return (
    <RACGridListItem {...props} className={cx(styles.item, className)}>
      {children}
    </RACGridListItem>
  );
}

export interface TableWrapperProps extends Omit<TableProps, "className"> {
  className?: string;
  children?: ReactNode;
}

/** A data table. */
export function Table({
  className,
  children,
  ...props
}: TableWrapperProps): JSX.Element {
  return (
    <RACTable {...props} className={cx(styles.table, className)}>
      {children}
    </RACTable>
  );
}

export {
  RACTableHeader as TableHeader,
  RACTableBody as TableBody,
  RACColumn as TableColumn,
  RACRow as TableRow,
  RACCell as TableCell,
};

export interface ListBoxWrapperProps<T> extends Omit<
  ListBoxProps<T>,
  "className"
> {
  className?: string;
  children?: ReactNode;
}

/** A plain listbox (short option lists). */
export function ListBox<T extends object>({
  className,
  children,
  ...props
}: ListBoxWrapperProps<T>): JSX.Element {
  return (
    <RACListBox {...props} className={cx(styles.options, className)}>
      {children}
    </RACListBox>
  );
}

export { RACListBoxItem as ListBoxOption };
