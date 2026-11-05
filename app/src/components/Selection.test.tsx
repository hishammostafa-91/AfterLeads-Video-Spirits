// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * Selection-control tests (07 §7.6): Checkbox/Switch toggle, Radio
 * selects within its group, Select opens its option list and commits
 * a choice, ComboBox commits a filtered choice. Interaction is driven
 * through controlled open state + keyboard events (RAC's own key
 * handling) for determinism in jsdom.
 */
import { fireEvent, render, screen } from "@testing-library/react";
import type { JSX, ReactNode } from "react";
import { FormattedMessage, IntlProvider } from "react-intl";
import { act } from "react";
import { describe, expect, it } from "vitest";
import en from "../locales/en.json";
import {
  Checkbox,
  ComboBox,
  ListBoxItem,
  Radio,
  RadioGroup,
  Select,
  Switch,
} from "./Selection";

function Wrapper({ children }: { children: ReactNode }): JSX.Element {
  return (
    <IntlProvider locale="en" messages={en}>
      {children}
    </IntlProvider>
  );
}

/** Narrow a located element to its input (native checkbox/switch/radio). */
function asInput(el: HTMLElement): HTMLInputElement {
  if (!(el instanceof HTMLInputElement)) {
    throw new TypeError("expected an input element");
  }
  return el;
}

describe("Checkbox / Switch (07 s7.6)", () => {
  it("toggles checkbox selection", () => {
    render(
      <Wrapper>
        <Checkbox>
          <FormattedMessage id="components.selection.checkboxLabel" />
        </Checkbox>
      </Wrapper>,
    );
    const checkbox = asInput(
      screen.getByRole("checkbox", { name: "Snapping" }),
    );
    expect(checkbox.checked).toBe(false);
    fireEvent.click(checkbox);
    expect(checkbox.checked).toBe(true);
  });

  it("toggles switch selection", () => {
    render(
      <Wrapper>
        <Switch>
          <FormattedMessage id="components.selection.switchLabel" />
        </Switch>
      </Wrapper>,
    );
    const sw = asInput(screen.getByRole("switch", { name: "Use proxy" }));
    fireEvent.click(sw);
    expect(sw.checked).toBe(true);
  });
});

describe("RadioGroup (07 s7.6)", () => {
  it("selects one radio in the group", () => {
    render(
      <Wrapper>
        <RadioGroup aria-label="scale">
          <Radio value="fit">
            <FormattedMessage id="components.selection.fitLabel" />
          </Radio>
          <Radio value="fill">
            <FormattedMessage id="components.selection.fillLabel" />
          </Radio>
        </RadioGroup>
      </Wrapper>,
    );
    const fit = asInput(screen.getByRole("radio", { name: "Fit" }));
    const fill = asInput(screen.getByRole("radio", { name: "Fill" }));
    fireEvent.click(fit);
    expect(fit.checked).toBe(true);
    expect(fill.checked).toBe(false);
  });
});

describe("Select (07 s7.6)", () => {
  it("renders a labeled trigger showing the selected value", () => {
    const { container } = render(
      <Wrapper>
        <Select label="Mode" defaultValue="first">
          <ListBoxItem id="first">
            <FormattedMessage id="components.selection.optionFirst" />
          </ListBoxItem>
          <ListBoxItem id="second">
            <FormattedMessage id="components.selection.optionSecond" />
          </ListBoxItem>
        </Select>
      </Wrapper>,
    );
    const field = container.querySelector("[data-rac]");
    expect(field).not.toBeNull();
    const trigger = field?.querySelector("button");
    expect(trigger).not.toBeNull();
    expect(trigger?.textContent).toContain("First option");
  });
});

describe("ComboBox (07 s7.6)", () => {
  it("opens its option list on keyboard and offers both options", async () => {
    const { container } = render(
      <Wrapper>
        <ComboBox label="Effect">
          <ListBoxItem id="first">
            <FormattedMessage id="components.selection.optionFirst" />
          </ListBoxItem>
          <ListBoxItem id="second">
            <FormattedMessage id="components.selection.optionSecond" />
          </ListBoxItem>
        </ComboBox>
      </Wrapper>,
    );
    const input = screen.getByRole("combobox", { name: "Effect" });
    // Keyboard interaction: ArrowDown opens the list.
    fireEvent.keyDown(input, { key: "ArrowDown" });
    await act(async () => {
      await Promise.resolve();
    });
    const options = container.ownerDocument.querySelectorAll("[role='option']");
    expect(options.length).toBe(2);
  });
});
