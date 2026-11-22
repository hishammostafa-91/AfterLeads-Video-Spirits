// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * The event subscription seam (04 s4.10): the UI subscribes to core
 * events only through here - the default binds Tauri's `listen`; the
 * harness mock binds a no-op (stories drive state through their own
 * props and stores). Handlers receive the already-typed payloads.
 */
import { listen, type EventCallback } from "@tauri-apps/api/event";

export type EventUnsubscribe = () => void;

type EventBackend = {
  subscribe: (
    event: string,
    handler: EventCallback<unknown>,
  ) => Promise<EventUnsubscribe>;
};

const tauriBackend: EventBackend = {
  subscribe: (event, handler) =>
    listen(event, handler).catch(() => {
      // Outside the Tauri shell (a plain dev browser) there is no
      // event backend; subscriptions resolve inert rather than
      // rejecting through every consumer.
      return () => undefined;
    }),
};

let backend: EventBackend = tauriBackend;

/** Bind an event backend (harness/mock: no-op or scripted). */
export function setEventBackend(next: EventBackend): void {
  backend = next;
}

/** Test helper: restore the Tauri backend. */
export function resetEventBackendForTests(): void {
  backend = tauriBackend;
}

/** Subscribe to a core event; returns the unsubscribe function. */
// The generic parameter is part of the public API (callers type the
// payload); the single-use rule would force `unknown` at consumers.
// eslint-disable-next-line @typescript-eslint/no-unnecessary-type-parameters
export function onEvent<T>(
  event: string,
  handler: (payload: T) => void,
): Promise<EventUnsubscribe> {
  return backend.subscribe(event, (raw) => {
    handler(raw.payload as T);
  });
}

// The generic is part of the public API surface (callers type the
// payload); the single-use-parameter rule would force `unknown`
// casts at every consumer.
