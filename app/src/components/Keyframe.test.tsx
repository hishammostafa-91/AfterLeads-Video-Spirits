// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * Keyframe control tests (07 §7.6): the stopwatch toggle exposes
 * add/remove names and aria-pressed; the navigator's ◀ ◆ ▶ buttons
 * disable at the boundaries and toggle the playhead keyframe.
 */
import { fireEvent, render, screen } from "@testing-library/react";
import type { JSX, ReactNode } from "react";
import { IntlProvider } from "react-intl";
import { describe, expect, it, vi } from "vitest";
import en from "../locales/en.json";
import { KeyframeToggle } from "./KeyframeToggle";
import { KeyframeNavigator } from "./KeyframeNavigator";

function Wrapper({ children }: { children: ReactNode }): JSX.Element {
  return (
    <IntlProvider locale="en" messages={en}>
      {children}
    </IntlProvider>
  );
}

/** Narrow a located element to its button. */
function asButton(el: HTMLElement): HTMLButtonElement {
  if (!(el instanceof HTMLButtonElement)) {
    throw new TypeError("expected a button element");
  }
  return el;
}

describe("KeyframeToggle (stopwatch state)", () => {
  it("names as add when unkeyframed and remove when keyframed", () => {
    const { rerender } = render(
      <Wrapper>
        <KeyframeToggle isKeyframed={false} onToggle={() => {}} />
      </Wrapper>,
    );
    const add = screen.getByRole("button", { name: "Add keyframe" });
    expect(add.getAttribute("aria-pressed")).toBe("false");
    rerender(
      <Wrapper>
        <KeyframeToggle isKeyframed onToggle={() => {}} />
      </Wrapper>,
    );
    const remove = screen.getByRole("button", { name: "Remove keyframe" });
    expect(remove.getAttribute("aria-pressed")).toBe("true");
  });

  it("toggles the keyframe", () => {
    const onToggle = vi.fn();
    render(
      <Wrapper>
        <KeyframeToggle isKeyframed={false} onToggle={onToggle} />
      </Wrapper>,
    );
    fireEvent.click(screen.getByRole("button", { name: "Add keyframe" }));
    expect(onToggle).toHaveBeenCalledWith(true);
  });
});

describe("KeyframeNavigator (enabled states)", () => {
  it("disables the boundary buttons when no keyframe exists there", () => {
    render(
      <Wrapper>
        <KeyframeNavigator
          hasPrevious={false}
          hasNext={false}
          isOnKeyframe={false}
          onPrevious={() => {}}
          onNext={() => {}}
          onToggle={() => {}}
        />
      </Wrapper>,
    );
    const prev = asButton(
      screen.getByRole("button", { name: "Previous keyframe" }),
    );
    const next = asButton(
      screen.getByRole("button", { name: "Next keyframe" }),
    );
    expect(prev.disabled).toBe(true);
    expect(next.disabled).toBe(true);
    expect(screen.getByRole("button", { name: "Add keyframe" })).toBeTruthy();
  });

  it("enables both arrows mid-track and names the diamond remove on a keyframe", () => {
    render(
      <Wrapper>
        <KeyframeNavigator
          hasPrevious
          hasNext
          isOnKeyframe
          onPrevious={() => {}}
          onNext={() => {}}
          onToggle={() => {}}
        />
      </Wrapper>,
    );
    const prev = asButton(
      screen.getByRole("button", { name: "Previous keyframe" }),
    );
    const next = asButton(
      screen.getByRole("button", { name: "Next keyframe" }),
    );
    expect(prev.disabled).toBe(false);
    expect(next.disabled).toBe(false);
    const diamond = screen.getByRole("button", {
      name: "Remove keyframe",
    });
    expect(diamond.getAttribute("aria-pressed")).toBe("true");
  });
});
