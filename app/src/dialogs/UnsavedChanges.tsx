// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * DLG-20 Unsaved Changes (07 s7.14): "Save changes to <project>?"
 * with Save / Don't save / Cancel. Not dismissable - the prompt
 * needs an explicit choice (07 s7.6 dialog rule).
 */
import { useIntl } from "react-intl";
import { useState } from "react";
import type { JSX } from "react";
import { Dialog, Modal, ModalOverlay } from "react-aria-components";
import { Button } from "../components/Button";
import styles from "./UnsavedChanges.module.css";

export type UnsavedChoice = "save" | "discard" | "cancel";

export interface UnsavedChangesDialogProps {
  isOpen: boolean;
  /** The project's display name. */
  projectName: string;
  /** Resolves the choice (the caller runs save-then-continue etc.). */
  onChoose: (choice: UnsavedChoice) => void;
}

export function UnsavedChangesDialog({
  isOpen,
  projectName,
  onChoose,
}: UnsavedChangesDialogProps): JSX.Element {
  const { formatMessage } = useIntl();
  return (
    <ModalOverlay isOpen={isOpen} isDismissable={false}>
      <Modal>
        <Dialog
          className={styles.dialog}
          aria-label={formatMessage(
            { id: "dlg.unsaved.title" },
            { name: projectName },
          )}
        >
          <UnsavedChangesBody projectName={projectName} onChoose={onChoose} />
        </Dialog>
      </Modal>
    </ModalOverlay>
  );
}

/** The dialog content without the overlay (harness stories). */
export function UnsavedChangesBody({
  projectName,
  onChoose,
}: {
  projectName: string;
  onChoose: (choice: UnsavedChoice) => void;
}): JSX.Element {
  const { formatMessage } = useIntl();
  const [busy] = useState(false);
  const choose = (choice: UnsavedChoice): void => {
    if (busy) return;
    onChoose(choice);
  };
  return (
    <>
      <h2 className={styles.title}>
        {formatMessage({ id: "dlg.unsaved.title" }, { name: projectName })}
      </h2>
      <div className={styles.actions}>
        <Button
          variant="secondary"
          isDisabled={busy}
          onPress={() => {
            choose("cancel");
          }}
        >
          {formatMessage({ id: "dlg.unsaved.cancel" })}
        </Button>
        <Button
          variant="danger"
          isDisabled={busy}
          onPress={() => {
            choose("discard");
          }}
        >
          {formatMessage({ id: "dlg.unsaved.dontSave" })}
        </Button>
        <Button
          variant="primary"
          isDisabled={busy}
          onPress={() => {
            choose("save");
          }}
        >
          {formatMessage({ id: "dlg.unsaved.save" })}
        </Button>
      </div>
    </>
  );
}
