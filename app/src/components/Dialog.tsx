// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * Overlays (07 §7.6): Dialog and AlertDialog (focus trap, focus
 * return, and Esc on dismissable prompts — RAC Modal guarantees;
 * AlertDialog focuses the safest control first), and Popover. RAC
 * 1.22 ships no AlertDialog primitive, so it is Dialog with
 * role="alertdialog" (research D6).
 */
import type { JSX, ReactNode } from "react";
import {
  Dialog as RACDialog,
  type DialogProps,
  Heading,
  Modal,
  ModalOverlay,
  Popover as RACPopover,
  type PopoverProps,
} from "react-aria-components";
import { Button } from "./Button";
import styles from "./Dialog.module.css";

function cx(...parts: Array<string | undefined>): string {
  return parts.filter(Boolean).join(" ");
}

export interface DialogWrapperProps extends Omit<DialogProps, "className"> {
  className?: string;
  children?: ReactNode | ((opts: { close: () => void }) => ReactNode);
}

/**
 * A modal dialog: focus is trapped inside, Esc closes (when the
 * overlay is dismissable), and focus returns to the trigger on close
 * (RAC Modal semantics). Unsaved-changes prompts render it
 * non-dismissable via the overlay's `isDismissable={false}`.
 */
export function Dialog({
  className,
  children,
  ...props
}: DialogWrapperProps): JSX.Element {
  return (
    <RACDialog {...props} className={cx(styles.dialog, className)}>
      {children}
    </RACDialog>
  );
}

export { Heading as DialogHeading, Modal, ModalOverlay };

export interface AlertDialogProps {
  /** Already-translated heading. */
  title: ReactNode;
  /** Already-translated body sentence. */
  children: ReactNode;
  /** Already-translated cancel label (receives initial focus — the safest control). */
  cancelLabel: ReactNode;
  /** Already-translated default-action label. */
  actionLabel: ReactNode;
  /** Invoked when the default action is chosen (receives `close`). */
  onAction?: (close: () => void) => void;
}

/**
 * An alert dialog: `role="alertdialog"` with initial focus on the
 * cancel (safest) control; the default action is never focused first.
 */
export function AlertDialog({
  title,
  children,
  cancelLabel,
  actionLabel,
  onAction,
}: AlertDialogProps): JSX.Element {
  return (
    <RACDialog role="alertdialog" className={styles.dialog}>
      {({ close }) => (
        <>
          <Heading slot="title" className={styles.title}>
            {title}
          </Heading>
          <p className={styles.body}>{children}</p>
          <div className={styles.buttonRow}>
            {/* 07 §7.6: the alert dialog's initial focus lands on the
                safest control — the cancel button. Initial focus
                inside a modal is the WCAG 2.4.3 pattern, so the
                generic no-autofocus rule is scoped off here. */}
            {/* eslint-disable-next-line jsx-a11y/no-autofocus */}
            <Button variant="secondary" autoFocus onPress={close}>
              {cancelLabel}
            </Button>
            <Button
              variant="danger"
              onPress={() => {
                onAction?.(close);
                close();
              }}
            >
              {actionLabel}
            </Button>
          </div>
        </>
      )}
    </RACDialog>
  );
}

export interface PopoverWrapperProps extends Omit<PopoverProps, "className"> {
  className?: string;
  children?: ReactNode;
}

/** A popover panel (render as a DialogTrigger child). */
export function Popover({
  className,
  children,
  ...props
}: PopoverWrapperProps): JSX.Element {
  return (
    <RACPopover {...props} className={cx(styles.popover, className)}>
      {children}
    </RACPopover>
  );
}
