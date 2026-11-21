// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * The command palette (REQ-APP-004; 12 §12.10 KB-131,
 * Ctrl+Shift+P): fuzzy search over every catalog command in the
 * active locale, keyboard-navigable results (RAC ListBox arrows +
 * Enter), shortcut display in the current layout's glyphs
 * (REQ-I18N-005), and later-version commands visible but not
 * runnable with their note (WP-05 card). The body is a standalone
 * component so the harness story renders it inline (the modal is
 * fixed-position and cannot sit in the story grid).
 */
import { useEffect, useMemo, useState } from "react";
import { useIntl } from "react-intl";
import {
  Dialog,
  DialogHeading,
  Modal,
  ModalOverlay,
} from "../../components/Dialog";
import { ListBox, ListBoxOption } from "../../components/Collections";
import { COMMANDS, type CommandSpec, type Shortcut } from "../catalog";
import {
  chordLabel,
  layoutMapSupported,
  type LayoutMapLike,
} from "../keyLabel";
import { commandRow, runCommand, type CommandRuntimeFlags } from "../registry";
import { fuzzyRank } from "./fuzzy";
import styles from "./CommandPalette.module.css";

/** The browser keyboard layout map, when available (async, once). */
function useKeyboardLayoutMap(): LayoutMapLike | null {
  const [map, setMap] = useState<LayoutMapLike | null>(null);
  useEffect(() => {
    if (!layoutMapSupported()) return;
    // navigator.keyboard is not in this tsconfig's DOM lib slice; the
    // support check above guarantees the runtime shape.
    const keyboard = (
      navigator as unknown as {
        keyboard?: { getLayoutMap?: () => Promise<LayoutMapLike> };
      }
    ).keyboard;
    const getLayoutMap = keyboard?.getLayoutMap;
    if (!getLayoutMap) return;
    let cancelled = false;
    getLayoutMap()
      .then((layout) => {
        if (!cancelled) setMap(layout);
      })
      .catch(() => {
        // Unsupported or denied — US labels apply.
      });
    return () => {
      cancelled = true;
    };
  }, []);
  return map;
}

export interface CommandPaletteBodyProps {
  flags: CommandRuntimeFlags;
  /** Called after a command ran (the modal closes on it). */
  onRan?: () => void;
}

const RESULT_LIMIT = 50;

/** Heading + search field + ranked results; owns the query state. */
export function CommandPaletteBody({
  flags,
  onRan,
}: CommandPaletteBodyProps): React.JSX.Element {
  const { formatMessage } = useIntl();
  const [query, setQuery] = useState("");
  const layoutMap = useKeyboardLayoutMap();

  const entries = useMemo(
    () =>
      COMMANDS.map((spec, order) => ({
        haystack: `${formatMessage({ id: spec.labelKey })} ${spec.id}`,
        order,
        item: spec,
      })),
    [formatMessage],
  );
  const results = useMemo(
    () => fuzzyRank(query, entries).slice(0, RESULT_LIMIT),
    [query, entries],
  );

  const label = (spec: CommandSpec): string =>
    formatMessage({ id: spec.labelKey });

  const run = (spec: CommandSpec): void => {
    const row = commandRow(spec, flags);
    if (!row.enabled) return;
    onRan?.();
    void runCommand(spec, flags);
  };

  return (
    <>
      <DialogHeading className={styles.heading}>
        {formatMessage({ id: "commands.palette.title" })}
      </DialogHeading>
      <input
        type="text"
        className={styles.input}
        value={query}
        onChange={(event) => {
          setQuery(event.target.value);
        }}
        aria-label={formatMessage({ id: "commands.palette.placeholder" })}
        placeholder={formatMessage({ id: "commands.palette.placeholder" })}
      />
      {results.length === 0 ? (
        <p className={styles.empty}>
          {formatMessage({ id: "commands.palette.noResults" })}
        </p>
      ) : (
        <ListBox
          aria-label={formatMessage({ id: "commands.palette.title" })}
          selectionMode="none"
          onAction={(key) => {
            const spec = COMMANDS.find((command) => command.id === key);
            if (spec) run(spec);
          }}
          className={styles.list}
        >
          {results.map(({ item: spec }) => {
            const row = commandRow(spec, flags);
            const shortcut: Shortcut | undefined =
              spec.shortcuts.length > 0 ? spec.shortcuts[0] : undefined;
            return (
              <ListBoxOption
                key={spec.id}
                id={spec.id}
                textValue={label(spec)}
                isDisabled={!row.enabled}
                className={styles.row}
              >
                <span className={styles.label}>{label(spec)}</span>
                {shortcut ? (
                  <kbd className={styles.kbd}>
                    {chordLabel(shortcut, layoutMap)}
                  </kbd>
                ) : null}
                {row.laterVersion ? (
                  <span className={styles.later}>
                    {formatMessage({ id: "commands.laterVersion" })}
                  </span>
                ) : null}
              </ListBoxOption>
            );
          })}
        </ListBox>
      )}
    </>
  );
}

export interface CommandPaletteProps {
  isOpen: boolean;
  onClose: () => void;
  flags: CommandRuntimeFlags;
}

export function CommandPalette({
  isOpen,
  onClose,
  flags,
}: CommandPaletteProps): React.JSX.Element {
  const { formatMessage } = useIntl();
  return (
    <ModalOverlay
      isOpen={isOpen}
      isDismissable
      onOpenChange={(open) => {
        if (!open) onClose();
      }}
    >
      <Modal className={styles.modal}>
        <Dialog
          className={styles.dialog}
          aria-label={formatMessage({ id: "commands.palette.title" })}
        >
          {/* key remounts the body per open — the query resets. */}
          <CommandPaletteBody
            key={isOpen ? "open" : "closed"}
            flags={flags}
            onRan={onClose}
          />
        </Dialog>
      </Modal>
    </ModalOverlay>
  );
}
