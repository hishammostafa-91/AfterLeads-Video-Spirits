// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * Notification stories (T030): the center with an error row (code +
 * recovery buttons) and an info row, the empty state, and mocked
 * app:notify events feeding the fabric through the event backend.
 */
import { useEffect } from "react";
import { NotificationCenter } from "../../shell/notifications/NotificationCenter";
import { setEventBackend } from "../../ipc/events";
import {
  resetNotificationStoreForTests,
  useNotificationStore,
  type NotifyPayload,
} from "../../shell/notifications/store";
import { resetShellPanelsForTests } from "../../shell/notifications/shellPanels";
import type { StoryFile } from "./types";

/** The fixture payloads (envelope + info, 04 s4.10 shapes). */
const ERROR_PAYLOAD: NotifyPayload = {
  code: "E1007",
  severity: "warning",
  messageKey: "error.E1007",
  params: { takeoverAvailable: "false" },
  retryable: false,
  recovery: [
    { commandId: "project.openReadOnly", labelKey: "action.openReadOnly" },
    { commandId: "project.takeOverLock", labelKey: "action.takeOverLock" },
  ],
  detail: "pid 123 on HOST-PC",
};

function CenterStory(payloads: NotifyPayload[]): () => React.JSX.Element {
  return function NotificationStoryCard(): React.JSX.Element {
    useEffect(() => {
      resetNotificationStoreForTests();
      for (const payload of payloads) {
        useNotificationStore.getState().notify(payload);
      }
    }, []);
    return (
      <NotificationCenter
        flags={{ focusedContext: "global", projectOpen: true }}
      />
    );
  };
}

/** The mock event backend: emits app:notify payloads once. */
function EventStory(): React.JSX.Element {
  useEffect(() => {
    resetNotificationStoreForTests();
    resetShellPanelsForTests();
    setEventBackend({
      subscribe: (event, handler) => {
        if (event === "app:notify") {
          window.setTimeout(() => {
            handler({ event, id: 0, payload: ERROR_PAYLOAD });
            handler({
              event,
              id: 1,
              payload: { severity: "info", messageKey: "notifications.empty" },
            });
          }, 0);
        }
        return Promise.resolve(() => undefined);
      },
    });
    return () => {
      setEventBackend({
        subscribe: () => Promise.resolve(() => undefined),
      });
    };
  }, []);
  return (
    <NotificationCenter
      flags={{ focusedContext: "global", projectOpen: true }}
    />
  );
}

export const notificationStories: StoryFile = {
  id: "notifications",
  stories: [
    {
      id: "notifications.empty",
      render: () => CenterStory([])(),
    },
    {
      id: "notifications.error",
      render: () => CenterStory([ERROR_PAYLOAD])(),
    },
    {
      id: "notifications.mixed",
      render: () =>
        CenterStory([
          { severity: "info", messageKey: "notifications.empty" },
          ERROR_PAYLOAD,
        ])(),
    },
    { id: "notifications.events", render: () => <EventStory /> },
  ],
};
