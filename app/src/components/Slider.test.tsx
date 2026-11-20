// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
/**
 * Slider + AngleDial tests (07 §7.6): slider role and keyboard
 * selection, dial angle math (normalization, pointer→angle), dial
 * keyboard stepping, and double-click reset.
 */
import { fireEvent, render, screen } from "@testing-library/react";
import type { JSX, ReactNode } from "react";
import { useState } from "react";
import { IntlProvider } from "react-intl";
import { describe, expect, it, vi } from "vitest";
import en from "../locales/en.json";
import { AngleDial, normalizeAngle, pointToAngle, Slider } from "./Slider";

function Wrapper({ children }: { children: ReactNode }): JSX.Element {
  return (
    <IntlProvider locale="en" messages={en}>
      {children}
    </IntlProvider>
  );
}

describe("Slider (07 s7.6)", () => {
  it("exposes the slider role with native range semantics", () => {
    render(
      <Wrapper>
        <Slider label="Opacity" minValue={0} maxValue={100} defaultValue={40} />
      </Wrapper>,
    );
    const slider = screen.getByRole("slider", { name: "Opacity" });
    expect(slider.getAttribute("min")).toBe("0");
    expect(slider.getAttribute("max")).toBe("100");
    expect(slider.getAttribute("value")).toBe("40");
    expect(slider.getAttribute("aria-valuetext")).toBe("40");
  });

  it("steps with the arrow keys through onChange", () => {
    const onChange = vi.fn();
    render(
      <Wrapper>
        <Slider
          label="Opacity"
          minValue={0}
          maxValue={100}
          value={50}
          onChange={onChange}
        />
      </Wrapper>,
    );
    const slider = screen.getByRole("slider", { name: "Opacity" });
    fireEvent.keyDown(slider, { key: "ArrowRight" });
    expect(onChange).toHaveBeenCalledWith(51);
    fireEvent.keyDown(slider, { key: "ArrowLeft" });
    expect(onChange).toHaveBeenCalledWith(49);
  });
});

describe("AngleDial (07 s7.6)", () => {
  it("normalizes angles into [0, 360)", () => {
    expect(normalizeAngle(-10)).toBe(350);
    expect(normalizeAngle(370)).toBe(10);
    expect(normalizeAngle(0)).toBe(0);
  });

  it("maps pointer positions to angles (0 at 12 o'clock, clockwise)", () => {
    expect(pointToAngle(0, -1)).toBe(0);
    expect(pointToAngle(1, 0)).toBe(90);
    expect(pointToAngle(0, 1)).toBe(180);
    expect(pointToAngle(-1, 0)).toBe(270);
  });

  it("exposes a slider role with 0-360 and valuetext degrees", () => {
    render(
      <Wrapper>
        <AngleDial value={135} onChange={() => {}} label="Angle" />
      </Wrapper>,
    );
    const dial = screen.getByRole("slider", { name: "Angle" });
    expect(dial.getAttribute("aria-valuemin")).toBe("0");
    expect(dial.getAttribute("aria-valuemax")).toBe("360");
    expect(dial.getAttribute("aria-valuenow")).toBe("135");
    expect(dial.getAttribute("aria-valuetext")).toBe("135\u00b0");
  });

  it("steps with arrows (Shift x10) and wraps", () => {
    function DialHarness(): JSX.Element {
      const [angle, setAngle] = useState(359);
      return <AngleDial value={angle} onChange={setAngle} label="Angle" />;
    }
    render(
      <Wrapper>
        <DialHarness />
      </Wrapper>,
    );
    const dial = screen.getByRole("slider", { name: "Angle" });
    fireEvent.keyDown(dial, { key: "ArrowRight" });
    expect(dial.getAttribute("aria-valuenow")).toBe("0");
    fireEvent.keyDown(dial, { key: "ArrowRight", shiftKey: true });
    expect(dial.getAttribute("aria-valuenow")).toBe("10");
  });

  it("resets to the default on double-click", () => {
    const onChange = vi.fn();
    render(
      <Wrapper>
        <AngleDial
          value={200}
          defaultValue={90}
          onChange={onChange}
          label="Angle"
        />
      </Wrapper>,
    );
    fireEvent.doubleClick(screen.getByRole("slider", { name: "Angle" }));
    expect(onChange).toHaveBeenCalledWith(90);
  });
});
