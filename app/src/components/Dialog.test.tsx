// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * Overlay + Tabs tests (07 §7.6): Dialog and AlertDialog render with
 * their roles and safest-control initial focus and report close
 * through onOpenChange (RAC's exit animation is a browser concern —
 * the Playwright harness covers the real-browser matrix), Popover and
 * Tooltip render through controlled triggers, the Toast region
 * announces politely and auto-dismisses, and Tabs select via the
 * keyboard.
 */
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import type { JSX, ReactNode } from "react";
import { FormattedMessage, IntlProvider } from "react-intl";
import { act } from "react";
import { DialogTrigger, TooltipTrigger } from "react-aria-components";
import { describe, expect, it, vi } from "vitest";
import en from "../locales/en.json";
import { Button } from "./Button";
import { AlertDialog, Dialog, ModalOverlay, Popover } from "./Dialog";
import { Tab, TabList, TabPanel, Tabs } from "./Tabs";
import { ToastRegion, showToast, useToastStore } from "./Toast";
import { TOOLTIP_DELAY_MS, Tooltip } from "./Tooltip";

function Wrapper({ children }: { children: ReactNode }): JSX.Element {
  return (
    <IntlProvider locale="en" messages={en}>
      {children}
    </IntlProvider>
  );
}

describe("Dialog (07 s7.6)", () => {
  it("renders a dialog and reports close through onOpenChange", async () => {
    const onOpenChange = vi.fn();
    render(
      <Wrapper>
        <ModalOverlay isOpen onOpenChange={onOpenChange}>
          <Dialog>
            {({ close }) => (
              <Button onPress={close}>
                <FormattedMessage id="components.dialog.cancel" />
              </Button>
            )}
          </Dialog>
        </ModalOverlay>
      </Wrapper>,
    );
    expect(screen.getByRole("dialog")).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "Cancel" }));
    await act(async () => {
      await Promise.resolve();
    });
    expect(onOpenChange).toHaveBeenCalledWith(false);
  });
});

describe("AlertDialog (role=alertdialog; safest control first)", () => {
  it("focuses the cancel button first", async () => {
    render(
      <Wrapper>
        <ModalOverlay isOpen>
          <AlertDialog
            title={<FormattedMessage id="components.dialog.discardTitle" />}
            cancelLabel={<FormattedMessage id="components.dialog.cancel" />}
            actionLabel={<FormattedMessage id="components.dialog.discard" />}
          >
            <FormattedMessage id="components.dialog.discardBody" />
          </AlertDialog>
        </ModalOverlay>
      </Wrapper>,
    );
    expect(screen.getByRole("alertdialog")).toBeTruthy();
    const cancel = screen.getByRole("button", { name: "Cancel" });
    await waitFor(() => {
      expect(cancel).toBe(document.activeElement);
    });
  });

  it("invokes the action on the danger button", () => {
    const onAction = vi.fn();
    render(
      <Wrapper>
        <ModalOverlay isOpen>
          <AlertDialog
            title={<FormattedMessage id="components.dialog.discardTitle" />}
            cancelLabel={<FormattedMessage id="components.dialog.cancel" />}
            actionLabel={<FormattedMessage id="components.dialog.discard" />}
            onAction={onAction}
          >
            <FormattedMessage id="components.dialog.discardBody" />
          </AlertDialog>
        </ModalOverlay>
      </Wrapper>,
    );
    fireEvent.click(screen.getByRole("button", { name: "Discard" }));
    expect(onAction).toHaveBeenCalled();
  });
});

describe("Popover", () => {
  it("renders its content through a controlled trigger", () => {
    render(
      <Wrapper>
        <DialogTrigger isOpen>
          <Button>
            <FormattedMessage id="components.popover.filters" />
          </Button>
          <Popover>
            <p className="popover-body">
              <FormattedMessage id="components.popover.body" />
            </p>
          </Popover>
        </DialogTrigger>
      </Wrapper>,
    );
    expect(screen.getByRole("dialog")).toBeTruthy();
    expect(screen.getByText("Popover content")).toBeTruthy();
  });
});

describe("Tooltip (500 ms)", () => {
  it("exposes the canonical delay and renders role=tooltip when open", () => {
    expect(TOOLTIP_DELAY_MS).toBe(500);
    render(
      <Wrapper>
        <TooltipTrigger isOpen delay={TOOLTIP_DELAY_MS}>
          <Button>
            <FormattedMessage id="components.tooltip.zoomIn" />
          </Button>
          <Tooltip>
            <FormattedMessage id="components.tooltip.zoomIn" />
          </Tooltip>
        </TooltipTrigger>
      </Wrapper>,
    );
    expect(screen.getByRole("tooltip")).toBeTruthy();
  });
});

describe("Toast (region + polite announcement)", () => {
  it("renders an always-present polite region and lists toasts", () => {
    const { unmount } = render(
      <Wrapper>
        <ToastRegion />
      </Wrapper>,
    );
    const region = screen.getByRole("region");
    expect(region.getAttribute("aria-live")).toBe("polite");
    expect(region.textContent).toBe("");
    act(() => {
      showToast({ messageId: "components.toast.saved", variant: "success" });
    });
    expect(screen.getByText("Project saved")).toBeTruthy();
    unmount();
    useToastStore.setState({ toasts: [] });
  });

  it("auto-dismisses after the duration", () => {
    vi.useFakeTimers();
    try {
      render(
        <Wrapper>
          <ToastRegion />
        </Wrapper>,
      );
      act(() => {
        showToast({ messageId: "components.toast.saved" });
      });
      act(() => {
        vi.advanceTimersByTime(4001);
      });
      expect(useToastStore.getState().toasts).toHaveLength(0);
    } finally {
      vi.useRealTimers();
    }
  });
});

describe("Tabs (07 s7.6)", () => {
  it("selects a tab via the keyboard and shows its panel", async () => {
    render(
      <Wrapper>
        <Tabs defaultSelectedKey="properties">
          <TabList aria-label="Inspector">
            <Tab id="properties">
              <FormattedMessage id="components.tabs.properties" />
            </Tab>
            <Tab id="effects">
              <FormattedMessage id="components.tabs.effects" />
            </Tab>
          </TabList>
          <TabPanel id="properties">
            <FormattedMessage id="components.tabs.properties" />
          </TabPanel>
          <TabPanel id="effects">
            <FormattedMessage id="components.tabs.effects" />
          </TabPanel>
        </Tabs>
      </Wrapper>,
    );
    const tabs = screen.getAllByRole("tab");
    expect(tabs.length).toBe(2);
    expect(tabs[0].getAttribute("aria-selected")).toBe("true");
    // RAC TabList: ArrowRight moves selection (selection follows focus).
    fireEvent.keyDown(tabs[0], { key: "ArrowRight" });
    await act(async () => {
      await Promise.resolve();
    });
    expect(tabs[1].getAttribute("aria-selected")).toBe("true");
    expect(screen.getAllByRole("tabpanel").length).toBe(1);
  });
});
