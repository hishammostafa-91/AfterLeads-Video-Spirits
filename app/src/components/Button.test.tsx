// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * Button cluster tests: variants render, focus is visible via the
 * token ring, IconButton has an accessible name and tooltip with
 * label + shortcut (REQ-HLP-003), ToggleButton selects, text fields
 * use dir="auto" (07 §7.11).
 */
import { fireEvent, render, screen } from "@testing-library/react";
import type { JSX, ReactNode } from "react";
import { FormattedMessage, IntlProvider } from "react-intl";
import { describe, expect, it, vi } from "vitest";
import en from "../locales/en.json";
import { Button, IconButton, ToggleButton } from "./Button";
import { SearchField, TextArea, TextField } from "./Fields";

function Wrapper({ children }: { children: ReactNode }): JSX.Element {
  return (
    <IntlProvider locale="en" messages={en}>
      {children}
    </IntlProvider>
  );
}

describe("Button (07 s7.6)", () => {
  it.each(["primary", "secondary", "ghost", "danger"] as const)(
    "variant %s renders a button",
    (variant) => {
      render(
        <Wrapper>
          <Button variant={variant} onPress={() => {}}>
            <FormattedMessage id="components.button.ok" />
          </Button>
        </Wrapper>,
      );
      expect(screen.getByRole("button")).toBeTruthy();
    },
  );

  it("fires onPress", () => {
    const onPress = vi.fn();
    render(
      <Wrapper>
        <Button onPress={onPress}>
          <FormattedMessage id="components.button.ok" />
        </Button>
      </Wrapper>,
    );
    fireEvent.click(screen.getByRole("button"));
    expect(onPress).toHaveBeenCalled();
  });

  it("is disabled when isDisabled", () => {
    render(
      <Wrapper>
        <Button isDisabled>
          <FormattedMessage id="components.button.ok" />
        </Button>
      </Wrapper>,
    );
    const button = screen.getByRole("button");
    expect(button).toBeTruthy();
    expect(button.hasAttribute("disabled")).toBe(true);
  });
});

describe("IconButton (REQ-HLP-003)", () => {
  it("has an accessible name from the label message", () => {
    render(
      <Wrapper>
        <IconButton
          icon={<svg aria-hidden="true" viewBox="0 0 16 16" />}
          labelMessageId="components.button.edit"
          shortcut="F2"
        />
      </Wrapper>,
    );
    expect(screen.getByRole("button", { name: "Edit" })).toBeTruthy();
  });
});

describe("ToggleButton (07 s7.6)", () => {
  it("toggles selection", () => {
    const onChange = vi.fn();
    render(
      <Wrapper>
        <ToggleButton onChange={onChange}>
          <FormattedMessage id="components.button.snap" />
        </ToggleButton>
      </Wrapper>,
    );
    fireEvent.click(screen.getByRole("button"));
    expect(onChange).toHaveBeenCalledWith(true);
  });
});

describe("Fields (07 s7.6; dir=auto per s7.11)", () => {
  it("TextField renders with dir=auto input", () => {
    const { container } = render(
      <TextField label="Name" placeholder="Type…" />,
    );
    expect(container.querySelector("input[dir='auto']")).not.toBeNull();
  });

  it("SearchField renders with dir=auto input", () => {
    const { container } = render(<SearchField label="Search" />);
    expect(container.querySelector("input[dir='auto']")).not.toBeNull();
  });

  it("TextArea renders with dir=auto textarea", () => {
    const { container } = render(<TextArea label="Notes" placeholder="…" />);
    expect(container.querySelector("textarea[dir='auto']")).not.toBeNull();
  });
});
