// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * Menu family (07 §7.6; research D6): Menu, Submenu, and ContextMenu
 * composed over React Aria's MenuTrigger/Menu, plus MenuBar — a
 * `role="menubar"` composition implementing the ARIA APG menubar
 * keyboard pattern (RAC 1.22 ships no MenuBar primitive). Keyboard
 * shortcuts align to the inline end (right in LTR, left in RTL) via
 * logical properties.
 */
import type { JSX, ReactNode } from "react";
import type React from "react";
import { useRef, useState } from "react";
import {
  Button as RACButton,
  type ButtonProps as RACButtonProps,
  Menu as RACMenu,
  type MenuProps,
  MenuItem as RACMenuItem,
  type MenuItemProps,
  MenuTrigger,
  SubmenuTrigger,
} from "react-aria-components";
import {
  Check,
  ChevronRight,
  EllipsisVertical,
  ICON_SIZE_DENSE,
  ICON_STROKE_WIDTH,
} from "../design/icons";
import { useIntl } from "react-intl";
import styles from "./Menu.module.css";

function cx(...parts: Array<string | undefined>): string {
  return parts.filter(Boolean).join(" ");
}

export interface MenuWrapperProps extends Omit<MenuProps<object>, "className"> {
  className?: string;
  children?: ReactNode;
}

/** A styled dropdown menu (render inside a MenuTrigger). */
export function Menu({
  className,
  children,
  ...props
}: MenuWrapperProps): JSX.Element {
  return (
    <RACMenu {...props} className={cx(styles.menu, className)}>
      {children}
    </RACMenu>
  );
}

export interface MenuItemWrapperProps extends Omit<MenuItemProps, "className"> {
  className?: string;
  /** Keyboard shortcut glyph, e.g. "Ctrl+Z" (KB catalog; not translated). */
  shortcut?: string;
  children?: ReactNode;
}

/** A menu item with an optional end-aligned shortcut hint. */
export function MenuItem({
  shortcut,
  className,
  children,
  ...props
}: MenuItemWrapperProps): JSX.Element {
  return (
    <RACMenuItem {...props} className={cx(styles.item, className)}>
      {children}
      {shortcut && (
        <kbd className={styles.shortcut} aria-hidden="true">
          {shortcut}
        </kbd>
      )}
    </RACMenuItem>
  );
}

export interface MenuItemCheckboxProps extends Omit<
  MenuItemProps,
  "className" | "children"
> {
  /** The checked state (owned by the parent; toggled via onAction). */
  isChecked: boolean;
  className?: string;
  children?: ReactNode;
}

/**
 * A checkable menu item (07 §7.6 Menu row: "checkable items for
 * toggles"). RAC 1.22 ships no menuitemcheckbox, so the role and
 * aria-checked ride the render element (D6-style composition, the
 * MenuBar/Tree precedent); the check glyph mirrors to the inline
 * start and is decorative — the state lives in aria-checked.
 */
export function MenuItemCheckbox({
  isChecked,
  className,
  children,
  ...props
}: MenuItemCheckboxProps): JSX.Element {
  return (
    <RACMenuItem
      {...props}
      render={(renderProps) => (
        <div
          // RAC's render props allow function-valued style/className
          // (render state); only the DOM-safe subset applies here.
          {...(renderProps as React.ComponentProps<"div">)}
          role="menuitemcheckbox"
          aria-checked={isChecked}
        />
      )}
      className={cx(styles.item, className)}
    >
      <span className={styles.checkSlot} aria-hidden="true">
        {isChecked && (
          <Check
            size={ICON_SIZE_DENSE}
            strokeWidth={ICON_STROKE_WIDTH}
            aria-hidden="true"
          />
        )}
      </span>
      {children}
    </RACMenuItem>
  );
}

export interface SubmenuProps {
  /** The submenu's trigger label. */
  label: string;
  children: ReactNode;
}

/** A nested menu (RAC SubmenuTrigger + Menu). */
export function Submenu({ label, children }: SubmenuProps): JSX.Element {
  return (
    <SubmenuTrigger>
      <RACMenuItem className={styles.item}>
        <span className={styles.itemLabel}>{label}</span>
        <ChevronRight
          className={styles.submenuArrow}
          aria-hidden="true"
          size={ICON_SIZE_DENSE}
          strokeWidth={ICON_STROKE_WIDTH}
        />
      </RACMenuItem>
      <RACMenu className={styles.menu}>{children}</RACMenu>
    </SubmenuTrigger>
  );
}

export interface ContextMenuProps {
  /** The content the user right-clicks on. */
  children: ReactNode;
  /** The menu items shown on right-click. */
  items: ReactNode;
}

/**
 * A context menu: right-click (contextmenu event) opens the menu at
 * the pointer position; a kebab trigger inside the area gives
 * keyboard users the same menu (FR-012 — every action reachable);
 * Esc or a click outside closes it (RAC MenuTrigger with a
 * controlled zero-size anchor at the target point).
 */
export function ContextMenu({
  children,
  items,
}: ContextMenuProps): JSX.Element {
  const [open, setOpen] = useState(false);
  const [point, setPoint] = useState({ x: 0, y: 0 });
  const areaRef = useRef<HTMLDivElement>(null);
  const { formatMessage } = useIntl();

  return (
    <div
      ref={areaRef}
      className={styles.contextArea}
      onContextMenu={(event) => {
        event.preventDefault();
        const rect = areaRef.current?.getBoundingClientRect();
        setPoint({
          x: event.clientX - (rect?.left ?? 0),
          y: event.clientY - (rect?.top ?? 0),
        });
        setOpen(true);
      }}
    >
      {children}
      <MenuTrigger isOpen={open} onOpenChange={setOpen}>
        <span
          className={styles.contextAnchor}
          style={{ insetInlineStart: point.x, top: point.y }}
          aria-hidden="true"
        />
        <RACMenu className={styles.menu}>{items}</RACMenu>
      </MenuTrigger>
      <button
        type="button"
        className={styles.contextKebab}
        aria-label={formatMessage({ id: "components.menu.open" })}
        onClick={(event) => {
          const rect = event.currentTarget.getBoundingClientRect();
          const area = areaRef.current?.getBoundingClientRect();
          setPoint({
            x: rect.left - (area?.left ?? 0) + rect.width / 2,
            y: rect.top - (area?.top ?? 0) + rect.height / 2,
          });
          setOpen(true);
        }}
      >
        <EllipsisVertical
          size={ICON_SIZE_DENSE}
          strokeWidth={ICON_STROKE_WIDTH}
          aria-hidden="true"
        />
      </button>
    </div>
  );
}

export interface MenuBarProps {
  /** Message ID for the menubar's accessible name. */
  labelMessageId: string;
  className?: string;
  children: ReactNode;
}

/**
 * A menu bar: `role="menubar"` whose items implement the APG menubar
 * keyboard pattern — ArrowLeft/ArrowRight cycle focus across the
 * top-level items (with wrap), Home/End jump to the first/last item;
 * Enter/Space/ArrowDown open the focused item's menu (RAC), Esc
 * closes and returns focus (RAC). The keys are handled on the focused
 * item (focus lives on the items, per APG) and bubble from there.
 */
export function MenuBar({
  labelMessageId,
  className,
  children,
}: MenuBarProps): JSX.Element {
  const { formatMessage } = useIntl();
  return (
    <div
      role="menubar"
      aria-label={formatMessage({ id: labelMessageId })}
      className={cx(styles.menuBar, className)}
    >
      {children}
    </div>
  );
}

/** The APG menubar arrow/Home/End navigation, run from a focused item. */
function menubarItemKeyDown(
  event: React.KeyboardEvent<HTMLButtonElement>,
): void {
  const container = event.currentTarget.closest('[role="menubar"]');
  if (!container) return;
  const triggers = Array.from(
    container.querySelectorAll<HTMLButtonElement>(`.${styles.menuBarTrigger}`),
  );
  const index = triggers.indexOf(event.currentTarget);
  if (index === -1) return;
  if (event.key === "ArrowRight" || event.key === "ArrowLeft") {
    event.preventDefault();
    const delta = event.key === "ArrowRight" ? 1 : -1;
    triggers[(index + delta + triggers.length) % triggers.length]?.focus();
  } else if (event.key === "Home") {
    event.preventDefault();
    triggers[0]?.focus();
  } else if (event.key === "End") {
    event.preventDefault();
    triggers[triggers.length - 1]?.focus();
  }
}

export interface MenuBarItemProps extends Omit<RACButtonProps, "className"> {
  /** The trigger's visible label (already translated). */
  label: ReactNode;
  className?: string;
  children: ReactNode;
}

/** One top-level menubar entry: a trigger (role=menuitem) + its menu. */
export function MenuBarItem({
  label,
  className,
  children,
  ...props
}: MenuBarItemProps): JSX.Element {
  return (
    <MenuTrigger>
      <RACButton
        {...props}
        render={(renderProps) => (
          <button {...renderProps} type="button" role="menuitem" />
        )}
        onKeyDown={menubarItemKeyDown}
        className={cx(styles.menuBarTrigger, className)}
      >
        {label}
      </RACButton>
      <RACMenu className={styles.menu}>{children}</RACMenu>
    </MenuTrigger>
  );
}
