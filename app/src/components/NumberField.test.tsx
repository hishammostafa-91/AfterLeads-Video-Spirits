// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * NumberField tests (07 §7.6): scrub modifier factors, typed math
 * (set / `+N` / `-N` / `*N`), wheel-when-focused, unit suffix
 * rendering, mixed "—" state, and the stepper/label composition.
 */
import { fireEvent, render } from "@testing-library/react";
import type { JSX, ReactNode } from "react";
import { IntlProvider } from "react-intl";
import { describe, expect, it, vi } from "vitest";
import en from "../locales/en.json";
import {
  applyEntry,
  NumberField,
  parseNumberEntry,
  scrubMultiplier,
} from "./NumberField";

function Wrapper({ children }: { children: ReactNode }): JSX.Element {
  return (
    <IntlProvider locale="en" messages={en}>
      {children}
    </IntlProvider>
  );
}

/** The field's (only) input — the label and group also carry "Scale". */
function fieldInput(container: HTMLElement): HTMLInputElement {
  const el = container.querySelector("input");
  if (!(el instanceof HTMLInputElement)) {
    throw new TypeError("expected the number input");
  }
  return el;
}

describe("scrub modifiers (07 s7.6)", () => {
  it("Shift x10, Ctrl x0.1, plain x1", () => {
    expect(scrubMultiplier({ shiftKey: false, ctrlKey: false })).toBe(1);
    expect(scrubMultiplier({ shiftKey: true, ctrlKey: false })).toBe(10);
    expect(scrubMultiplier({ shiftKey: false, ctrlKey: true })).toBe(0.1);
  });
});

describe("typed math (07 s7.6)", () => {
  it("parses unsigned numbers as absolute sets", () => {
    expect(parseNumberEntry("42")).toEqual({ kind: "set", value: 42 });
    expect(parseNumberEntry("3.5")).toEqual({ kind: "set", value: 3.5 });
  });

  it("parses relative +N / -N / *N expressions", () => {
    expect(parseNumberEntry("+10")).toEqual({
      kind: "relative",
      op: "+",
      operand: 10,
    });
    expect(parseNumberEntry("-2")).toEqual({
      kind: "relative",
      op: "-",
      operand: 2,
    });
    expect(parseNumberEntry("*3")).toEqual({
      kind: "relative",
      op: "*",
      operand: 3,
    });
  });

  it("rejects non-numeric text", () => {
    expect(parseNumberEntry("abc")).toBeNull();
  });

  it("applies entries against the current value", () => {
    expect(applyEntry(5, { kind: "set", value: 9 })).toBe(9);
    expect(applyEntry(5, { kind: "relative", op: "+", operand: 10 })).toBe(15);
    expect(applyEntry(5, { kind: "relative", op: "-", operand: 2 })).toBe(3);
    expect(applyEntry(5, { kind: "relative", op: "*", operand: 2 })).toBe(10);
  });

  it("commits typed math on Enter through onChange", () => {
    const onChange = vi.fn();
    const { container } = render(
      <Wrapper>
        <NumberField label="Scale" value={5} onChange={onChange} suffix="%" />
      </Wrapper>,
    );
    const input = fieldInput(container);
    fireEvent.change(input, { target: { value: "+10" } });
    fireEvent.keyDown(input, { key: "Enter" });
    expect(onChange).toHaveBeenCalledWith(15);
  });

  it("clamps the committed math result to the bounds", () => {
    const onChange = vi.fn();
    const { container } = render(
      <Wrapper>
        <NumberField
          label="Scale"
          value={5}
          minValue={0}
          maxValue={10}
          onChange={onChange}
        />
      </Wrapper>,
    );
    const input = fieldInput(container);
    fireEvent.change(input, { target: { value: "*3" } });
    fireEvent.keyDown(input, { key: "Enter" });
    expect(onChange).toHaveBeenCalledWith(10);
  });
});

describe("wheel-when-focused (07 s7.6)", () => {
  it("steps by the wheel direction while focused", () => {
    const onChange = vi.fn();
    const { container } = render(
      <Wrapper>
        <NumberField label="Scale" value={5} minValue={0} onChange={onChange} />
      </Wrapper>,
    );
    const input = fieldInput(container);
    input.focus();
    fireEvent.wheel(input.closest("div") ?? input, { deltaY: -100 });
    expect(onChange).toHaveBeenCalledWith(6);
  });

  it("ignores the wheel when the field is not focused", () => {
    const onChange = vi.fn();
    const { container } = render(
      <Wrapper>
        <NumberField label="Scale" value={5} minValue={0} onChange={onChange} />
      </Wrapper>,
    );
    const input = fieldInput(container);
    fireEvent.wheel(input.closest("div") ?? input, { deltaY: -100 });
    expect(onChange).not.toHaveBeenCalled();
  });
});

describe("composition (07 s7.6)", () => {
  it("renders the stepper buttons and the unit suffix", () => {
    const { container } = render(
      <Wrapper>
        <NumberField label="Scale" value={50} suffix="%" />
      </Wrapper>,
    );
    expect(container.querySelector("input")).toBeTruthy();
    const buttons = container.querySelectorAll("button");
    expect(buttons.length).toBe(2);
    expect(container.textContent).toContain("%");
  });

  it("shows the mixed dash placeholder in the mixed state", () => {
    const { container } = render(
      <Wrapper>
        <NumberField label="Scale" isMixed suffix="%" />
      </Wrapper>,
    );
    expect(fieldInput(container).placeholder).toBe("—");
  });
});
