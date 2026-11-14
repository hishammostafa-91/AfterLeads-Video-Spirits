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
import {
  Children,
  createContext,
  useCallback,
  useContext,
  useEffect,
  useRef,
  useState,
} from "react";
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
} from "react-aria-components";
import { ChevronRight } from "../design/icons";
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
      {...props}
      role="listbox"
      className={classes}
      style={{ blockSize: totalHeight, overflow: "auto" }}
      tabIndex={0}
    >
      {/* Keyboard-scrollable window: the overflow container is the
          listbox and takes tabIndex=0 so keyboard users can scroll it
          (axe scrollable-region-focusable / WCAG 2.1.1). The layout
          wrapper is presentation-only so the options stay the
          listbox's semantic children (axe aria-required-children). */}
      {windowed && <div style={{ blockSize: offsetY }} aria-hidden="true" />}
      <div role="presentation" className={styles.options}>
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

export interface TreeWrapperProps {
  className?: string;
  children?: ReactNode;
  /** Accessible name of the tree. */
  "aria-label"?: string;
  /** Invoked with the item's textValue on activation (Enter/click). */
  onAction?: (key: string) => void;
}

interface TreeState {
  expanded: ReadonlySet<string>;
  toggle: (key: string) => void;
}

const TreeStateContext = createContext<TreeState | null>(null);

const isTreeItem = (node: ReactNode): boolean =>
  typeof node === "object" &&
  node !== null &&
  "type" in node &&
  (node as { type?: { treeItem?: boolean } }).type?.treeItem === true;

/**
 * A tree (bins and their clips) — `role="tree"` with the APG tree
 * keyboard pattern (arrows, Home/End, roving focus). Owner ruling
 * (option A, 2026-11-06): RAC 1.22.1's Tree crashes the page through
 * its hidden-`<template>` collection machinery under React 19
 * (createTextNode during commit — reproduced in Chromium, React
 * 19.2/19.3 alike), so this composes the semantics directly instead;
 * GridList/Table/ListBox stay on RAC.
 */
export function Tree({
  className,
  children,
  onAction,
  ...props
}: TreeWrapperProps): JSX.Element {
  const [expanded, setExpanded] = useState<ReadonlySet<string>>(
    () => new Set<string>(),
  );
  const treeRef = useRef<HTMLDivElement>(null);
  const toggle = useCallback((key: string): void => {
    setExpanded((prev) => {
      const next = new Set(prev);
      if (next.has(key)) {
        next.delete(key);
      } else {
        next.add(key);
      }
      return next;
    });
  }, []);

  // Roving tabindex: the first visible item is the tree's tab stop.
  useEffect(() => {
    const items =
      treeRef.current?.querySelectorAll<HTMLElement>("[role='treeitem']");
    if (!items) return;
    const visible = Array.from(items).filter(
      (el) => el.closest("[hidden]") === null,
    );
    items.forEach((item) => {
      item.tabIndex = -1;
    });
    if (visible[0]) visible[0].tabIndex = 0;
  }, [expanded]);

  // APG tree keyboard: Up/Down cycle the visible items, Right opens
  // (or descends), Left collapses (or ascends), Home/End jump.
  const onKeyDown = (event: React.KeyboardEvent<HTMLDivElement>): void => {
    const items = Array.from(
      event.currentTarget.querySelectorAll<HTMLElement>("[role='treeitem']"),
    ).filter((el) => el.closest("[hidden]") === null);
    const index = items.indexOf(document.activeElement as HTMLElement);
    if (index === -1) return;
    const current = items[index];
    const key = current.dataset.treeKey ?? "";
    const isExpanded = current.getAttribute("aria-expanded") === "true";
    if (event.key === "ArrowDown") {
      event.preventDefault();
      items[(index + 1) % items.length]?.focus();
    } else if (event.key === "ArrowUp") {
      event.preventDefault();
      items[(index - 1 + items.length) % items.length]?.focus();
    } else if (event.key === "ArrowRight") {
      event.preventDefault();
      if (current.getAttribute("aria-expanded") && !isExpanded) {
        toggle(key);
      } else {
        items[index + 1]?.focus();
      }
    } else if (event.key === "ArrowLeft") {
      event.preventDefault();
      if (current.getAttribute("aria-expanded") && isExpanded) {
        toggle(key);
      } else {
        const group = current.parentElement?.closest("[role='treeitem']");
        (group as HTMLElement | null)?.focus();
      }
    } else if (event.key === "Home") {
      event.preventDefault();
      items[0]?.focus();
    } else if (event.key === "End") {
      event.preventDefault();
      items[items.length - 1]?.focus();
    } else if (event.key === "Enter") {
      event.preventDefault();
      onAction?.(key);
    }
  };

  return (
    <TreeStateContext.Provider value={{ expanded, toggle }}>
      <div
        ref={treeRef}
        role="tree"
        aria-label={props["aria-label"]}
        className={cx(styles.tree, className)}
        // Programmatic focus target (roving tabindex lives on the
        // treeitems; the tree container itself is not a tab stop).
        tabIndex={-1}
        onKeyDown={onKeyDown}
      >
        {children}
      </div>
    </TreeStateContext.Provider>
  );
}

export interface TreeItemWrapperProps {
  /** The item's value (returned by the tree's onAction). */
  textValue: string;
  className?: string;
  children?: ReactNode;
}

const TreeLevelContext = createContext<{ level: number }>({ level: 1 });

function TreeItemBase({
  textValue,
  className,
  children,
}: TreeItemWrapperProps): JSX.Element {
  const { level } = useContext(TreeLevelContext);
  const state = useContext(TreeStateContext);
  // React.Children.toArray flattens and drops empties (Array.isArray
  // narrows ReactNode to any[], tripping no-unsafe-assignment).
  const childNodes: ReactNode[] = Children.toArray(children);
  const nested = childNodes.filter((child) => isTreeItem(child));
  const labelNodes = childNodes.filter((child) => !isTreeItem(child));
  const hasChildren = nested.length > 0;
  const isOpen = state?.expanded.has(textValue) ?? false;
  return (
    <div className={styles.treeNode}>
      <div
        role="treeitem"
        aria-level={level}
        aria-selected={false}
        aria-expanded={hasChildren ? isOpen : undefined}
        data-tree-key={textValue}
        tabIndex={-1}
        className={cx(styles.item, className)}
        onClick={(event) => {
          if (hasChildren) {
            state?.toggle(textValue);
          }
          event.currentTarget.focus();
        }}
        onKeyDown={(event) => {
          // Enter/Space mirror the click (toggle); Enter additionally
          // bubbles to the tree's activation handler (onAction).
          if (event.key === "Enter" || event.key === " ") {
            event.preventDefault();
            if (hasChildren) {
              state?.toggle(textValue);
            }
          }
        }}
      >
        {hasChildren && (
          <ChevronRight
            className={styles.treeChevron}
            aria-hidden="true"
            size={12}
          />
        )}
        <span className={styles.itemLabel}>{labelNodes}</span>
      </div>
      {hasChildren && (
        <div role="group" hidden={!isOpen} className={styles.treeGroup}>
          <TreeLevelContext.Provider value={{ level: level + 1 }}>
            {nested}
          </TreeLevelContext.Provider>
        </div>
      )}
    </div>
  );
}

/** One tree node; nested TreeItems form its subtree. */
export const TreeItem = Object.assign(TreeItemBase, {
  treeItem: true as const,
});

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
