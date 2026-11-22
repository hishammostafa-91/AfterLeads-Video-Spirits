// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * DLG-01 New Project (07 s7.14): name (required, 1-200 chars,
 * Windows-safe - validated inline per the doc rule), location
 * (folder picker, default the Videos folder), sequence preset
 * (SQP-01..12, Auto default), frame rate (FPS-01..08), sample rate
 * (48 kHz default). Create / Cancel. Backend errors (E1008, E8004)
 * surface as DLG-level messages with their recovery actions (07
 * s7.10 style).
 */
import { useIntl } from "react-intl";
import { useState } from "react";
import type { JSX } from "react";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { Dialog, Modal, ModalOverlay } from "react-aria-components";
import { Button } from "../components/Button";
import { TextField } from "../components/Fields";
import { Select } from "../components/Selection";
import { ListBoxOption } from "../components/Collections";
import {
  DEFAULT_FRAME_RATE_ID,
  DEFAULT_PRESET_ID,
  DEFAULT_SAMPLE_RATE,
  FRAME_RATES,
  SAMPLE_RATES,
  SEQUENCE_PRESETS,
  frameRateLabelKey,
  presetById,
  presetLabelKey,
} from "../projects/presets";
import type { ErrorEnvelope } from "../ipc/bindings/ErrorEnvelope";
import styles from "./NewProject.module.css";

export interface NewProjectRequestShape {
  name: string;
  folder: string;
  sequencePreset: string;
  frameRate: string;
  sampleRate: number;
}

export interface NewProjectDialogProps {
  isOpen: boolean;
  /** The default location (the Videos folder). */
  defaultFolder: string;
  onClose: () => void;
  /** Create: resolves true when the project opened. */
  onCreate: (request: NewProjectRequestShape) => Promise<boolean>;
  /** The last backend envelope, if any (E1008/E8004). */
  error: ErrorEnvelope | null;
}

/** The DLG-01 name rule (mirrors the backend's validation: the
 * length counts Unicode code points exactly like Rust's `chars()`). */
export function nameError(name: string): string | null {
  // eslint-disable-next-line @typescript-eslint/no-misused-spread -- code points, matching the backend's Rust `chars().count()`
  const length = [...name].length;
  if (length < 1 || length > 200) {
    return "dlg.newProject.nameLength";
  }
  if (name === "." || name === "..") {
    return "dlg.newProject.nameReserved";
  }
  if (/[<>:"/\\|?*\u0000-\u001f]/.test(name)) {
    return "dlg.newProject.nameChars";
  }
  if (/[. ]$/.test(name)) {
    return "dlg.newProject.nameTrailing";
  }
  return null;
}

export function NewProjectDialog({
  isOpen,
  defaultFolder,
  onClose,
  onCreate,
  error,
}: NewProjectDialogProps): JSX.Element {
  const { formatMessage } = useIntl();
  return (
    <ModalOverlay
      isDismissable
      isOpen={isOpen}
      onOpenChange={(open) => {
        if (!open) onClose();
      }}
    >
      <Modal>
        <Dialog
          className={styles.dialog}
          aria-label={formatMessage({ id: "dlg.newProject.title" })}
        >
          <NewProjectBody
            defaultFolder={defaultFolder}
            error={error}
            onCreate={onCreate}
            onDone={onClose}
          />
        </Dialog>
      </Modal>
    </ModalOverlay>
  );
}

/** The dialog content without the overlay (harness stories embed
 * this; fixed overlays render outside story cards). */
export function NewProjectBody({
  defaultFolder,
  error,
  onCreate,
  onDone,
}: {
  defaultFolder: string;
  error: ErrorEnvelope | null;
  onCreate: (request: NewProjectRequestShape) => Promise<boolean>;
  onDone: () => void;
}): JSX.Element {
  const { formatMessage } = useIntl();
  const [name, setName] = useState("");
  const [folder, setFolder] = useState(defaultFolder);
  const [preset, setPreset] = useState(DEFAULT_PRESET_ID);
  const [fps, setFps] = useState(DEFAULT_FRAME_RATE_ID);
  const [sampleRate, setSampleRate] = useState(DEFAULT_SAMPLE_RATE);
  const [busy, setBusy] = useState(false);
  const [touched, setTouched] = useState(false);
  const nameIssue = touched ? nameError(name) : null;
  const createDisabled = busy || nameError(name) !== null;

  const create = async (): Promise<void> => {
    if (createDisabled) return;
    setBusy(true);
    const created = await onCreate({
      name,
      folder,
      sequencePreset: preset,
      frameRate: fps,
      sampleRate,
    });
    setBusy(false);
    if (created) onDone();
  };

  return (
    <>
      <h2 className={styles.title}>
        {formatMessage({ id: "dlg.newProject.title" })}
      </h2>
      <div className={styles.form}>
        <div>
          <TextField
            label={formatMessage({ id: "dlg.newProject.name" })}
            value={name}
            onChange={setName}
            onBlur={() => {
              setTouched(true);
            }}
            isInvalid={nameIssue !== null}
            // 07 s7.13 rule 5: dialogs focus the first field.
            // eslint-disable-next-line jsx-a11y/no-autofocus
            autoFocus
          />
          {nameIssue !== null ? (
            <p className={styles.error} role="alert">
              {formatMessage({ id: nameIssue })}
            </p>
          ) : null}
        </div>
        <div className={styles.location}>
          <TextField
            label={formatMessage({ id: "dlg.newProject.location" })}
            value={folder}
            onChange={setFolder}
          />
          <Button
            variant="secondary"
            onPress={() => {
              void openDialog({ directory: true }).then((picked) => {
                if (typeof picked === "string") setFolder(picked);
              });
            }}
          >
            {formatMessage({ id: "dlg.newProject.browse" })}
          </Button>
        </div>
        <Select
          label={formatMessage({ id: "dlg.newProject.preset" })}
          value={preset}
          onChange={(key) => {
            setPreset(String(key));
          }}
        >
          {SEQUENCE_PRESETS.map((entry) => (
            <ListBoxOption key={entry.id} id={entry.id}>
              {`${formatMessage({ id: presetLabelKey(entry.id) })}${
                entry.width !== null
                  ? ` - ${String(entry.width)}x${String(entry.height)}`
                  : ""
              }`}
            </ListBoxOption>
          ))}
        </Select>
        <div className={styles.row}>
          <Select
            label={formatMessage({ id: "dlg.newProject.frameRate" })}
            value={fps}
            onChange={(key) => {
              setFps(String(key));
            }}
          >
            {FRAME_RATES.map((rate) => (
              <ListBoxOption key={rate.id} id={rate.id}>
                {formatMessage({ id: frameRateLabelKey(rate.id) })}
              </ListBoxOption>
            ))}
          </Select>
          <Select
            label={formatMessage({ id: "dlg.newProject.sampleRate" })}
            value={String(sampleRate)}
            onChange={(key) => {
              setSampleRate(Number(key));
            }}
          >
            {SAMPLE_RATES.map((rate) => (
              <ListBoxOption key={rate} id={String(rate)}>
                {formatMessage(
                  { id: "dlg.newProject.hz" },
                  { khz: rate / 1000 },
                )}
              </ListBoxOption>
            ))}
          </Select>
        </div>
        {presetById(preset)?.autoMatch === true ? (
          <p className={styles.note}>
            {formatMessage({ id: "dlg.newProject.autoNote" })}
          </p>
        ) : null}
      </div>
      {error !== null ? (
        <p className={styles.error} role="alert">
          {formatMessage({ id: error.messageKey })}
          {error.recovery.length > 0
            ? ` - ${error.recovery
                .map((action) => formatMessage({ id: action.labelKey }))
                .join(" / ")}`
            : ""}
        </p>
      ) : null}
      <div className={styles.actions}>
        <Button variant="secondary" onPress={onDone}>
          {formatMessage({ id: "dlg.newProject.cancel" })}
        </Button>
        <Button
          variant="primary"
          isDisabled={createDisabled}
          onPress={() => {
            void create();
          }}
        >
          {busy
            ? formatMessage({ id: "dlg.newProject.creating" })
            : formatMessage({ id: "dlg.newProject.create" })}
        </Button>
      </div>
    </>
  );
}
