// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * The IPC transport seam (04 s4.2 rule 1): commands are called only
 * through `commands.ts` wrappers, which route over this seam. The
 * default talks to the Tauri backend; the harness binds a mock
 * transport (ADR-020's mock backend) so stories and tests run
 * without the shell process.
 *
 * Rejections normalize to the UI error envelope (04 s4.9): backend
 * envelopes pass through untouched; framework-level failures
 * (deserialization, a missing WebView bridge) map to E0001 -
 * "an unexpected internal error occurred" (12 s12.13).
 */
import { invoke } from "@tauri-apps/api/core";
import type { ErrorEnvelope } from "./bindings/ErrorEnvelope";

export type IpcTransport = {
  invoke<T>(command: string, payload?: unknown): Promise<T>;
};

const tauriTransport: IpcTransport = {
  invoke: (command, payload) =>
    invoke(command, payload as Parameters<typeof invoke>[1]),
};

let transport: IpcTransport = tauriTransport;

/** Bind a transport backend (harness: the mock; tests: fixtures). */
export function setIpcTransport(next: IpcTransport): void {
  transport = next;
}

/** Test helper: restore the Tauri transport. */
export function resetIpcTransportForTests(): void {
  transport = tauriTransport;
}

/** Whether a rejection already carries the envelope shape. */
export function isEnvelope(value: unknown): value is ErrorEnvelope {
  return (
    typeof value === "object" &&
    value !== null &&
    "code" in value &&
    "messageKey" in value
  );
}

/** The E0001 envelope for framework-level failures (04 s4.9). */
export function internalEnvelope(detail: string): ErrorEnvelope {
  return {
    code: "E0001",
    severity: "error",
    messageKey: "error.E0001",
    params: {},
    retryable: false,
    recovery: [],
    detail,
  };
}

/** Invoke through the bound transport, normalizing rejections. */
export async function invokeNormalized<T>(
  command: string,
  payload?: unknown,
): Promise<T> {
  try {
    return await transport.invoke<T>(command, payload);
  } catch (rejection: unknown) {
    if (isEnvelope(rejection)) throw rejection;
    // The wrapper contract rejects with the envelope itself (04
    // s4.2: "the TS wrapper rejects with the envelope") - the
    // structured rejection IS the API shape, not an error-handling
    // bug the rule targets.
    // eslint-disable-next-line @typescript-eslint/only-throw-error
    throw internalEnvelope(String(rejection));
  }
}
