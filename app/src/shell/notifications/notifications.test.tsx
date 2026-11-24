// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * Notification fabric tests (T-APP-005): envelope and info payloads
 * normalize into history entries, error rows carry the code + their
 * recovery actions, info routes politely (toasts), and the center
 * renders the list with a working clear action.
 */
import { fireEvent, render, screen } from "@testing-library/react";
import type { JSX, ReactNode } from "react";
import { IntlProvider } from "react-intl";
import { beforeEach, describe, expect, it } from "vitest";
import en from "../../locales/en.json";
import { NotificationCenter } from "./NotificationCenter";
import {
  resetNotificationStoreForTests,
  useNotificationStore,
  type NotifyPayload,
} from "./store";
import { useToastStore } from "../../components/Toast";

function Wrapper({ children }: { children: ReactNode }): JSX.Element {
  return (
    <IntlProvider locale="en" messages={en}>
      {children}
    </IntlProvider>
  );
}

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
  detail: "pid 123 on host",
};

describe("t_app_005 — the notification fabric (FR-014)", () => {
  beforeEach(() => {
    resetNotificationStoreForTests();
    useToastStore.setState({ toasts: [] });
  });

  it("an envelope becomes an error row with code and recovery", () => {
    useNotificationStore.getState().notify(ERROR_PAYLOAD);
    const entries = useNotificationStore.getState().entries;
    expect(entries).toHaveLength(1);
    expect(entries[0].code).toBe("E1007");
    expect(entries[0].recovery.map((action) => action.commandId)).toEqual([
      "project.openReadOnly",
      "project.takeOverLock",
    ]);
    // The toast routes the envelope's message (T048 owns assertive).
    expect(useToastStore.getState().toasts[0].variant).toBe("error");
  });

  it("an info payload routes politely with its message key", () => {
    useNotificationStore.getState().notify({
      severity: "info",
      messageKey: "notifications.empty",
    });
    const entries = useNotificationStore.getState().entries;
    expect(entries[0].kind).toBe("info");
    expect(entries[0].code).toBeUndefined();
    expect(useToastStore.getState().toasts[0].variant).toBe("info");
  });

  it("the center lists entries, renders codes, and clears", () => {
    useNotificationStore.getState().notify(ERROR_PAYLOAD);
    useNotificationStore.getState().notify({
      severity: "info",
      messageKey: "notifications.empty",
    });
    render(
      <Wrapper>
        <NotificationCenter
          flags={{ focusedContext: "global", projectOpen: false }}
        />
      </Wrapper>,
    );
    expect(screen.getByText("E1007")).toBeTruthy();
    expect(screen.getByText("No notifications.")).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "Clear all" }));
    expect(useNotificationStore.getState().entries).toHaveLength(0);
    expect(screen.getByText("No notifications.")).toBeTruthy();
  });

  it("history is newest-first", () => {
    useNotificationStore.getState().notify({
      severity: "info",
      messageKey: "notifications.empty",
    });
    useNotificationStore.getState().notify(ERROR_PAYLOAD);
    const entries = useNotificationStore.getState().entries;
    expect(entries[0].code).toBe("E1007");
    expect(entries[1].code).toBeUndefined();
  });
});
