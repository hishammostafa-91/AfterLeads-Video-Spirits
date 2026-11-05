// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * Toast (07 §7.6; research D6): a region + polite announcement — the
 * region is always mounted (so screen readers catch insertions), each
 * toast is announced politely through `aria-live`, and toasts
 * auto-dismiss. Composed rather than RAC's UNSTABLE_Toast (stable API
 * guarantee; the primitive is a list in a live region).
 */
import type { JSX } from "react";
import { Check, Info, TriangleAlert } from "lucide-react";
import { useIntl } from "react-intl";
import { create } from "zustand";
import styles from "./Toast.module.css";

/** How long a toast stays visible (auto-dismiss). */
export const TOAST_DURATION_MS = 4000;

export type ToastVariant = "info" | "success" | "error";

export interface ToastItem {
  id: number;
  /** Locale message ID of the toast text. */
  messageId: string;
  variant: ToastVariant;
}

interface ToastState {
  toasts: ToastItem[];
  push: (messageId: string, variant?: ToastVariant) => void;
  dismiss: (id: number) => void;
}

let nextToastId = 1;

/** The toast queue (module store; one region consumes it). */
export const useToastStore = create<ToastState>((set) => ({
  toasts: [],
  push: (messageId, variant = "info") => {
    const id = nextToastId;
    nextToastId += 1;
    set((state) => ({ toasts: [...state.toasts, { id, messageId, variant }] }));
    window.setTimeout(() => {
      set((state) => ({
        toasts: state.toasts.filter((toast) => toast.id !== id),
      }));
    }, TOAST_DURATION_MS);
  },
  dismiss: (id) => {
    set((state) => ({
      toasts: state.toasts.filter((toast) => toast.id !== id),
    }));
  },
}));

/** Options for {@link showToast}. */
export interface ShowToastOptions {
  /** Locale message ID of the toast text. */
  messageId: string;
  variant?: ToastVariant;
}

/** Show a toast from anywhere (non-hook callers included). */
export function showToast(options: ShowToastOptions): void {
  useToastStore.getState().push(options.messageId, options.variant ?? "info");
}

function ToastIcon({ variant }: { variant: ToastVariant }): JSX.Element {
  if (variant === "success") {
    return <Check size={16} aria-hidden="true" />;
  }
  if (variant === "error") {
    return <TriangleAlert size={16} aria-hidden="true" />;
  }
  return <Info size={16} aria-hidden="true" />;
}

/**
 * The toast region: always mounted, polite, named — toasts appearing
 * in it are announced by assistive technology (07 §7.6).
 */
export function ToastRegion(): JSX.Element {
  const toasts = useToastStore((state) => state.toasts);
  const { formatMessage } = useIntl();
  return (
    <div
      role="region"
      aria-live="polite"
      aria-label={formatMessage({ id: "components.toast.region" })}
      className={styles.region}
    >
      {toasts.map((toast) => (
        <div
          key={toast.id}
          className={styles.toast}
          data-variant={toast.variant}
        >
          <ToastIcon variant={toast.variant} />
          <span className={styles.message}>
            {formatMessage({ id: toast.messageId })}
          </span>
        </div>
      ))}
    </div>
  );
}
