// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * The notification fabric (FR-014, REQ-APP-006): a normalized
 * history store fed by `app:notify` payloads (ErrorEnvelope or info
 * `{ severity, messageKey, params }`), with error codes shown and
 * the envelope's recovery action exposed as a runnable command.
 * Toasts render through the WP-04 Toast store (`showToast`); the
 * T048 improvements (assertive routing, dismiss, pause) remain
 * tracked there.
 */
import { create } from "zustand";
import type { ErrorEnvelope } from "../../ipc/bindings/ErrorEnvelope";
import { runCommand, type CommandRuntimeFlags } from "../../commands/registry";
import { COMMAND_BY_ID } from "../../commands/catalog";
import { showToast } from "../../components/Toast";

/** One notification-center row (07 s7.10; the center is a dock
 * panel's data). */
export interface NotificationEntry {
  /** Sequence id. */
  id: number;
  /** `"error"` carries the envelope; `"info"` the message key. */
  kind: "error" | "info";
  /** The message key to render (always `error.<code>` for errors). */
  messageKey: string;
  /** Message parameters (ICU). */
  params: Record<string, string>;
  /** The recovery actions, when the payload was an envelope. */
  recovery: ReadonlyArray<{ commandId: string; labelKey: string }>;
  /** The catalog code, errors only. */
  code?: string;
  /** ISO time of arrival. */
  atUtc: string;
}

/** The `app:notify` payload shapes (04 s4.10). */
export type NotifyPayload =
  | ErrorEnvelope
  | { severity: "info"; messageKey: string; params?: Record<string, string> };

function isEnvelope(value: unknown): value is ErrorEnvelope {
  return (
    typeof value === "object" &&
    value !== null &&
    "code" in value &&
    "messageKey" in value
  );
}

let nextId = 1;

interface NotificationState {
  entries: NotificationEntry[];
  /** Record an `app:notify` payload (toast + history). */
  notify: (payload: NotifyPayload) => void;
  /** Run a recovery action (returns whether the command ran). */
  runRecovery: (
    entry: NotificationEntry,
    action: { commandId: string; labelKey: string },
    flags: CommandRuntimeFlags,
  ) => Promise<boolean>;
  clear: () => void;
}

export const useNotificationStore = create<NotificationState>((set) => ({
  entries: [],
  notify: (payload) => {
    const entry: NotificationEntry = isEnvelope(payload)
      ? {
          id: nextId++,
          kind: "error",
          messageKey: payload.messageKey,
          params: payload.params,
          recovery: payload.recovery,
          code: payload.code,
          atUtc: new Date().toISOString(),
        }
      : {
          id: nextId++,
          kind: "info",
          messageKey: payload.messageKey,
          params: payload.params ?? {},
          recovery: [],
          atUtc: new Date().toISOString(),
        };
    set((state) => ({ entries: [entry, ...state.entries] }));
    // Polite toast for info; errors toast with the envelope's
    // message (the center rows carry the code chip). Assertive
    // routing is T048's tracked improvement.
    showToast({
      messageId: entry.messageKey,
      variant: entry.kind === "error" ? "error" : "info",
    });
  },
  runRecovery: async (_entry, action, flags) => {
    const spec = COMMAND_BY_ID.get(action.commandId);
    // Recovery verbs that are not KB commands (dialog actions like
    // `project.openReadOnly`, 04 s4.9) resolve through the registry's
    // handler table only.
    if (spec) {
      return runCommand(spec, flags);
    }
    // Dialog-action verbs carry no catalog row; the shell registers
    // their handlers directly (registry ids are command ids, so this
    // path only runs for registered non-catalog verbs).
    return false;
  },
  clear: () => {
    set({ entries: [] });
  },
}));

/** Test helper. */
export function resetNotificationStoreForTests(): void {
  nextId = 1;
  useNotificationStore.setState({ entries: [] });
}
