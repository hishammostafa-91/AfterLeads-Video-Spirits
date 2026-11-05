// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * Overlay stories (07 §7.6; T019): Dialog, AlertDialog, Popover,
 * Tooltip, Toast.
 */
import { FormattedMessage } from "react-intl";
import type { JSX } from "react";
import { DialogTrigger, TooltipTrigger } from "react-aria-components";
import { Button } from "../../components/Button";
import {
  AlertDialog,
  Dialog,
  ModalOverlay,
  Popover,
} from "../../components/Dialog";
import { showToast, ToastRegion } from "../../components/Toast";
import { Tooltip, TOOLTIP_DELAY_MS } from "../../components/Tooltip";
import styles from "./stories.module.css";
import type { StoryFile } from "./types";

function DialogDemo(): JSX.Element {
  return (
    <DialogTrigger>
      <Button>
        <FormattedMessage id="components.dialog.open" />
      </Button>
      <ModalOverlay isDismissable>
        <Dialog>
          {({ close }) => (
            <>
              <h3>
                <FormattedMessage id="components.dialog.discardTitle" />
              </h3>
              <p>
                <FormattedMessage id="components.dialog.discardBody" />
              </p>
              <div className={styles.row}>
                <Button variant="primary" onPress={close}>
                  <FormattedMessage id="components.dialog.cancel" />
                </Button>
              </div>
            </>
          )}
        </Dialog>
      </ModalOverlay>
    </DialogTrigger>
  );
}

function AlertDialogDemo(): JSX.Element {
  return (
    <DialogTrigger>
      <Button variant="danger">
        <FormattedMessage id="components.dialog.discard" />
      </Button>
      <ModalOverlay isDismissable={false}>
        <AlertDialog
          title={<FormattedMessage id="components.dialog.discardTitle" />}
          cancelLabel={<FormattedMessage id="components.dialog.cancel" />}
          actionLabel={<FormattedMessage id="components.dialog.discard" />}
        >
          <FormattedMessage id="components.dialog.discardBody" />
        </AlertDialog>
      </ModalOverlay>
    </DialogTrigger>
  );
}

function PopoverDemo(): JSX.Element {
  return (
    <DialogTrigger>
      <Button>
        <FormattedMessage id="components.popover.filters" />
      </Button>
      <Popover>
        <FormattedMessage id="components.popover.body" />
      </Popover>
    </DialogTrigger>
  );
}

function TooltipDemo(): JSX.Element {
  return (
    <TooltipTrigger delay={TOOLTIP_DELAY_MS}>
      <Button>
        <FormattedMessage id="components.tooltip.zoomIn" />
      </Button>
      <Tooltip>
        <FormattedMessage id="components.tooltip.zoomIn" />
      </Tooltip>
    </TooltipTrigger>
  );
}

function ToastDemo(): JSX.Element {
  return (
    <div className={styles.row}>
      <Button
        onPress={() => {
          showToast({
            messageId: "components.toast.saved",
            variant: "success",
          });
        }}
      >
        <FormattedMessage id="components.toast.show" />
      </Button>
      <ToastRegion />
    </div>
  );
}

export const overlayStories: StoryFile = {
  id: "overlays",
  stories: [
    { id: "overlays.dialog", render: () => <DialogDemo /> },
    { id: "overlays.alert-dialog", render: () => <AlertDialogDemo /> },
    { id: "overlays.popover", render: () => <PopoverDemo /> },
    { id: "overlays.tooltip", render: () => <TooltipDemo /> },
    { id: "overlays.toast", render: () => <ToastDemo /> },
  ],
};
