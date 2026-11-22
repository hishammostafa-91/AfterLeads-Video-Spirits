// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * The E1007 recovery surface (FR-013 AC3): `project_open` rejected
 * because another holder keeps the lock - the envelope's two
 * documented recovery actions ("Open read-only" / "Take over", 12
 * s12.13) plus Cancel. Take-over only shows when offered (dead or
 * expired holder, 03 s3.9.4).
 */
import { useIntl } from "react-intl";
import type { JSX } from "react";
import { Dialog, Modal, ModalOverlay } from "react-aria-components";
import { Button } from "../components/Button";
import type { LockConflict } from "../projects/store";
import styles from "./LockConflict.module.css";

export interface LockConflictDialogProps {
  conflict: LockConflict | null;
  /** Open accepting the lock (read-only). */
  onOpenReadonly: (path: string) => void;
  /** The confirmed take-over. */
  onTakeOver: (path: string) => void;
  onCancel: () => void;
}

export function LockConflictDialog({
  conflict,
  onOpenReadonly,
  onTakeOver,
  onCancel,
}: LockConflictDialogProps): JSX.Element {
  const { formatMessage } = useIntl();
  return (
    <ModalOverlay isOpen={conflict !== null} isDismissable={false}>
      <Modal>
        <Dialog
          className={styles.dialog}
          aria-label={formatMessage({ id: "dlg.lock.title" })}
        >
          <LockConflictBody
            takeoverAvailable={conflict?.takeoverAvailable === true}
            onOpenReadonly={() => {
              if (conflict) onOpenReadonly(conflict.path);
            }}
            onTakeOver={() => {
              if (conflict) onTakeOver(conflict.path);
            }}
            onCancel={onCancel}
          />
        </Dialog>
      </Modal>
    </ModalOverlay>
  );
}

/** The dialog content without the overlay (harness stories). */
export function LockConflictBody({
  takeoverAvailable,
  onOpenReadonly,
  onTakeOver,
  onCancel,
}: {
  takeoverAvailable: boolean;
  onOpenReadonly: () => void;
  onTakeOver: () => void;
  onCancel: () => void;
}): JSX.Element {
  const { formatMessage } = useIntl();
  return (
    <>
      <h2 className={styles.title}>
        {formatMessage({ id: "dlg.lock.title" })}
      </h2>
      <p className={styles.body}>{formatMessage({ id: "dlg.lock.body" })}</p>
      <div className={styles.actions}>
        <Button variant="secondary" onPress={onCancel}>
          {formatMessage({ id: "dlg.lock.cancel" })}
        </Button>
        <Button variant="secondary" onPress={onOpenReadonly}>
          {formatMessage({ id: "dlg.lock.openReadonly" })}
        </Button>
        {takeoverAvailable ? (
          <Button variant="primary" onPress={onTakeOver}>
            {formatMessage({ id: "dlg.lock.takeOver" })}
          </Button>
        ) : null}
      </div>
    </>
  );
}
