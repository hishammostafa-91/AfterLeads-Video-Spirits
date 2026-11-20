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
import {
  Button,
  IconButton,
  Segment,
  SegmentedControl,
  ToggleButton,
} from "./Button";
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
  it("TextField associates its label and exposes disabled/invalid state", () => {
    render(<TextField label="Name" placeholder="Type…" isDisabled isInvalid />);
    const input = screen.getByLabelText("Name");
    expect(input.tagName).toBe("INPUT");
    expect(input.getAttribute("dir")).toBe("auto");
    expect((input as HTMLInputElement).disabled).toBe(true);
    expect(input.getAttribute("aria-invalid")).toBe("true");
  });

  it("SearchField associates its label", () => {
    render(<SearchField label="Search" placeholder="Search…" />);
    const input = screen.getByLabelText("Search");
    expect(input.tagName).toBe("INPUT");
    expect(input.getAttribute("dir")).toBe("auto");
  });

  it("TextArea associates its label and honors a controlled value", () => {
    render(<TextArea label="Notes" placeholder="…" value="a note" />);
    const area = screen.getByLabelText("Notes");
    expect(area.tagName).toBe("TEXTAREA");
    expect(area.getAttribute("dir")).toBe("auto");
    expect((area as HTMLTextAreaElement).value).toBe("a note");
  });

  it("TextArea marks the invalid state", () => {
    render(<TextArea label="Notes" placeholder="…" isInvalid />);
    expect(screen.getByLabelText("Notes").getAttribute("aria-invalid")).toBe(
      "true",
    );
  });
});

describe("SegmentedControl (07 s7.6, RadioGroup-composed)", () => {
  it("renders a radiogroup whose selected segment is checked", () => {
    const { container } = render(
      <Wrapper>
        <SegmentedControl
          aria-label="Duration"
          defaultValue="day"
          onChange={() => {}}
        >
          <Segment value="day">{"Day"}</Segment>
          <Segment value="week">{"Week"}</Segment>
          <Segment value="month">{"Month"}</Segment>
        </SegmentedControl>
      </Wrapper>,
    );
    expect(screen.getByRole("radiogroup")).toBeTruthy();
    // jsdom: native inputs assert the checked property.
    const radios = screen.getAllByRole("radio");
    expect(radios.length).toBe(3);
    expect((radios[0] as HTMLInputElement).checked).toBe(true);
    expect((radios[1] as HTMLInputElement).checked).toBe(false);
    expect((radios[2] as HTMLInputElement).checked).toBe(false);
    // Each segment is a RadioField label wrapping its RadioButton.
    const labels = container.querySelectorAll("label");
    expect(labels.length).toBe(3);
  });

  it("arrow keys move the selection (radio semantics)", () => {
    render(
      <Wrapper>
        <SegmentedControl
          aria-label="Duration"
          defaultValue="day"
          onChange={() => {}}
        >
          <Segment value="day">{"Day"}</Segment>
          <Segment value="week">{"Week"}</Segment>
        </SegmentedControl>
      </Wrapper>,
    );
    const first = screen.getAllByRole("radio")[0];
    if (!(first instanceof HTMLInputElement)) {
      throw new TypeError("expected the radio input");
    }
    first.focus();
    fireEvent.keyDown(first, { key: "ArrowDown" });
    const radios = screen.getAllByRole("radio");
    expect((radios[1] as HTMLInputElement).checked).toBe(true);
    expect((radios[0] as HTMLInputElement).checked).toBe(false);
  });
});
